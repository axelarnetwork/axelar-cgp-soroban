mod utils;

use axelar_gateway::testutils::{generate_proof, get_approve_hash};
use axelar_gateway::types::Message as GatewayMessage;
use axelar_soroban_std::events;
use axelar_soroban_std::traits::BytesExt;
use interchain_token_service::event::{
    InterchainTransferReceivedEvent, InterchainTransferSentEvent,
};
use interchain_token_service::types::{HubMessage, InterchainTransfer, Message};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, BytesN, String};
use utils::{register_chains, setup_env, setup_gas_token, setup_its_token, HUB_CHAIN};

#[test]
fn interchain_transfer_send_succeeds() {
    let (env, client, _, _) = setup_env();

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 1000;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = String::from_str(&env, "ethereum");
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -4));
}

#[test]
#[should_panic(expected = "burn, Error(Contract, #9)")]
fn interchain_transfer_send_fails_on_insufficient_balance() {
    let (env, client, _, _) = setup_env();
    register_chains(&env, &client);

    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let amount = 1000;
    let token_id = setup_its_token(&env, &client, &sender, amount);

    let destination_chain = client.its_hub_chain_name();
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &(amount + 1),
        &data,
        &gas_token,
    );
}

#[test]
fn interchain_transfer_receive_succeeds() {
    let (env, client, gateway_client, signers) = setup_env();
    register_chains(&env, &client);

    let sender = Address::generate(&env).to_xdr(&env);
    let recipient = Address::generate(&env).to_xdr(&env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();

    let amount = 1000;
    let deployer = Address::generate(&env);
    let token_id = setup_its_token(&env, &client, &deployer, amount);

    let data = Some(Bytes::from_hex(&env, "abcd"));

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: sender,
            destination_address: recipient,
            amount,
            data,
        }),
    };
    let payload = msg.abi_encode(&env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let message_id = String::from_str(&env, "test");

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    gateway_client.approve_messages(&messages, &proof);

    client.execute(&source_chain, &message_id, &source_address, &payload);

    goldie::assert!(events::fmt_last_emitted_event::<
        InterchainTransferReceivedEvent,
    >(&env));
}
