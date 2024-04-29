// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{settings::KoreSettings,error::NodeError};
use kore_base::{KeyDerivator, crypto::{Ed25519KeyPair, KeyGenerator, KeyMaterial, KeyPair, Secp256k1KeyPair, KeyPairType}};

use pkcs8::{pkcs5, Document, EncryptedPrivateKeyInfo, PrivateKeyInfo};
use hex_literal::hex;

use std::fs;

/// Get node key pair.
/// If the key pair does not exist, it is generated and encrypted with the provided password.
/// If the key pair exists, it is decrypted with the provided password.
/// The key pair is stored in the keys directory.
/// 
/// # Arguments
/// 
/// * `settings` - Kore settings
/// * `password` - Password to encrypt/decrypt the key pair
/// 
/// # Returns
/// 
/// * `Result<KeyPair, NodeError>` - Key pair
/// 
/// # Errors
/// 
/// * `NodeError::InternalApi` - Internal API error
/// * `NodeError::Keys` - Keys error
/// 
pub fn node_key_pair(settings: &KoreSettings, password: &str) -> Result<KeyPair, NodeError> {
    if fs::metadata(&settings.keys_path).is_err() {
        fs::create_dir_all(&settings.keys_path).map_err(|error| {
            NodeError::InternalApi(format!("Error creating keys directory: {}", error))
        })?;
    }
    let path = format!("{}/node_private.der", &settings.keys_path);
    match fs::metadata(&path) {
        Ok(_) => {
            let document = Document::read_der_file(path).map_err(|error| {
                NodeError::Keys(format!("Error reading node private key: {}", error))
            })?;
            let enc_pk = EncryptedPrivateKeyInfo::try_from(document.as_bytes())
                .map_err(|error| {
                    NodeError::Keys(format!(
                        "Error reading node private key: {}",
                        error
                    ))
            })?;
            let dec_pk = enc_pk.decrypt(password).map_err(|error| {
                NodeError::Keys(format!(
                    "Error decrypting node private key: {}",
                    error
                ))
            })?;
            let key_type = match & settings.settings.node.key_derivator {
                KeyDerivator::Ed25519 => KeyPairType::Ed25519,
                KeyDerivator::Secp256k1 => KeyPairType::Secp256k1,
            };
            let key_pair = KeyPair::from_secret_der(key_type, dec_pk.as_bytes())
                .map_err(|error| {
                    NodeError::Keys(format!(
                        "Error creating key pair from secret der: {}",
                        error
                    ))
                })?;
            Ok(key_pair)
        }
        Err(_) => {
            let key_pair = match &settings.settings.node.key_derivator {
                KeyDerivator::Ed25519 => {
                    KeyPair::Ed25519(Ed25519KeyPair::new())
                }
                KeyDerivator::Secp256k1 => {
                    KeyPair::Secp256k1(Secp256k1KeyPair::new())
                }
            };
            let der = key_pair.to_secret_der()
                .map_err(|error| {
                    NodeError::Keys(format!(
                        "Error getting secret der: {}",
                        error
                    ))
                })?;
            let pk = PrivateKeyInfo::try_from(der.as_slice()).map_err(|error| {
                NodeError::Keys(format!(
                    "Error creating private key info: {}",
                    error
                ))
            })?;
            let params = pkcs5::pbes2::Parameters::pbkdf2_sha256_aes256cbc(
                2048,
                &hex!("79d982e70df91a88"),
                &hex!("b2d02d78b2efd9dff694cf8e0af40925"),
            ).map_err(|error| {
                NodeError::Keys(format!(
                    "Error creating pkcs5 parameters: {}",
                    error
                ))
            })?;
            let enc_pk = pk.encrypt_with_params(params, password)
                .map_err(|_| NodeError::Keys("Error encrypting private key".to_owned()))?;
            enc_pk.write_der_file(path).map_err(|error| {
                NodeError::Keys(format!("Error writing node private key: {}", error))
            })?;
            Ok(key_pair)
        }
    } 
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::settings::KoreSettings;
    use kore_base::crypto::KeyMaterial;
    use std::fs;


    #[test]
    fn test_node_key_pair() {
        let mut settings = KoreSettings::default();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("keys");
        settings.keys_path = path.to_str().unwrap().to_owned();
        let key_pair = node_key_pair(&settings, "password").unwrap();
        let key_pair2 = node_key_pair(&settings, "password").unwrap();
        assert_eq!(key_pair.to_bytes(), key_pair2.to_bytes());
        fs::remove_dir_all(&settings.keys_path).unwrap();
    }
}