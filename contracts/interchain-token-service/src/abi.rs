use axelar_soroban_std::ensure;
use soroban_sdk::{Bytes, BytesN, Env, String, contracterror};
use alloy_primitives::{FixedBytes, Uint, U256};
use alloy_sol_types::{sol, SolValue};

use crate::abi::alloc::string::ToString;
use crate::types::{self, Message, HubMessage};
extern crate alloc;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageError {
    InsufficientMessageLength = 0,
    InvalidMessageType = 1,
    AbiDecodeFailed = 2,
    InvalidTokenManagerType = 3,
    InvalidAmount = 4,
}

sol! {
    enum MessageType {
        InterchainTransfer,
        DeployInterchainToken,
        DeployTokenManager,
        SendToHub,
        ReceiveFromHub,
    }

    struct InterchainTransfer {
        uint256 messageType;
        bytes32 tokenId;
        bytes sourceAddress;
        bytes destinationAddress;
        uint256 amount;
        bytes data;
    }

    struct DeployInterchainToken {
        uint256 messageType;
        bytes32 tokenId;
        string name;
        string symbol;
        uint8 decimals;
        bytes minter;
    }

    struct DeployTokenManager {
        uint256 messageType;
        bytes32 tokenId;
        uint256 tokenManagerType;
        bytes params;
    }

    struct SendToHub {
        uint256 messageType;
        string destination_chain;
        bytes message;
    }

    struct ReceiveFromHub {
        uint256 messageType;
        string source_chain;
        bytes message;
    }
}

impl Message {
    pub fn abi_encode(self, env: &Env) -> Bytes {
        let msg = match self {
            Message::InterchainTransfer(types::InterchainTransfer {
                token_id,
                source_address,
                destination_address,
                amount,
                data,
            }) => InterchainTransfer {
                messageType: U256::from(types::MessageType::InterchainTransfer as u32),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                sourceAddress: source_address.to_alloc_vec().into(),
                destinationAddress: destination_address.to_alloc_vec().into(),
                amount: amount.try_into().unwrap_or_default(),
                data: data.map(|d| d.to_alloc_vec()).unwrap_or_default().into(),
            }
            .abi_encode_params(),
            Message::DeployInterchainToken(types::DeployInterchainToken {
                token_id,
                name,
                symbol,
                decimals,
                minter,
            }) => DeployInterchainToken {
                messageType: U256::from(types::MessageType::DeployInterchainToken as u32),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                name: name.to_string(),
                symbol: symbol.to_string(),
                decimals: decimals.try_into().unwrap_or_default(),
                minter: minter.map(|m| m.to_alloc_vec()).unwrap_or_default().into(), 
            }
            .abi_encode_params(),
            Message::DeployTokenManager(types::DeployTokenManager {
                token_id,
                token_manager_type,
                params,
            }) => DeployTokenManager {
                messageType: U256::from(types::MessageType::DeployTokenManager as u32),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                tokenManagerType: U256::from(token_manager_type as u32),
                params: params.to_alloc_vec().into(),
            }
            .abi_encode_params(),
        };
        Bytes::from_slice(&env, &msg)    
    }

    pub fn abi_decode(env: &Env, payload: &Bytes) -> Result<Self, MessageError> {
        ensure!(
            payload.len() >= 32,
            MessageError::InsufficientMessageLength
        );

        let binding = payload.to_alloc_vec();
        let payload_array = binding.as_slice();

        let message_type = match MessageType::abi_decode(&payload_array[0..32], true) {
            Ok(value) => value,
            Err(_) => return Err(MessageError::InvalidMessageType),
        };

        let message = match message_type {
            MessageType::InterchainTransfer => {
                let decoded = match InterchainTransfer::abi_decode_params(&payload_array, true) {
                    Ok(value) => value,
                    Err(_) => return Err(MessageError::AbiDecodeFailed),
                };

                types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &decoded.tokenId.into()),
                    source_address: Bytes::from_slice(&env, &decoded.sourceAddress.to_vec()),
                    destination_address: Bytes::from_slice(&env, &decoded.destinationAddress.to_vec()),
                    amount: convert_to_i128(decoded.amount)?,
                    data: match decoded.data.len() {
                        0 => None,
                        _ => Some(Bytes::from_slice(&env, &decoded.data.to_vec())),
                    }
                })
       
            },
            MessageType::DeployInterchainToken => {
                let decoded = match DeployInterchainToken::abi_decode_params(&payload_array, true) {
                    Ok(value) => value,
                    Err(_) => return Err(MessageError::AbiDecodeFailed),
                };

                types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &decoded.tokenId.into()),
                    name: String::from_str(&env, &decoded.name),
                    symbol: String::from_str(&env, &decoded.symbol),
                    decimals: decoded.decimals.into(),
                    minter: match decoded.minter.len() {
                        0 => None,
                        _ => Some(Bytes::from_slice(&env, &decoded.minter.to_vec()))
                    },
                })
            },
            MessageType::DeployTokenManager => {
                let decoded = match DeployTokenManager::abi_decode_params(&payload_array, true) {
                    Ok(value) => value,
                    Err(_) => return Err(MessageError::AbiDecodeFailed),
                };

                types::Message::DeployTokenManager(types::DeployTokenManager {
                    token_id: BytesN::from_array(&env, &decoded.tokenId.into()),
                    token_manager_type: 
                        if let Ok(token_manager_u32) = u32::try_from(decoded.tokenManagerType) {
                            match get_token_manager_type(token_manager_u32) {
                                Some(value) => value,
                                None => return Err(MessageError::InvalidTokenManagerType),
                            }
                        } else {
                            return Err(MessageError::InvalidTokenManagerType)
                        },
                    params: Bytes::from_slice(&env, &decoded.params.to_vec()),
                })
            },
            _ => return Err(MessageError::InvalidMessageType),
        };

        Ok(message)
    }
}

impl HubMessage {
    pub fn abi_encode(self, env: &Env) -> Bytes {
        let msg = match self {
            HubMessage::SendToHub(types::SendToHub {
                destination_chain,
                message,
            }) => SendToHub {
                messageType: U256::from(types::MessageType::SendToHub as u32),
                destination_chain: destination_chain.to_string(),
                message: message.abi_encode(&env).to_alloc_vec().into(),
            }
            .abi_encode_params(),
            HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain,
                message,
            }) => ReceiveFromHub {
                messageType: U256::from(types::MessageType::ReceiveFromHub as u32),
                source_chain: source_chain.to_string(),
                message: message.abi_encode(&env).to_alloc_vec().into(),
            }
            .abi_encode_params(),
        };
        Bytes::from_slice(&env, &msg)
    }

    pub fn abi_decode(env: &Env, payload: &Bytes) -> Result<Self, MessageError> {
        ensure!(
            payload.len() >= 32,
            MessageError::InsufficientMessageLength
        );

        let binding = payload.to_alloc_vec();
        let payload_array = binding.as_slice();

        let message_type = match MessageType::abi_decode(&payload_array[0..32], true) {
            Ok(value) => value,
            Err(_) => return Err(MessageError::InvalidMessageType),
        };

        let message = match message_type {
            MessageType::SendToHub => {
                let decoded = match SendToHub::abi_decode_params(&payload_array, true) {
                    Ok(value) => value,
                    Err(_) => return Err(MessageError::AbiDecodeFailed),
                };

                types::HubMessage::SendToHub(types::SendToHub {
                    destination_chain: String::from_str(&env, &decoded.destination_chain),
                    message: Message::abi_decode(&env, &Bytes::from_slice(&env, &decoded.message.to_vec()))?,
                })
            },
            MessageType::ReceiveFromHub => {
                let decoded = match ReceiveFromHub::abi_decode_params(&payload_array, true) {
                    Ok(value) => value,
                    Err(_) => return Err(MessageError::AbiDecodeFailed),
                };

                types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                    source_chain: String::from_str(&env, &decoded.source_chain),
                    message: Message::abi_decode(&env, &Bytes::from_slice(&env, &decoded.message.to_vec()))?,
                })
            },
            _ => return Err(MessageError::InvalidMessageType),
        };

        Ok(message)
    }
}

fn get_token_manager_type(decoded_value: u32) -> Option<types::TokenManagerType> {
    match decoded_value {
        0 => Some(types::TokenManagerType::NativeInterchainToken),
        1 => Some(types::TokenManagerType::MintBurnFrom),
        2 => Some(types::TokenManagerType::LockUnlock),
        3 => Some(types::TokenManagerType::LockUnlockFee),
        4 => Some(types::TokenManagerType::MintBurn),
        _ => None,
    }
}

fn convert_to_i128(decoded_value: Uint<256, 4>) -> Result<i128, MessageError> {
    let slice = decoded_value.as_le_slice();

    if !slice[16..].iter().all(|&b| b == 0) {
        return Err(MessageError::InvalidAmount);
    }

    let mut truncated = [0u8; 16];
    truncated.copy_from_slice(&slice[..16]);

    Ok(i128::from_le_bytes(truncated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use soroban_sdk::{Bytes, BytesN, Env, String, TryFromVal};
    use std::vec::Vec;

    const MAX_I128: i128 = 170141183460469231731687303715884105727 as i128;

    fn bytes_from_hex(env: &Env, hex_string: &str) -> Bytes {
        let bytes_vec: Vec<u8> = hex::decode(hex_string).unwrap();
        Bytes::from_slice(env, &bytes_vec)
    }

    #[test]
    fn abi_encode_decode() {
        let env = Env::default();

        let its_messages = vec![
            types::Message::InterchainTransfer(types::InterchainTransfer {
                token_id: BytesN::from_array(&env, &[1; 32]),
                source_address: Bytes::from_slice(&env, &[2; 32]),
                destination_address: Bytes::from_slice(&env, &[3; 32]),
                amount: 9_876_543_210_123_456_789,
                data: Some(Bytes::from_slice(&env, &[4; 32])),
            }),
            types::Message::DeployInterchainToken(types::DeployInterchainToken { 
                token_id: BytesN::from_array(&env, &[1; 32]),
                name: String::from_str(&env, "some_token"), 
                symbol: String::from_str(&env, "TKN"), 
                decimals: 18, 
                minter: Some(Bytes::from_slice(&env, &[1; 32]))
            }),
            types::Message::DeployTokenManager(types::DeployTokenManager { 
                token_id: BytesN::from_array(&env, &[1; 32]),
                token_manager_type: types::TokenManagerType::NativeInterchainToken, 
                params: Bytes::try_from_val(&env, &"1234").unwrap()
            }),
        ];

        for msg in its_messages {
            let encoded = msg.clone().abi_encode(&env);
            let decoded = Message::abi_decode(&env, &encoded);
            assert_eq!(msg, decoded.unwrap());
        }

        let hub_messages = vec![
            types::HubMessage::SendToHub(types::SendToHub { 
                destination_chain: String::from_str(&env, "some_chain"), 
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken { 
                    token_id: BytesN::from_array(&env, &[1; 32]),
                    name: String::from_str(&env, "some_token"), 
                    symbol: String::from_str(&env, "TKN"), 
                    decimals: 18, 
                    minter: Some(Bytes::from_slice(&env, &[1; 32]))
                }),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub { 
                source_chain: String::from_str(&env, "some_chain"),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[1; 32]),
                    source_address: Bytes::from_slice(&env, &[2; 32]),
                    destination_address: Bytes::from_slice(&env, &[3; 32]),
                    amount: 9_876_543_210_123_456_789,
                    data: Some(Bytes::from_slice(&env, &[4; 32])),
                }), 
            }),
        ];

        for msg in hub_messages {
            let encoded = msg.clone().abi_encode(&env);
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(msg, decoded.unwrap());
        }
    }

    #[test]
    fn uint256_to_i128() {
        let amount: i128 = 9_876_543_210_123_456_789;
        let uint: Uint<256, 4> = amount.try_into().unwrap();

        assert_eq!(convert_to_i128(uint).unwrap(), amount);
    }

    #[test]
    #[should_panic]
    fn i128_conversion_panics_dirty_bytes() {
        let amount: i128 = 9_876_543_210_123_456_789;
        let bytes = amount.to_le_bytes();
        assert_eq!(bytes, [21, 113, 52, 176, 184, 135, 16, 137, 0, 0, 0, 0, 0, 0, 0, 0]);

        let bad_bytes = [21, 113, 52, 176, 184, 135, 16, 137, 0, 0, 0, 0, 0, 0, 0, 1];
        let bad_uint = U256::from_le_bytes(bad_bytes);

        convert_to_i128(bad_uint).unwrap();
    }

    #[test]
    fn interchain_transfer_encode_decode() {
        let env = Env::default();
        let remote_chain = String::from_str(&env, &"chain");

        let cases = vec![
            types::HubMessage::SendToHub(types::SendToHub { 
                destination_chain: remote_chain.clone(), 
                message: types::Message::InterchainTransfer(types::InterchainTransfer { 
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    source_address: bytes_from_hex(&env, "00"),
                    destination_address: bytes_from_hex(&env, "00"),
                    amount: 1u64.try_into().unwrap(), 
                    data: None, 
                })
                .into()
            }),
            types::HubMessage::SendToHub(types::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
                    destination_address: bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
                    amount: MAX_I128,
                    data: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    source_address: bytes_from_hex(&env, "00"),
                    destination_address: bytes_from_hex(&env, "00"),
                    amount: 1u64.try_into().unwrap(),
                    data: None,
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
                    destination_address: bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
                    amount: MAX_I128,
                    data: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            }),
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original|
                hex::encode(
                    original.clone()
                        .abi_encode(&env)
                        .to_buffer::<1024>()
                        .as_slice()
                )
            )
            .collect();

        goldie::assert_json!(encoded);

        for original in cases {
            let encoded = original.clone().abi_encode(&env);
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(original, decoded.unwrap());
        }
    }

    #[test]
    fn deploy_interchain_token_encode_decode() {
        let env = Env::default();
        let remote_chain = String::from_str(&env, &"chain");

        let cases = vec![
            types::HubMessage::SendToHub(types::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"t"),
                    symbol: String::from_str(&env, &"T"),
                    decimals: 0,
                    minter: None,
                })
                .into(),
            }),
            types::HubMessage::SendToHub(types::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, &"Test Token"),
                    symbol: String::from_str(&env, &"TST"),
                    decimals: 18,
                    minter: Some(bytes_from_hex(&env, "1234")),
                })
                .into(),
            }),
            types::HubMessage::SendToHub(types::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"Unicode Token 🪙"),
                    symbol: String::from_str(&env, &"UNI🔣"),
                    decimals: 255,
                    minter: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"t"),
                    symbol: String::from_str(&env, &"T"),
                    decimals: 0,
                    minter: None,
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, &"Test Token"),
                    symbol: String::from_str(&env, &"TST"),
                    decimals: 18,
                    minter: Some(bytes_from_hex(&env, "1234")),
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"Unicode Token 🪙"),
                    symbol: String::from_str(&env, &"UNI🔣"),
                    decimals: 255,
                    minter: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            }),
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original|
                hex::encode(
                    original.clone()
                        .abi_encode(&env)
                        .to_buffer::<1024>()
                        .as_slice()
                )
            )
            .collect();

        goldie::assert_json!(encoded);

        for original in cases {
            let encoded = original.clone().abi_encode(&env);
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(original, decoded.unwrap());
        }
    }

    #[test]
    fn deploy_token_manager_encode_decode() {
        let env = Env::default();
        let remote_chain = String::from_str(&env, &"chain");

        let cases = vec![
            types::HubMessage::SendToHub(types::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployTokenManager(types::DeployTokenManager {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    token_manager_type: types::TokenManagerType::NativeInterchainToken,
                    params: bytes_from_hex(&env, "00"),
                })
                .into(),
            }),
            types::HubMessage::SendToHub(types::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployTokenManager(types::DeployTokenManager {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    token_manager_type: types::TokenManagerType::MintBurn,
                    params: bytes_from_hex(&env, "1234"),
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployTokenManager(types::DeployTokenManager {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    token_manager_type: types::TokenManagerType::NativeInterchainToken,
                    params: bytes_from_hex(&env, "00"),
                })
                .into(),
            }),
            types::HubMessage::ReceiveFromHub(types::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployTokenManager(types::DeployTokenManager {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    token_manager_type: types::TokenManagerType::MintBurn,
                    params: bytes_from_hex(&env, "1234"),
                })
                .into(),
            }),
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original|
                hex::encode(
                    original.clone()
                        .abi_encode(&env)
                        .to_buffer::<1024>()
                        .as_slice()
                )
            )
            .collect();

        goldie::assert_json!(encoded);

        for original in cases {
            let encoded = original.clone().abi_encode(&env);
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(original, decoded.unwrap());
        }
    }
}
