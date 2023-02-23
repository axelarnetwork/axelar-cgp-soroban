/*
 * Axelar ETH utils
 * Credit: https://github.com/AndWa
 */
use ethabi::decode;
use ethabi::encode;
use ethabi::ethereum_types::H256;
use ethabi::Address;
use ethabi::ParamType;
use ethabi::Token;
use sha3::{Digest, Keccak256};
use soroban_sdk::{Bytes, Vec};

extern crate alloc;

/// It takes a slice of bytes and returns a 32-byte hash
/// Compute the Keccak-256 hash of input bytes.
///
/// Panics if the computed hash is not the expected length (32 bytes).
///
/// Arguments:
///
/// * `bytes`: The bytes to hash.
///
/// Returns:
///
/// A 32 byte array
pub fn keccak256<S>(bytes: S) -> [u8; 32]
where
    S: AsRef<[u8]>,
{
    let hash = Keccak256::digest(bytes.as_ref());
    let hash: [u8; 32] = hash
        .as_slice()
        .try_into()
        .expect("hash is not the correct length");
    hash
}

/// It takes a byte array and a list of expected output types, and returns a list of tokens
///
/// Arguments:
///
/// * `data`: The data to decode.
/// * `expected_output_types`: The types of the values that are expected to be returned.
///
/// Returns:
///
/// A vector of tokens.
// pub fn abi_decode(data: &[u8], expected_output_types: &[ParamType]) -> Result<Vec<Token>, String> {
//     match decode(expected_output_types, data) {
//         Ok(tokens) => Ok(tokens),
//         Err(e) => Err(format!("Error decoding ABI-encoded data: {:?}", e)),
//     }
// }

/// It takes a vector of tokens and returns a vector of bytes
///
/// Arguments:
///
/// * `tokens`: A vector of tokens to encode.
///
/// Returns:
///
/// A vector of bytes.
// pub fn abi_encode(tokens: Vec<Token>) -> Bytes {
//     encode(&tokens);
// }

/// It takes a string, removes the first two characters, and then converts the remaining string into a
/// vector of bytes
///
/// Arguments:
///
/// * `payload`: The payload of the transaction.
///
/// Returns:
///
/// A vector of bytes
pub fn clean_payload(payload: Bytes) -> alloc::vec::Vec<u8>  {
    let mut payload_copy = payload.clone();
    payload_copy.remove(0);
    payload_copy.remove(1);
    let mut clean_payload = alloc::vec![];
    for element in payload_copy {
        clean_payload.push(element)
    }
    return clean_payload;
    
}

// // Converts Token into Bytes
// pub fn into_bytes(token: Token) -> Option<Bytes> {
//     match token {
//         Bytes(bytes) => Some(bytes),
//         _ => None,
//     }
// }
