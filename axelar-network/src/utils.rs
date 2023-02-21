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
use soroban_sdk::{Bytes};

/// It takes a hash and a signature, and returns the address that signed the hash
///
/// Arguments:
///
/// * `hash`: The hash of the message to be signed.
/// * `signature`: The signature to verify.
///
/// Returns:
///
/// The address of the signer.
pub fn ecrecover(hash: H256, signature: &[u8]) -> Result<Address, ()> {
    assert_eq!(signature.len(), 65);

    let hash = secp256k1::Message::parse_slice(hash.as_bytes()).unwrap();
    let v = signature[64];
    let signature = secp256k1::Signature::parse_slice(&signature[0..64]).unwrap();
    let bit = match v {
        0..=26 => v,
        _ => v - 27,
    };

    if let Ok(recovery_id) = secp256k1::RecoveryId::parse(bit) {
        if let Ok(public_key) = secp256k1::recover(&hash, &signature, &recovery_id) {
            // recover returns a 65-byte key, but addresses come from the raw 64-byte key
            let r = sha3::Keccak256::digest(&public_key.serialize()[1..]);
            return Ok(Address::from_slice(&r[12..]));
        }
    }

    Err(())
}

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
pub fn abi_decode(data: &[u8], expected_output_types: &[ParamType]) -> Result<Vec<Token>, String> {
    match decode(expected_output_types, data) {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(format!("Error decoding ABI-encoded data: {:?}", e)),
    }
}

/// It takes a vector of tokens and returns a vector of bytes
///
/// Arguments:
///
/// * `tokens`: A vector of tokens to encode.
///
/// Returns:
///
/// A vector of bytes.
pub fn abi_encode(tokens: Vec<Token>) -> Vec<u8> {
    encode(&tokens)
}

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
pub fn clean_payload(payload: Bytes) -> Vec<u8> {
    payload.remove(0);
    payload.remove(1);
    let clean_payload: Vec<u8> = Vec::new();
    for element in payload {
        clean_payload.push(element)
    }
    return clean_payload;
    

}

/// It takes a string, removes the first two characters, and then converts the remaining string into a
/// 256-bit hash
///
/// Arguments:
///
/// * `payload`: The payload of the transaction.
///
/// Returns:
///
/// A H256 hash
pub fn to_h256(payload: String) -> H256 {
    let clean_payload = &payload[2..payload.len()];
    <H256 as std::str::FromStr>::from_str(clean_payload).unwrap()
}

/// It takes a 32-byte array and returns a hex string
///
/// Arguments:
///
/// * `payload`: [u8; 32] - The payload is a 32 byte array.
///
/// Returns:
///
/// A string
pub fn to_eth_hex_string(payload: [u8; 32]) -> String {
    format!("0x{}", hex::encode(payload))
}