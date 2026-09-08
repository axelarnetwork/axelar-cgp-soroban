use soroban_token_sdk::events::{Approve, Burn, MintWithAmountOnly, TransferWithAmountOnly};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;
use stellar_axelar_std::events::Event;
use stellar_axelar_std::interfaces::OwnableInterface;
use stellar_axelar_std::token::StellarAssetInterface;
use stellar_axelar_std::{
    assert_with_error, contract, contractimpl, ensure, interfaces, only_owner, soroban_sdk,
    Address, BytesN, Env, MuxedAddress, String, Upgradable,
};

use crate::error::ContractError;
use crate::event::{MinterAddedEvent, MinterRemovedEvent, SetAdminEvent};
use crate::interface::InterchainTokenInterface;
use crate::storage::{self, AllowanceDataKey, AllowanceValue};

#[contract]
#[derive(Upgradable)]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(
        env: Env,
        owner: Address,
        minter: Option<Address>,
        token_id: BytesN<32>,
        token_metadata: TokenMetadata,
    ) {
        interfaces::set_owner(&env, &owner);

        Self::write_metadata(&env, token_metadata);

        storage::set_token_id(&env, &token_id);

        if let Some(minter) = minter {
            storage::set_minter_status(&env, minter.clone());

            MinterAddedEvent { minter }.emit(&env);
        }
    }
}

#[contractimpl]
impl OwnableInterface for InterchainToken {
    #[allow_during_migration]
    fn owner(env: &Env) -> Address {
        interfaces::owner(env)
    }

    fn transfer_ownership(env: &Env, new_owner: Address) {
        let old_owner = Self::owner(env);

        interfaces::transfer_ownership::<Self>(env, new_owner.clone());

        SetAdminEvent {
            admin: old_owner,
            new_admin: new_owner,
        }
        .emit(env);
    }
}

// Note: Some methods below are intentionally unimplemented as they are not supported by this token.
// Those are the admin mutations `set_authorized` and `clawback` — freezing an account and
// confiscating a balance are capabilities this token deliberately does not have, so panicking
// correctly signals "not supported". Read-only queries do not panic: they have a truthful answer.
#[contractimpl]
impl StellarAssetInterface for InterchainToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        Self::read_allowance(&env, from, spender).amount
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        Self::validate_amount(&env, amount);

        Self::write_allowance(
            &env,
            from.clone(),
            spender.clone(),
            amount,
            expiration_ledger,
        );

        Approve {
            from,
            spender,
            amount,
            expiration_ledger,
        }
        .publish(&env);
    }

    fn balance(env: Env, id: Address) -> i128 {
        storage::try_balance(&env, id).unwrap_or_default()
    }

    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();

        Self::validate_amount(&env, amount);
        Self::spend_balance(&env, from.clone(), amount);
        let to_address = to.address();
        Self::receive_balance(&env, to_address.clone(), amount);

        TransferWithAmountOnly {
            from,
            to: to_address,
            amount,
        }
        .publish(&env);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        Self::validate_amount(&env, amount);
        Self::spend_allowance(&env, from.clone(), spender, amount);
        Self::spend_balance(&env, from.clone(), amount);
        Self::receive_balance(&env, to.clone(), amount);

        TransferWithAmountOnly { from, to, amount }.publish(&env);
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        Self::validate_amount(&env, amount);
        Self::spend_balance(&env, from.clone(), amount);

        Burn { from, amount }.publish(&env);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        Self::validate_amount(&env, amount);
        Self::spend_allowance(&env, from.clone(), spender, amount);
        Self::spend_balance(&env, from.clone(), amount);

        Burn { from, amount }.publish(&env);
    }

    fn decimals(env: Env) -> u32 {
        TokenUtils::new(&env).metadata().get_metadata().decimal
    }

    fn name(env: Env) -> String {
        TokenUtils::new(&env).metadata().get_metadata().name
    }

    fn symbol(env: Env) -> String {
        TokenUtils::new(&env).metadata().get_metadata().symbol
    }

    fn set_admin(env: Env, admin: Address) {
        Self::transfer_ownership(&env, admin);
    }

    fn admin(env: Env) -> Address {
        Self::owner(&env)
    }

    fn set_authorized(_env: Env, _id: Address, _authorize: bool) {
        unimplemented!()
    }

    /// This token has no authorization mechanism — there is no `set_authorized` and no
    /// `clawback`, so every holder is always authorized and `true` is the truthful answer for any
    /// address. Returning it instead of panicking lets a contract integrating this token treat it
    /// like a Stellar Asset Contract and check `authorized` before interacting, the same way it
    /// would for native XLM or any SAC without the `AUTH_REQUIRED` flag.
    fn authorized(_env: Env, _id: Address) -> bool {
        true
    }

    fn mint(env: Env, to: Address, amount: i128) {
        let owner = Self::owner(&env);
        owner.require_auth();

        Self::validate_amount(&env, amount);

        Self::receive_balance(&env, to.clone(), amount);

        MintWithAmountOnly { to, amount }.publish(&env);
    }

    fn clawback(_env: Env, _from: Address, _amount: i128) {
        unimplemented!()
    }
}

#[contractimpl]
impl InterchainTokenInterface for InterchainToken {
    fn token_id(env: &Env) -> BytesN<32> {
        storage::token_id(env)
    }

    fn is_minter(env: &Env, minter: Address) -> bool {
        storage::is_minter(env, minter)
    }

    fn mint_from(
        env: &Env,
        minter: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        minter.require_auth();

        ensure!(Self::is_minter(env, minter), ContractError::NotMinter);

        Self::validate_amount(env, amount);

        Self::receive_balance(env, to.clone(), amount);

        MintWithAmountOnly { to, amount }.publish(env);

        Ok(())
    }

    #[only_owner]
    fn add_minter(env: &Env, minter: Address) {
        assert_with_error!(
            env,
            !Self::is_minter(env, minter.clone()),
            ContractError::MinterAlreadyExists
        );

        storage::set_minter_status(env, minter.clone());

        MinterAddedEvent { minter }.emit(env);
    }

    #[only_owner]
    fn remove_minter(env: &Env, minter: Address) {
        assert_with_error!(
            env,
            Self::is_minter(env, minter.clone()),
            ContractError::NotMinter
        );

        storage::remove_minter_status(env, minter.clone());

        MinterRemovedEvent { minter }.emit(env);
    }
}

impl InterchainToken {
    fn validate_amount(env: &Env, amount: i128) {
        assert_with_error!(env, amount >= 0, ContractError::InvalidAmount);
    }

    fn read_allowance(env: &Env, from: Address, spender: Address) -> AllowanceValue {
        let key = AllowanceDataKey { from, spender };
        storage::try_allowance(env, key).map_or(
            AllowanceValue {
                amount: 0,
                expiration_ledger: 0,
            },
            |allowance| {
                if allowance.expiration_ledger < env.ledger().sequence() {
                    AllowanceValue {
                        amount: 0,
                        expiration_ledger: allowance.expiration_ledger,
                    }
                } else {
                    allowance
                }
            },
        )
    }

    fn write_allowance(
        env: &Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        let allowance = AllowanceValue {
            amount,
            expiration_ledger,
        };

        assert_with_error!(
            env,
            !(amount > 0 && expiration_ledger < env.ledger().sequence()),
            ContractError::InvalidExpirationLedger
        );

        let key = AllowanceDataKey { from, spender };
        storage::set_allowance(env, key.clone(), &allowance);

        if amount > 0 {
            let live_for = expiration_ledger
                .checked_sub(env.ledger().sequence())
                .unwrap();

            storage::extend_allowance_ttl(env, key, live_for, live_for);
        }
    }

    fn spend_allowance(env: &Env, from: Address, spender: Address, amount: i128) {
        let allowance = Self::read_allowance(env, from.clone(), spender.clone());

        assert_with_error!(
            env,
            allowance.amount >= amount,
            ContractError::InsufficientAllowance
        );

        if amount > 0 {
            Self::write_allowance(
                env,
                from,
                spender,
                allowance
                    .amount
                    .checked_sub(amount)
                    .expect("insufficient allowance"),
                allowance.expiration_ledger,
            );
        }
    }

    fn receive_balance(env: &Env, addr: Address, amount: i128) {
        let current_balance = storage::try_balance(env, addr.clone()).unwrap_or_default();

        storage::set_balance(env, addr, &(current_balance + amount));
    }

    fn spend_balance(env: &Env, addr: Address, amount: i128) {
        let balance = storage::try_balance(env, addr.clone()).unwrap_or_default();

        assert_with_error!(env, balance >= amount, ContractError::InsufficientBalance);

        storage::set_balance(env, addr, &(balance - amount));
    }

    fn write_metadata(env: &Env, metadata: TokenMetadata) {
        TokenUtils::new(env).metadata().set_metadata(&metadata);
    }
}
