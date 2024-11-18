use alloy_primitives::{FixedBytes, Uint, U256};
use alloy_sol_types::{sol, SolValue};
use axelar_soroban_std::ensure;
use soroban_sdk::{contracterror, Bytes, BytesN, Env, String};

use crate::types::{self, HubMessage, Message};
extern crate alloc;
use crate::abi::alloc::{string::String as StdString, vec};

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageError {
    InsufficientMessageLength = 0,
    InvalidMessageType = 1,
    AbiDecodeFailed = 2,
    InvalidAmount = 3,
}

sol! {
    enum MessageType {
        InterchainTransfer,
        DeployInterchainToken,
        DeployTokenManager, // note, this case is not supported by the ITS hub
        SendToHub,
        ReceiveFromHub
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
            Self::InterchainTransfer(types::InterchainTransfer {
                token_id,
                source_address,
                destination_address,
                amount,
                data,
            }) => InterchainTransfer {
                messageType: MessageType::InterchainTransfer.into(),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                sourceAddress: source_address.to_alloc_vec().into(),
                destinationAddress: destination_address.to_alloc_vec().into(),
                amount: amount.try_into().expect("failed to convert"),
                data: into_vec(data).into(),
            }
            .abi_encode_params(),
            Self::DeployInterchainToken(types::DeployInterchainToken {
                token_id,
                name,
                symbol,
                decimals,
                minter,
            }) => DeployInterchainToken {
                messageType: MessageType::DeployInterchainToken.into(),
                tokenId: FixedBytes::<32>::new(token_id.into()),
                name: to_std_string(name),
                symbol: to_std_string(symbol),
                decimals: decimals.try_into().expect("failed to convert"),
                minter: into_vec(minter).into(),
            }
            .abi_encode_params(),
        };
        Bytes::from_slice(env, &msg)
    }

    pub fn abi_decode(env: &Env, payload: &Bytes) -> Result<Self, MessageError> {
        ensure!(payload.len() >= 32, MessageError::InsufficientMessageLength);

        let payload = payload.to_alloc_vec();

        let message_type = MessageType::abi_decode(&payload[0..32], true)
            .map_err(|_| MessageError::InvalidMessageType)?;

        let message = match message_type {
            MessageType::InterchainTransfer => {
                let decoded = InterchainTransfer::abi_decode_params(&payload, true)
                    .map_err(|_| MessageError::AbiDecodeFailed)?;

                Self::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(env, &decoded.tokenId.into()),
                    source_address: Bytes::from_slice(env, decoded.sourceAddress.as_ref()),
                    destination_address: Bytes::from_slice(
                        env,
                        decoded.destinationAddress.as_ref(),
                    ),
                    amount: to_i128(decoded.amount)?,
                    data: from_vec(env, decoded.data.as_ref()),
                })
            }
            MessageType::DeployInterchainToken => {
                let decoded = DeployInterchainToken::abi_decode_params(&payload, true)
                    .map_err(|_| MessageError::AbiDecodeFailed)?;

                Self::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(env, &decoded.tokenId.into()),
                    name: String::from_str(env, &decoded.name),
                    symbol: String::from_str(env, &decoded.symbol),
                    decimals: decoded.decimals.into(),
                    minter: from_vec(env, decoded.minter.as_ref()),
                })
            }
            _ => return Err(MessageError::InvalidMessageType),
        };

        Ok(message)
    }
}

#[allow(dead_code)]
impl HubMessage {
    pub fn abi_encode(self, env: &Env) -> Bytes {
        let msg = match self {
            Self::SendToHub {
                destination_chain,
                message,
            } => SendToHub {
                messageType: MessageType::SendToHub.into(),
                destination_chain: to_std_string(destination_chain),
                message: message.abi_encode(env).to_alloc_vec().into(),
            }
            .abi_encode_params(),
            Self::ReceiveFromHub {
                source_chain,
                message,
            } => ReceiveFromHub {
                messageType: MessageType::ReceiveFromHub.into(),
                source_chain: to_std_string(source_chain),
                message: message.abi_encode(env).to_alloc_vec().into(),
            }
            .abi_encode_params(),
        };
        Bytes::from_slice(env, &msg)
    }

    pub fn abi_decode(env: &Env, payload: &Bytes) -> Result<Self, MessageError> {
        ensure!(payload.len() >= 32, MessageError::InsufficientMessageLength);

        let payload = payload.to_alloc_vec();

        let message_type = MessageType::abi_decode(&payload[0..32], true)
            .map_err(|_| MessageError::InvalidMessageType)?;

        let message = match message_type {
            MessageType::SendToHub => {
                let decoded = SendToHub::abi_decode_params(&payload, true)
                    .map_err(|_| MessageError::AbiDecodeFailed)?;

                Self::SendToHub {
                    destination_chain: String::from_str(env, &decoded.destination_chain),
                    message: Message::abi_decode(
                        env,
                        &Bytes::from_slice(env, decoded.message.as_ref()),
                    )?,
                }
            }
            MessageType::ReceiveFromHub => {
                let decoded = ReceiveFromHub::abi_decode_params(&payload, true)
                    .map_err(|_| MessageError::AbiDecodeFailed)?;

                Self::ReceiveFromHub {
                    source_chain: String::from_str(env, &decoded.source_chain),
                    message: Message::abi_decode(
                        env,
                        &Bytes::from_slice(env, decoded.message.as_ref()),
                    )?,
                }
            }
            _ => return Err(MessageError::InvalidMessageType),
        };

        Ok(message)
    }
}

fn to_std_string(soroban_string: String) -> StdString {
    let length = soroban_string.len() as usize;
    let mut bytes = vec![0u8; length];
    soroban_string.copy_into_slice(&mut bytes);
    StdString::from_utf8(bytes).expect("Invalid UTF-8 sequence")
}

fn to_i128(value: Uint<256, 4>) -> Result<i128, MessageError> {
    let slice = value.as_le_slice();

    let mut bytes_to_remove = [0; 16];
    let mut bytes_to_convert = [0; 16];
    bytes_to_remove.copy_from_slice(&slice[16..]);
    bytes_to_convert.copy_from_slice(&slice[..16]);

    ensure!(
        i128::from_le_bytes(bytes_to_remove) == 0,
        MessageError::InvalidAmount
    );

    Ok(i128::from_le_bytes(bytes_to_convert))
}

fn into_vec(value: Option<Bytes>) -> alloc::vec::Vec<u8> {
    value.map(|d| d.to_alloc_vec()).unwrap_or_default()
}

fn from_vec(env: &Env, value: &[u8]) -> Option<Bytes> {
    if value.is_empty() {
        None
    } else {
        Some(Bytes::from_slice(env, value))
    }
}

impl From<MessageType> for U256 {
    fn from(value: MessageType) -> Self {
        Self::from(value as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use soroban_sdk::{Bytes, BytesN, Env, String};
    use std::vec::Vec;

    const MAX_I128: i128 = i128::MAX as i128;

    fn bytes_from_hex(env: &Env, hex_string: &str) -> Bytes {
        let bytes_vec: Vec<u8> = hex::decode(hex_string).unwrap();
        Bytes::from_slice(env, &bytes_vec)
    }

    #[test]
    fn uint256_to_i128() {
        let amount: i128 = 9_876_543_210_123_456_789;
        let uint: Uint<256, 4> = amount.try_into().unwrap();

        assert_eq!(to_i128(uint).unwrap(), amount);
    }

    #[test]
    #[should_panic]
    fn i128_conversion_panics_dirty_bytes() {
        let amount: i128 = 9_876_543_210_123_456_789;
        let bytes = amount.to_le_bytes();
        assert_eq!(
            bytes,
            [21, 113, 52, 176, 184, 135, 16, 137, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        let bad_bytes = [21, 113, 52, 176, 184, 135, 16, 137, 0, 0, 0, 0, 0, 0, 0, 1];
        let _bad_uint = U256::from_le_bytes(bad_bytes);
    }

    #[test]
    fn interchain_transfer_encode_decode() {
        let env = Env::default();
        let remote_chain = String::from_str(&env, &"chain");

        let cases = vec![
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    source_address: bytes_from_hex(&env, "00"),
                    destination_address: bytes_from_hex(&env, "00"),
                    amount: 1u64.try_into().unwrap(),
                    data: None,
                })
                .into(),
            },
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: bytes_from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    destination_address: bytes_from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    amount: MAX_I128,
                    data: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    source_address: bytes_from_hex(&env, "00"),
                    destination_address: bytes_from_hex(&env, "00"),
                    amount: 1u64.try_into().unwrap(),
                    data: None,
                })
                .into(),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::InterchainTransfer(types::InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: bytes_from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    destination_address: bytes_from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    amount: MAX_I128,
                    data: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            },
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original| {
                hex::encode(
                    original
                        .clone()
                        .abi_encode(&env)
                        .to_buffer::<1024>()
                        .as_slice(),
                )
            })
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
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"t"),
                    symbol: String::from_str(&env, &"T"),
                    decimals: 0,
                    minter: None,
                })
                .into(),
            },
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, &"Test Token"),
                    symbol: String::from_str(&env, &"TST"),
                    decimals: 18,
                    minter: Some(bytes_from_hex(&env, "1234")),
                })
                .into(),
            },
            types::HubMessage::SendToHub {
                destination_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"Unicode Token 🪙"),
                    symbol: String::from_str(&env, &"UNI🔣"),
                    decimals: 255,
                    minter: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"t"),
                    symbol: String::from_str(&env, &"T"),
                    decimals: 0,
                    minter: None,
                })
                .into(),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, &"Test Token"),
                    symbol: String::from_str(&env, &"TST"),
                    decimals: 18,
                    minter: Some(bytes_from_hex(&env, "1234")),
                })
                .into(),
            },
            types::HubMessage::ReceiveFromHub {
                source_chain: remote_chain.clone(),
                message: types::Message::DeployInterchainToken(types::DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[0u8; 32]),
                    name: String::from_str(&env, &"Unicode Token 🪙"),
                    symbol: String::from_str(&env, &"UNI🔣"),
                    decimals: 255,
                    minter: Some(bytes_from_hex(&env, "abcd")),
                })
                .into(),
            },
        ];

        let encoded: Vec<_> = cases
            .iter()
            .map(|original| {
                hex::encode(
                    original
                        .clone()
                        .abi_encode(&env)
                        .to_buffer::<1024>()
                        .as_slice(),
                )
            })
            .collect();

        goldie::assert_json!(encoded);

        for original in cases {
            let encoded = original.clone().abi_encode(&env);
            let decoded = HubMessage::abi_decode(&env, &encoded);
            assert_eq!(original, decoded.unwrap());
        }
    }
}
