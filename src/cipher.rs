use crate::error::MyError;
use openssl::rand::rand_bytes;
use openssl::symm::{self, Cipher};
use std::env;

const IV_LENGTH: usize = 16;

pub async fn encrypt(input: &str) -> Result<String, MyError> {
    let cipher = Cipher::aes_128_cbc();
    let aes_key = env::var("AES_KEY").expect("AES_KEY");
    let aes_key = hex::decode(aes_key).expect("Failed to decode AES_KEY.");

    let mut iv = [0; IV_LENGTH];
    rand_bytes(&mut iv).map_err(|e| MyError::OpensslError(e))?;

    let ciphertext = symm::encrypt(cipher, &aes_key, Some(&iv), input.as_bytes())
        .map_err(|e| MyError::OpensslError(e))?;

    let mut raw = iv.to_vec();
    raw.extend(ciphertext);
    let output = openssl::base64::encode_block(&raw);
    Ok(output)
}

pub async fn decrypt(input: &str) -> Result<String, MyError> {
    let cipher = Cipher::aes_128_cbc();
    let aes_key = env::var("AES_KEY").expect("AES_KEY");
    let aes_key = hex::decode(aes_key).expect("Failed to decode AES_KEY.");

    let raw = openssl::base64::decode_block(input).map_err(|e| MyError::OpensslError(e))?;
    let iv = &raw[..IV_LENGTH];
    let cipthertext = &raw[IV_LENGTH..];

    let output = symm::decrypt(cipher, &aes_key, Some(&iv), &cipthertext)
        .map_err(|e| MyError::OpensslError(e))?;
    let output = String::from_utf8(output).map_err(|e| MyError::FromUtf8Error(e))?;
    Ok(output)
}
