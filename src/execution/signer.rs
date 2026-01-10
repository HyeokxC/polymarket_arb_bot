use anyhow::Result;
use k256::ecdsa::{SigningKey, Signature, signature::Signer};
use sha2::{Sha256, Digest};
use hex;

pub struct TxSigner {
    signing_key: SigningKey,
}

impl TxSigner {
    pub fn new(secret: &str) -> Result<Self> {
        // Parse secret key from hex string
        let secret_bytes = hex::decode(secret)?;
        
        // Ensure we have exactly 32 bytes
        if secret_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Secret key must be 32 bytes"));
        }
        
        let signing_key = SigningKey::from_slice(&secret_bytes)?;
        
        Ok(Self { signing_key })
    }


    pub fn sign_message(&self, message: &[u8]) -> Result<String> {
        // Hash the message
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash = hasher.finalize();

        // Sign the hash
        let signature: Signature = self.signing_key.sign(&hash);
        
        // Return hex-encoded signature
        Ok(hex::encode(signature.to_bytes()))
    }

    pub fn get_public_key(&self) -> String {
        let verifying_key = self.signing_key.verifying_key();
        hex::encode(verifying_key.to_encoded_point(false).as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_creation() {
        // Generate a test key (32 bytes)
        let test_secret = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        
        let signer = TxSigner::new(test_secret);
        assert!(signer.is_ok());
    }

    #[test]
    fn test_sign_message() {
        let test_secret = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let signer = TxSigner::new(test_secret).unwrap();
        
        let message = b"test message";
        let signature = signer.sign_message(message);
        
        assert!(signature.is_ok());
        assert_eq!(signature.unwrap().len(), 128); // 64 bytes = 128 hex chars
    }
}
