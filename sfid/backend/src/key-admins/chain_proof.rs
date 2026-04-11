use serde::{Deserialize, Serialize};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEnvelope {
    pub key_id: String,
    pub key_version: String,
    pub alg: String,
    pub payload: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicKeyOutput {
    pub key_id: String,
    pub key_version: String,
    pub alg: String,
    pub public_key_hex: String,
}

pub fn build_public_key_output(
    key_id: &str,
    key_version: &str,
    key_alg: &str,
    public_key_hex: &str,
) -> PublicKeyOutput {
    PublicKeyOutput {
        key_id: key_id.to_string(),
        key_version: key_version.to_string(),
        alg: key_alg.to_string(),
        public_key_hex: public_key_hex.to_string(),
    }
}

#[allow(dead_code)]
pub fn make_signature_envelope<T: Serialize>(
    key_id: &str,
    key_version: &str,
    key_alg: &str,
    signing_key: &Sr25519Pair,
    payload: &T,
) -> Result<SignatureEnvelope, String> {
    let payload_text =
        serde_json::to_string(payload).map_err(|err| format!("serialize payload failed: {err}"))?;
    let signature = signing_key.sign(payload_text.as_bytes());

    Ok(SignatureEnvelope {
        key_id: key_id.to_string(),
        key_version: key_version.to_string(),
        alg: key_alg.to_string(),
        payload: payload_text,
        signature_hex: hex::encode(signature.0),
    })
}
