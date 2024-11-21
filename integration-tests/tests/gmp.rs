use axelar_gateway::testutils::{generate_proof, get_approve_hash, setup_gateway};
use axelar_gateway::types::Message;
use axelar_gateway::AxelarGatewayClient;
use axelar_soroban_std::assert_last_emitted_event;
use soroban_sdk::{contract, contractimpl, log, Bytes, Symbol};
use soroban_sdk::{testutils::BytesN as _, vec, Address, BytesN, Env, String};

use axelar_gateway::executable::AxelarExecutableInterface;

#[contract]
pub struct AxelarApp;

#[contractimpl]
impl AxelarExecutableInterface for AxelarApp {
    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&"gateway").unwrap()
    }

    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) {
        let _ = Self::validate_message(&env, &source_chain, &message_id, &source_address, &payload);

        env.events()
            .publish((Symbol::new(&env, "executed"),), (payload,));
    }
}

#[contractimpl]
impl AxelarApp {
    pub fn initialize(env: Env, gateway: Address) {
        env.storage().instance().set(&"initialized", &true);

        env.storage().instance().set(&"gateway", &gateway);
    }

    pub fn send(env: Env, destination_chain: String, destination_address: String, message: Bytes) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }
}

fn setup_app<'a>(env: &Env, gateway: &Address) -> AxelarAppClient<'a> {
    let contract_id = env.register_contract(None, AxelarApp);
    let client = AxelarAppClient::new(env, &contract_id);
    client.initialize(gateway);

    client
}

#[test]
/// This is an integration test for sending a cross-chain message from source to destination Axelar powered app
fn test_gmp() {
    let env = Env::default(); // Auth doesn't need to be mocked for this integration test

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, "source");
    let (_, source_gateway_client) = setup_gateway(&env, 0, 5);
    let source_app = setup_app(&env, &source_gateway_client.address);

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, "destination");
    let (signers, destination_gateway_client) = setup_gateway(&env, 0, 5);
    let destination_gateway_id = destination_gateway_client.address.clone();
    let destination_app = setup_app(&env, &destination_gateway_id);
    let destination_app_id = destination_app.address.clone();

    // Set cross-chain message params
    let source_address = source_app.address.to_string();
    let destination_address = destination_app_id.to_string();
    let payload: Bytes = BytesN::<20>::random(&env).into();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    // Initiate cross-chain contract call
    log!(env, "Sending message from source to destination");

    source_app.send(&destination_chain, &destination_address, &payload);

    // Axelar hub confirms the contract call, i.e Axelar verifiers verify/vote on the emitted event
    let message_id = String::from_str(&env, "test");

    log!(env, "Confirming message from source Axelar gateway");

    assert_last_emitted_event(
        &env,
        &source_gateway_client.address,
        (
            Symbol::new(&env, "contract_called"),
            source_app.address.clone(),
            destination_chain,
            destination_address,
            payload_hash.clone(),
        ),
        payload.clone(),
    );

    // Axelar hub signs the message approval
    log!(env, "Signing message approval for destination");

    let messages = vec![
        &env,
        Message {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: destination_app_id.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    // Submit the signed batch to the destination Axelar gateway
    log!(
        env,
        "Submitting signed message approval to destination Axelar gateway"
    );

    destination_gateway_client.approve_messages(&messages, &proof);

    // Execute the app
    log!(env, "Executing message on destination app");

    destination_app.execute(&source_chain, &message_id, &source_address, &payload);

    assert_last_emitted_event(
        &env,
        &destination_app_id,
        (Symbol::new(&env, "executed"),),
        (payload,),
    );
}
