mod utils;
use axelar_gateway::testutils::{generate_proof, get_approve_hash};
use axelar_gateway::types::Message as GatewayMessage;
use axelar_soroban_std::traits::BytesExt;
use axelar_soroban_std::{assert_emitted_event, assert_last_emitted_event, assert_ok};
use interchain_token_service::types::{HubMessage, InterchainTransfer, Message};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, BytesN, String, Symbol};
use utils::{register_chains, setup_env, setup_gas_token, HUB_CHAIN};

#[test]
fn interchain_transfer_send() {
    let (env, client, gateway_client, _) = setup_env();
    register_chains(&env, &client);
    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);

    let token_id = BytesN::from_array(&env, &[255u8; 32]);
    let destination_chain = String::from_str(&env, HUB_CHAIN);
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let amount = i128::MAX;
    let data = Some(Bytes::from_hex(&env, "abcd"));

    let msg = Message::InterchainTransfer(InterchainTransfer {
        token_id: token_id.clone(),
        source_address: sender.clone().to_xdr(&env),
        destination_address: destination_address.clone(),
        amount,
        data: data.clone(),
    });

    let expected_payload = assert_ok!(HubMessage::SendToHub {
        destination_chain: String::from_str(&env, HUB_CHAIN),
        message: msg
    }
    .abi_encode(&env));

    let expected_destination_chain = client.its_hub_chain_name();
    let expected_destination_address = client.its_hub_address();
    let expected_payload_hash: BytesN<32> = env.crypto().keccak256(&expected_payload).into();

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    assert_last_emitted_event(
        &env,
        &gateway_client.address,
        (
            Symbol::new(&env, "contract_called"),
            client.address,
            expected_destination_chain,
            expected_destination_address,
            expected_payload_hash,
        ),
        expected_payload,
    );
}

#[test]
fn interchain_transfer_receive() {
    let (env, client, gateway_client, signers) = setup_env();
    register_chains(&env, &client);

    let sender = Address::generate(&env).to_xdr(&env);
    let recipient = Address::generate(&env).to_xdr(&env);
    let source_chain = client.its_hub_chain_name();
    let source_address = Address::generate(&env).to_string();

    let token_id = BytesN::from_array(&env, &[255u8; 32]);
    let amount = i128::MAX;
    let data = Some(Bytes::from_hex(&env, "abcd"));

    let msg = HubMessage::ReceiveFromHub {
        source_chain: String::from_str(&env, HUB_CHAIN),
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id: token_id.clone(),
            source_address: sender.clone(),
            destination_address: recipient.clone(),
            amount,
            data: data.clone(),
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

    assert_emitted_event(
        &env,
        -1,
        &client.address,
        (
            Symbol::new(&env, "interchain_transfer_received"),
            String::from_str(&env, HUB_CHAIN),
            token_id,
            sender,
            recipient,
            amount,
        ),
        (data,),
    );
}
