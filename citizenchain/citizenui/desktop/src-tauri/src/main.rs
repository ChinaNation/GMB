use base64::Engine;
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey as Ed25519PublicKey};
use hex::FromHex;
use image::Luma;
use qrcode::QrCode;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use std::{
    env,
    fs,
    io::Cursor,
    path::PathBuf,
    process::{Child, Command},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, RunEvent};

struct LocalNodeState(Mutex<Option<Child>>);

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

fn candidate_node_bins(app: &AppHandle) -> Vec<PathBuf> {
    let mut bins = Vec::new();
    if let Ok(path) = env::var("CITIZENCHAIN_NODE_BIN") {
        if !path.trim().is_empty() {
            bins.push(PathBuf::from(path));
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bins.push(manifest_dir.join("../../../target/debug/citizenchain-node"));
    bins.push(manifest_dir.join("../../../target/debug/node"));
    bins.push(manifest_dir.join("../../../target/release/citizenchain-node"));
    bins.push(manifest_dir.join("../../../target/release/node"));

    if let Ok(resource_dir) = app.path().resource_dir() {
        bins.push(resource_dir.join("citizenchain-node"));
        bins.push(resource_dir.join("citizenchain-node.exe"));
        bins.push(resource_dir.join("node"));
        bins.push(resource_dir.join("node.exe"));
        bins.push(resource_dir.join("binaries/citizenchain-node"));
        bins.push(resource_dir.join("binaries/citizenchain-node.exe"));
    }

    bins
}

fn spawn_local_node(app: &AppHandle) -> Result<Child, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    let node_data = app_data.join("node-data");
    fs::create_dir_all(&node_data).map_err(|e| format!("create node data dir failed: {e}"))?;

    for bin in candidate_node_bins(app) {
        if !bin.exists() {
            continue;
        }
        let mut cmd = Command::new(&bin);
        cmd.arg("--base-path")
            .arg(node_data.as_os_str())
            .arg("--rpc-port")
            .arg("9944");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
        }
        if let Ok(child) = cmd.spawn() {
            return Ok(child);
        }
    }

    Err("cannot start local node; set CITIZENCHAIN_NODE_BIN to node executable path".into())
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        let pid = child.id() as i32;
        if pid > 0 {
            let _ = libc::kill(-pid, libc::SIGTERM);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn main() {
    let app = tauri::Builder::default()
        .manage(LocalNodeState(Mutex::new(None)))
        .setup(|app| {
            match spawn_local_node(app.handle()) {
                Ok(child) => {
                    if let Ok(mut state) = app.state::<LocalNodeState>().0.lock() {
                        *state = Some(child);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_qr_data_url,
            verify_login_signature
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            if let Ok(mut state) = app.state::<LocalNodeState>().0.lock() {
                if let Some(mut child) = state.take() {
                    terminate_child(&mut child);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{decode_hex_32, normalize_signature};

    #[test]
    fn decode_hex_32_accepts_prefixed_hex() {
        let key = decode_hex_32(
            "public_key",
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("hex should decode");
        assert_eq!(key[0], 0x01);
        assert_eq!(key[31], 0xef);
    }

    #[test]
    fn decode_hex_32_rejects_wrong_length() {
        assert!(decode_hex_32("public_key", "0x1234").is_err());
    }

    #[test]
    fn normalize_signature_strips_multisig_prefix() {
        let normalized = normalize_signature(vec![0, 1, 2, 3]);
        assert_eq!(normalized, vec![0, 1, 2, 3]);

        let mut prefixed = vec![1];
        prefixed.extend_from_slice(&[9; 64]);
        let normalized_prefixed = normalize_signature(prefixed);
        assert_eq!(normalized_prefixed.len(), 64);
        assert_eq!(normalized_prefixed[0], 9);
    }
}
