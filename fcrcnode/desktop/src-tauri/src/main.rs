use base64::Engine;
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey as Ed25519PublicKey};
use hex::FromHex;
use image::Luma;
use qrcode::QrCode;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use std::io::Cursor;

#[tauri::command]
fn generate_qr_data_url(payload: String) -> Result<String, String> {
    let code = QrCode::new(payload.as_bytes()).map_err(|e| e.to_string())?;
    let image = code.render::<Luma<u8>>().min_dimensions(220, 220).build();

    let mut buffer: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(buffer);
    Ok(format!("data:image/png;base64,{}", b64))
}

fn decode_hex_32(label: &str, value: &str) -> Result<[u8; 32], String> {
    let text = value.trim().trim_start_matches("0x");
    let bytes = <Vec<u8>>::from_hex(text).map_err(|_| format!("invalid {} hex", label))?;
    if bytes.len() != 32 {
        return Err(format!("{} must be 32 bytes", label));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_signature(value: &str) -> Result<Vec<u8>, String> {
    let text = value.trim();
    if text.is_empty() {
        return Err("signature is empty".into());
    }

    if text.starts_with("0x") {
        let raw = <Vec<u8>>::from_hex(text.trim_start_matches("0x"))
            .map_err(|_| "invalid signature hex".to_string())?;
        return Ok(raw);
    }

    if text.chars().all(|c| c.is_ascii_hexdigit()) && text.len() % 2 == 0 {
        let raw = <Vec<u8>>::from_hex(text).map_err(|_| "invalid signature hex".to_string())?;
        return Ok(raw);
    }

    base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|_| "invalid signature encoding".into())
}

fn normalize_signature(raw: Vec<u8>) -> Vec<u8> {
    if raw.len() == 65 {
        raw[1..].to_vec()
    } else {
        raw
    }
}

fn verify_sr25519(public_key: [u8; 32], signature: &[u8], message: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }

    let Ok(public) = Sr25519PublicKey::from_bytes(&public_key) else {
        return false;
    };
    let Ok(signature_bytes) = <[u8; 64]>::try_from(signature) else {
        return false;
    };
    let Ok(sig) = Sr25519Signature::from_bytes(&signature_bytes) else {
        return false;
    };
    public
        .verify(signing_context(b"substrate").bytes(message), &sig)
        .is_ok()
}

fn verify_ed25519(public_key: [u8; 32], signature: &[u8], message: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }
    let Ok(public) = Ed25519PublicKey::from_bytes(&public_key) else {
        return false;
    };
    let Ok(sig) = Ed25519Signature::from_slice(signature) else {
        return false;
    };
    public.verify(message, &sig).is_ok()
}

#[tauri::command]
fn verify_login_signature(
    payload: String,
    signature: String,
    public_key: String,
    crypto: Option<String>,
) -> Result<bool, String> {
    let message = payload.as_bytes();
    let public_key = decode_hex_32("public_key", &public_key)?;
    let signature = normalize_signature(decode_signature(&signature)?);

    let verified = match crypto.as_deref() {
        Some("sr25519") => verify_sr25519(public_key, &signature, message),
        Some("ed25519") => verify_ed25519(public_key, &signature, message),
        _ => {
            // Auto mode: try sr25519 first, then ed25519.
            verify_sr25519(public_key, &signature, message)
                || verify_ed25519(public_key, &signature, message)
        }
    };

    Ok(verified)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            generate_qr_data_url,
            verify_login_signature
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
