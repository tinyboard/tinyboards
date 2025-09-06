use crate::error::TinyBoardsError;
use openssl::{pkey::PKey, rsa::Rsa};

pub struct ActorKeypair {
    pub private_key: String,
    pub public_key: String,
}

pub fn generate_actor_keypair() -> Result<ActorKeypair, TinyBoardsError> {
    let rsa = Rsa::generate(2048).map_err(|e| {
        TinyBoardsError::from_error_message(e, 500, "Failed to generate RSA keypair")
    })?;
    let pkey = PKey::from_rsa(rsa).map_err(|e| {
        TinyBoardsError::from_error_message(e, 500, "Failed to create PKey from RSA")
    })?;
    
    let private_key = pkey.private_key_to_pem_pkcs8().map_err(|e| {
        TinyBoardsError::from_error_message(e, 500, "Failed to export private key")
    })?;
    let public_key = pkey.public_key_to_pem().map_err(|e| {
        TinyBoardsError::from_error_message(e, 500, "Failed to export public key")
    })?;
    
    Ok(ActorKeypair {
        private_key: String::from_utf8_lossy(&private_key).to_string(),
        public_key: String::from_utf8_lossy(&public_key).to_string(),
    })
}