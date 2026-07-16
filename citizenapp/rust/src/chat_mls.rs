use std::{
    collections::HashSet,
    ffi::CStr,
    fs::{self, File},
    os::raw::c_char,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use openmls::{
    prelude::{
        tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize},
        BasicCredential, Ciphersuite, Credential, CredentialWithKey, Extensions, GroupId,
        KeyPackage, KeyPackageBundle, KeyPackageIn, MlsGroup, MlsGroupCreateConfig,
        MlsMessageBodyIn, MlsMessageIn, ProcessedMessageContent, ProtocolMessage,
        ProtocolVersion, RatchetTreeIn, StagedWelcome,
    },
    storage::OpenMlsProvider as OpenMlsStorageProvider,
};
use openmls_basic_credential::SignatureKeyPair;
use openmls_memory_storage::MemoryStorage;
use openmls_rust_crypto::{OpenMlsRustCrypto, RustCrypto};
use openmls_traits::{
    signatures::Signer, types::SignatureScheme, OpenMlsProvider as OpenMlsTraitsProvider,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const GMB_MLS_CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
const DEFAULT_KEYPACKAGE_TTL_MILLIS: u64 = 30 * 24 * 60 * 60 * 1000;

#[derive(Deserialize)]
struct CreateKeyPackageRequest {
    owner_account: String,
    device_id: String,
    state_store_dir: Option<String>,
}

#[derive(Deserialize)]
struct TwoPartySmokeRequest {
    plaintext: String,
}

#[derive(Deserialize)]
struct EncryptRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    conversation_id: String,
    recipient_account: String,
    plaintext_hex: String,
    recipient_key_package_hex: Option<String>,
}

#[derive(Deserialize)]
struct DecryptRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    conversation_id: String,
    wire_message_hex: String,
    ratchet_tree_hex: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DeviceRecord {
    owner_account: String,
    device_id: String,
    signature_public_key_hex: String,
    signature_scheme: String,
}

struct MlsProvider {
    crypto: RustCrypto,
    storage: MemoryStorage,
}

impl OpenMlsTraitsProvider for MlsProvider {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = MemoryStorage;

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
    }

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }
}

/// 生成真实 OpenMLS KeyPackage，并以 JSON 返回 hex。
///
/// # Safety
/// - `request_json` 必须是合法 UTF-8 C 字符串。
/// - 返回字符串必须由 `smoldot_free_string` 释放。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_create_key_package_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match create_key_package_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 执行真实 OpenMLS 双人组 round-trip smoke。
///
/// # Safety
/// - `request_json` 必须是合法 UTF-8 C 字符串。
/// - 返回字符串必须由 `smoldot_free_string` 释放。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_two_party_smoke_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match two_party_smoke_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 使用持久化 MLS 会话加密 application message。
///
/// # Safety
/// - `request_json` 必须是合法 UTF-8 C 字符串。
/// - 返回字符串必须由 `smoldot_free_string` 释放。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_encrypt_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match encrypt_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 处理 Welcome 或解密 application message。
///
/// # Safety
/// - `request_json` 必须是合法 UTF-8 C 字符串。
/// - 返回字符串必须由 `smoldot_free_string` 释放。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_decrypt_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match decrypt_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

fn create_key_package_json(request_json: *const c_char) -> Result<String, String> {
    let request: CreateKeyPackageRequest = parse_request(request_json)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;

    let (key_package_hex, cipher_suite, device_public_key_hex) =
        if let Some(dir) = request.state_store_dir.as_deref() {
            let state_dir = Path::new(dir);
            let provider = load_provider(state_dir)?;
            let (credential, signer) = ensure_device_signer(
                &provider,
                state_dir,
                &request.owner_account,
                &request.device_id,
            )?;
            let bundle = generate_key_package(&provider, &signer, credential)?;
            let key_package_hex = hex::encode(
                bundle
                    .key_package()
                    .tls_serialize_detached()
                    .map_err(|error| format!("序列化 OpenMLS KeyPackage 失败: {error}"))?,
            );
            let device_public_key_hex = hex::encode(signer.to_public_vec());
            save_provider(state_dir, &provider)?;
            (
                key_package_hex,
                format!("{:?}", GMB_MLS_CIPHERSUITE),
                device_public_key_hex,
            )
        } else {
            let provider = OpenMlsRustCrypto::default();
            let (credential, signer) = generate_credential(
                format!("{}:{}", request.owner_account, request.device_id).into_bytes(),
                GMB_MLS_CIPHERSUITE.signature_algorithm(),
                &provider,
            )?;
            let bundle = generate_key_package(&provider, &signer, credential)?;
            let key_package_hex = hex::encode(
                bundle
                    .key_package()
                    .tls_serialize_detached()
                    .map_err(|error| format!("序列化 OpenMLS KeyPackage 失败: {error}"))?,
            );
            (
                key_package_hex,
                format!("{:?}", GMB_MLS_CIPHERSUITE),
                hex::encode(signer.to_public_vec()),
            )
        };
    let now = now_millis();
    let response = json!({
        "protocol_version": 1,
        "owner_account": request.owner_account,
        "device_id": request.device_id,
        "device_public_key_hex": device_public_key_hex,
        "key_package_id": format!("kp-{}", key_package_hex.chars().take(24).collect::<String>()),
        "key_package_hex": key_package_hex,
        "cipher_suite": cipher_suite,
        "created_at_millis": now,
        "expires_at_millis": now + DEFAULT_KEYPACKAGE_TTL_MILLIS,
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn two_party_smoke_json(request_json: *const c_char) -> Result<String, String> {
    let request: TwoPartySmokeRequest = parse_request(request_json)?;
    require_non_empty("plaintext", &request.plaintext)?;

    let alice_provider = OpenMlsRustCrypto::default();
    let bob_provider = OpenMlsRustCrypto::default();

    let (alice_credential, alice_signer) = generate_credential(
        b"alice-wallet:alice-phone".to_vec(),
        GMB_MLS_CIPHERSUITE.signature_algorithm(),
        &alice_provider,
    )?;
    let (bob_credential, bob_signer) = generate_credential(
        b"bob-wallet:bob-phone".to_vec(),
        GMB_MLS_CIPHERSUITE.signature_algorithm(),
        &bob_provider,
    )?;
    let bob_key_package = generate_key_package(&bob_provider, &bob_signer, bob_credential)?
        .key_package()
        .clone();

    let group_config = MlsGroupCreateConfig::builder()
        .ciphersuite(GMB_MLS_CIPHERSUITE)
        .use_ratchet_tree_extension(true)
        .build();
    let group_id = GroupId::from_slice(b"gmb-im-native-smoke");
    let mut alice_group = MlsGroup::new_with_group_id(
        &alice_provider,
        &alice_signer,
        &group_config,
        group_id,
        alice_credential,
    )
    .map_err(|error| format!("创建 Alice OpenMLS group 失败: {error:?}"))?;

    let (_, welcome, _) = alice_group
        .add_members(&alice_provider, &alice_signer, &[bob_key_package.clone()])
        .map_err(|error| format!("添加 Bob KeyPackage 失败: {error:?}"))?;
    alice_group
        .merge_pending_commit(&alice_provider)
        .map_err(|error| format!("合并 Alice pending commit 失败: {error:?}"))?;

    let welcome_bytes = welcome
        .tls_serialize_detached()
        .map_err(|error| format!("序列化 OpenMLS Welcome 失败: {error}"))?;
    let welcome_in = MlsMessageIn::tls_deserialize_exact(welcome_bytes.clone())
        .map_err(|error| format!("反序列化 OpenMLS Welcome 失败: {error}"))?;
    let welcome = match welcome_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => welcome,
        _ => return Err("OpenMLS Welcome 类型错误".to_string()),
    };
    let mut bob_group = StagedWelcome::new_from_welcome(
        &bob_provider,
        group_config.join_config(),
        welcome,
        Some(alice_group.export_ratchet_tree().into()),
    )
    .map_err(|error| format!("Bob 处理 Welcome 失败: {error:?}"))?
    .into_group(&bob_provider)
    .map_err(|error| format!("Bob 创建 group 失败: {error:?}"))?;

    let message = alice_group
        .create_message(&alice_provider, &alice_signer, request.plaintext.as_bytes())
        .map_err(|error| format!("创建 OpenMLS application message 失败: {error:?}"))?;
    let message_bytes = message
        .clone()
        .tls_serialize_detached()
        .map_err(|error| format!("序列化 OpenMLS application message 失败: {error}"))?;
    let message_in = MlsMessageIn::tls_deserialize_exact(message_bytes.clone())
        .map_err(|error| format!("反序列化 OpenMLS message 失败: {error}"))?;
    let protocol_message = message_in
        .try_into_protocol_message()
        .map_err(|_| "OpenMLS message 不是 protocol message".to_string())?;
    let processed = bob_group
        .process_message(&bob_provider, protocol_message)
        .map_err(|error| format!("Bob 解密 OpenMLS message 失败: {error:?}"))?;
    let decrypted = match processed.into_content() {
        ProcessedMessageContent::ApplicationMessage(message) => {
            String::from_utf8(message.into_bytes())
                .map_err(|error| format!("OpenMLS 明文不是 UTF-8: {error}"))?
        }
        _ => return Err("OpenMLS 处理结果不是 application message".to_string()),
    };

    let response = json!({
        "plaintext": request.plaintext,
        "decrypted_plaintext": decrypted,
        "cipher_suite": format!("{:?}", GMB_MLS_CIPHERSUITE),
        "bob_key_package_hex": hex::encode(
            bob_key_package
                .tls_serialize_detached()
                .map_err(|error| format!("序列化 Bob KeyPackage 失败: {error}"))?,
        ),
        "welcome_hex": hex::encode(welcome_bytes),
        "alice_wire_message_hex": hex::encode(message_bytes),
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn encrypt_json(request_json: *const c_char) -> Result<String, String> {
    let request: EncryptRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("conversation_id", &request.conversation_id)?;
    require_non_empty("recipient_account", &request.recipient_account)?;
    require_non_empty("plaintext_hex", &request.plaintext_hex)?;

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let (credential, signer) = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.conversation_id)?;
    let plaintext = decode_hex_field("plaintext_hex", &request.plaintext_hex)?;
    let group_config = mls_group_config();

    let mut welcome_hex = None;
    let mut ratchet_tree_hex = None;
    let mut created_new_session = false;
    let mut group = match MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS group 失败: {error:?}"))?
    {
        Some(group) => group,
        None => {
            let key_package_hex = request
                .recipient_key_package_hex
                .as_deref()
                .ok_or_else(|| "首次 MLS 会话必须提供 recipient_key_package_hex".to_string())?;
            let key_package_bytes = decode_hex_field("recipient_key_package_hex", key_package_hex)?;
            let recipient_key_package: KeyPackage =
                KeyPackageIn::tls_deserialize_exact(key_package_bytes)
                    .map_err(|error| format!("反序列化 recipient KeyPackage 失败: {error}"))?
                    .validate(provider.crypto(), ProtocolVersion::default())
                    .map_err(|error| format!("验证 recipient KeyPackage 失败: {error:?}"))?;
            let mut new_group = MlsGroup::new_with_group_id(
                &provider,
                &signer,
                &group_config,
                group_id.clone(),
                credential,
            )
            .map_err(|error| format!("创建 MLS group 失败: {error:?}"))?;
            let (_, welcome, _) = new_group
                .add_members(&provider, &signer, &[recipient_key_package])
                .map_err(|error| format!("添加 recipient KeyPackage 失败: {error:?}"))?;
            new_group
                .merge_pending_commit(&provider)
                .map_err(|error| format!("合并 pending commit 失败: {error:?}"))?;
            let welcome_bytes = welcome
                .tls_serialize_detached()
                .map_err(|error| format!("序列化 Welcome 失败: {error}"))?;
            let tree_bytes = new_group
                .export_ratchet_tree()
                .tls_serialize_detached()
                .map_err(|error| format!("序列化 ratchet tree 失败: {error}"))?;
            welcome_hex = Some(hex::encode(welcome_bytes));
            ratchet_tree_hex = Some(hex::encode(tree_bytes));
            created_new_session = true;
            new_group
        }
    };

    let application_message = group
        .create_message(&provider, &signer, &plaintext)
        .map_err(|error| format!("创建 MLS application message 失败: {error:?}"))?;
    let application_hex = hex::encode(
        application_message
            .tls_serialize_detached()
            .map_err(|error| format!("序列化 MLS application message 失败: {error}"))?,
    );
    save_provider(state_dir, &provider)?;

    let response = json!({
        "conversation_id": request.conversation_id,
        "recipient_account": request.recipient_account,
        "cipher_suite": format!("{:?}", GMB_MLS_CIPHERSUITE),
        "created_new_session": created_new_session,
        "welcome_wire_message_hex": welcome_hex,
        "application_wire_message_hex": application_hex,
        "ratchet_tree_hex": ratchet_tree_hex,
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn decrypt_json(request_json: *const c_char) -> Result<String, String> {
    let request: DecryptRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("conversation_id", &request.conversation_id)?;
    require_non_empty("wire_message_hex", &request.wire_message_hex)?;

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let _ = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.conversation_id)?;
    let wire_bytes = decode_hex_field("wire_message_hex", &request.wire_message_hex)?;
    let message_in = MlsMessageIn::tls_deserialize_exact(wire_bytes)
        .map_err(|error| format!("反序列化 MLS wire message 失败: {error}"))?;

    let response = match message_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => {
            let ratchet_tree = match request.ratchet_tree_hex.as_deref() {
                Some(value) if !value.trim().is_empty() => {
                    let tree_bytes = decode_hex_field("ratchet_tree_hex", value)?;
                    Some(
                        RatchetTreeIn::tls_deserialize_exact(tree_bytes)
                            .map_err(|error| format!("反序列化 ratchet tree 失败: {error}"))?,
                    )
                }
                _ => None,
            };
            let group = StagedWelcome::new_from_welcome(
                &provider,
                mls_group_config().join_config(),
                welcome,
                ratchet_tree,
            )
            .map_err(|error| format!("处理 MLS Welcome 失败: {error:?}"))?
            .into_group(&provider)
            .map_err(|error| format!("从 Welcome 创建 MLS group 失败: {error:?}"))?;
            if group.group_id() != &group_id {
                return Err("Welcome group_id 与 conversation_id 不一致".to_string());
            }
            save_provider(state_dir, &provider)?;
            json!({
                "conversation_id": request.conversation_id,
                "message_kind": "welcome",
                "cipher_suite": format!("{:?}", GMB_MLS_CIPHERSUITE),
                "plaintext_hex": null,
            })
        }
        MlsMessageBodyIn::PublicMessage(message) => decrypt_protocol_message(
            state_dir,
            &provider,
            &request.conversation_id,
            group_id,
            message.into(),
        )?,
        MlsMessageBodyIn::PrivateMessage(message) => decrypt_protocol_message(
            state_dir,
            &provider,
            &request.conversation_id,
            group_id,
            message.into(),
        )?,
        _ => return Err("不支持的 MLS wire message 类型".to_string()),
    };
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn decrypt_protocol_message(
    state_dir: &Path,
    provider: &MlsProvider,
    conversation_id: &str,
    group_id: GroupId,
    protocol_message: openmls::prelude::ProtocolMessage,
) -> Result<serde_json::Value, String> {
    let mut group = MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS group 失败: {error:?}"))?
        .ok_or_else(|| "MLS 会话不存在，application message 需要先处理 Welcome".to_string())?;
    let processed = group
        .process_message(provider, protocol_message)
        .map_err(|error| format!("解密 MLS application message 失败: {error:?}"))?;
    let plaintext = match processed.into_content() {
        ProcessedMessageContent::ApplicationMessage(message) => message.into_bytes(),
        _ => return Err("MLS 处理结果不是 application message".to_string()),
    };
    save_provider(state_dir, provider)?;
    Ok(json!({
        "conversation_id": conversation_id,
        "message_kind": "application",
        "cipher_suite": format!("{:?}", GMB_MLS_CIPHERSUITE),
        "plaintext_hex": hex::encode(plaintext),
    }))
}

fn parse_request<T>(request_json: *const c_char) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    if request_json.is_null() {
        return Err("request_json is null".to_string());
    }
    let request = unsafe { CStr::from_ptr(request_json) }
        .to_str()
        .map_err(|_| "request_json 不是合法 UTF-8".to_string())?;
    serde_json::from_str(request).map_err(|error| format!("解析 request_json 失败: {error}"))
}

fn generate_credential(
    identity: Vec<u8>,
    signature_algorithm: SignatureScheme,
    provider: &impl OpenMlsStorageProvider,
) -> Result<(CredentialWithKey, SignatureKeyPair), String> {
    let credential = BasicCredential::new(identity);
    let signature_keys = SignatureKeyPair::new(signature_algorithm)
        .map_err(|error| format!("生成 OpenMLS 签名密钥失败: {error:?}"))?;
    signature_keys
        .store(provider.storage())
        .map_err(|error| format!("保存 OpenMLS 签名密钥失败: {error:?}"))?;
    Ok((
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keys.to_public_vec().into(),
        },
        signature_keys,
    ))
}

fn generate_key_package(
    provider: &impl OpenMlsStorageProvider,
    signer: &impl Signer,
    credential_with_key: CredentialWithKey,
) -> Result<KeyPackageBundle, String> {
    KeyPackage::builder()
        .key_package_extensions(Extensions::empty())
        .build(GMB_MLS_CIPHERSUITE, provider, signer, credential_with_key)
        .map_err(|error| format!("生成 OpenMLS KeyPackage 失败: {error:?}"))
}

fn load_provider(state_dir: &Path) -> Result<MlsProvider, String> {
    fs::create_dir_all(state_dir).map_err(|error| format!("创建 MLS 状态目录失败: {error}"))?;
    let mut storage = MemoryStorage::default();
    let storage_path = storage_path(state_dir);
    if storage_path.exists() {
        let file = File::open(&storage_path)
            .map_err(|error| format!("打开 OpenMLS storage 文件失败: {error}"))?;
        storage
            .load_from_file(&file)
            .map_err(|error| format!("加载 OpenMLS storage 失败: {error}"))?;
    }
    Ok(MlsProvider {
        crypto: RustCrypto::default(),
        storage,
    })
}

fn save_provider(state_dir: &Path, provider: &MlsProvider) -> Result<(), String> {
    fs::create_dir_all(state_dir).map_err(|error| format!("创建 MLS 状态目录失败: {error}"))?;
    let file = File::create(storage_path(state_dir))
        .map_err(|error| format!("创建 OpenMLS storage 文件失败: {error}"))?;
    provider
        .storage()
        .save_to_file(&file)
        .map_err(|error| format!("保存 OpenMLS storage 失败: {error}"))
}

fn ensure_device_signer(
    provider: &MlsProvider,
    state_dir: &Path,
    owner_account: &str,
    device_id: &str,
) -> Result<(CredentialWithKey, SignatureKeyPair), String> {
    let record_path = device_record_path(state_dir);
    let signature_algorithm = GMB_MLS_CIPHERSUITE.signature_algorithm();
    if record_path.exists() {
        let raw = fs::read_to_string(&record_path)
            .map_err(|error| format!("读取 MLS 设备记录失败: {error}"))?;
        let record: DeviceRecord = serde_json::from_str(&raw)
            .map_err(|error| format!("解析 MLS 设备记录失败: {error}"))?;
        if record.owner_account != owner_account || record.device_id != device_id {
            return Err("MLS 状态目录已绑定到其他钱包聊天账户或设备".to_string());
        }
        let public_key =
            decode_hex_field("signature_public_key_hex", &record.signature_public_key_hex)?;
        let signer = SignatureKeyPair::read(provider.storage(), &public_key, signature_algorithm)
            .ok_or_else(|| "MLS 设备签名密钥不在 OpenMLS storage 中".to_string())?;
        let credential = credential_with_public_key(owner_account, device_id, public_key);
        return Ok((credential, signer));
    }

    let (credential, signer) = generate_credential(
        format!("{owner_account}:{device_id}").into_bytes(),
        signature_algorithm,
        provider,
    )?;
    let record = DeviceRecord {
        owner_account: owner_account.to_string(),
        device_id: device_id.to_string(),
        signature_public_key_hex: hex::encode(signer.to_public_vec()),
        signature_scheme: format!("{:?}", signature_algorithm),
    };
    fs::write(
        record_path,
        serde_json::to_string_pretty(&record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("写入 MLS 设备记录失败: {error}"))?;
    Ok((credential, signer))
}

fn credential_with_public_key(
    owner_account: &str,
    device_id: &str,
    public_key: Vec<u8>,
) -> CredentialWithKey {
    CredentialWithKey {
        credential: BasicCredential::new(format!("{owner_account}:{device_id}").into_bytes())
            .into(),
        signature_key: public_key.into(),
    }
}

fn mls_group_config() -> MlsGroupCreateConfig {
    MlsGroupCreateConfig::builder()
        .ciphersuite(GMB_MLS_CIPHERSUITE)
        .use_ratchet_tree_extension(true)
        .build()
}

fn group_id_from_conversation(conversation_id: &str) -> Result<GroupId, String> {
    require_non_empty("conversation_id", conversation_id)?;
    Ok(GroupId::from_slice(conversation_id.as_bytes()))
}

fn decode_hex_field(field_name: &str, value: &str) -> Result<Vec<u8>, String> {
    let normalized = value.strip_prefix("0x").unwrap_or(value);
    if normalized.is_empty() {
        return Err(format!("{field_name} 不能为空"));
    }
    if normalized.len() % 2 != 0 {
        return Err(format!("{field_name} hex 长度必须为偶数"));
    }
    hex::decode(normalized).map_err(|error| format!("{field_name} 不是合法 hex: {error}"))
}

fn storage_path(state_dir: &Path) -> PathBuf {
    state_dir.join("openmls_storage.json")
}

fn device_record_path(state_dir: &Path) -> PathBuf {
    state_dir.join("device.json")
}

fn require_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("OpenMLS 字段 {field_name} 不能为空"));
    }
    Ok(())
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// 私密小群(MLS 群原生)FFI —— 单次加密 + 发送端扇出,服务端零存储不变。
// 密文/Welcome/Commit 都由本层生成,Dart 只按名册封 N 信封投递。
// 详见 memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md。
// ============================================================================

/// 单群成员硬上限。发送端(Dart)与本层(MLS 实际成员数)双拦,任一超限即拒。
const MAX_GROUP_MEMBERS: usize = 1989;

#[derive(Deserialize)]
struct GroupCreateRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    group_id: String,
}

#[derive(Deserialize)]
struct GroupAddMembersRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    group_id: String,
    key_packages_hex: Vec<String>,
}

#[derive(Deserialize)]
struct GroupRemoveMembersRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    group_id: String,
    /// 按**账户**移除(移除该账户在群内的全部设备叶子)。
    member_accounts: Vec<String>,
}

#[derive(Deserialize)]
struct GroupCreateMessageRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    group_id: String,
    plaintext_hex: String,
}

#[derive(Deserialize)]
struct GroupProcessRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    group_id: String,
    wire_message_hex: String,
    ratchet_tree_hex: Option<String>,
}

#[derive(Deserialize)]
struct GroupStateRequest {
    state_store_dir: String,
    owner_account: String,
    device_id: String,
    group_id: String,
}

/// 创建 MLS 群(创建者为唯一成员,epoch 0)。
///
/// # Safety
/// 见 `gmb_chat_mls_create_key_package_json`。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_group_create_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match group_create_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 批量加人:产 1 个 Commit(发给现有成员)+ 1 个 Welcome(发给全部新人)。
///
/// # Safety
/// 见 `gmb_chat_mls_create_key_package_json`。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_group_add_members_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match group_add_members_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 删人:产 Commit(发给剩余成员 + 被删者)。
///
/// # Safety
/// 见 `gmb_chat_mls_create_key_package_json`。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_group_remove_members_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match group_remove_members_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 群 application message:单次加密,Dart 侧按名册扇 N 信封。
///
/// # Safety
/// 见 `gmb_chat_mls_create_key_package_json`。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_group_create_message_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match group_create_message_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 处理入站群消息(Welcome / Commit / Application)。收端唯一入口,按 epoch 判定
/// applied / out_of_order / stale,乱序缓冲由 Dart 依此状态负责。
///
/// # Safety
/// 见 `gmb_chat_mls_create_key_package_json`。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_group_process_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match group_process_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 只读群状态:当前 epoch + 成员名册(MLS 真源,供 Dart 镜像对账与上限守)。
///
/// # Safety
/// 见 `gmb_chat_mls_create_key_package_json`。
#[no_mangle]
pub unsafe extern "C" fn gmb_chat_mls_group_state_json(
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match group_state_json(request_json) {
        Ok(value) => crate::string_into_raw(value, error_out),
        Err(message) => {
            crate::set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 从 BasicCredential 还原成员标识("owner_account:device_id")。
fn identity_of(credential: &Credential) -> String {
    String::from_utf8_lossy(credential.serialized_content()).into_owned()
}

/// 从成员标识取账户段("owner_account:device_id" → "owner_account")。
fn account_of(credential: &Credential) -> String {
    let identity = identity_of(credential);
    match identity.split_once(':') {
        Some((account, _)) => account.to_string(),
        None => identity,
    }
}

fn group_create_json(request_json: *const c_char) -> Result<String, String> {
    let request: GroupCreateRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("group_id", &request.group_id)?;

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let (credential, signer) = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.group_id)?;
    if MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS 群失败: {error:?}"))?
        .is_some()
    {
        return Err("MLS 群已存在，请勿重复创建".to_string());
    }
    let group = MlsGroup::new_with_group_id(
        &provider,
        &signer,
        &mls_group_config(),
        group_id,
        credential,
    )
    .map_err(|error| format!("创建 MLS 群失败: {error:?}"))?;
    let epoch = group.epoch().as_u64();
    save_provider(state_dir, &provider)?;

    let response = json!({
        "group_id": request.group_id,
        "epoch": epoch,
        "cipher_suite": format!("{:?}", GMB_MLS_CIPHERSUITE),
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn group_add_members_json(request_json: *const c_char) -> Result<String, String> {
    let request: GroupAddMembersRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("group_id", &request.group_id)?;
    if request.key_packages_hex.is_empty() {
        return Err("group_add_members 至少需要一个 KeyPackage".to_string());
    }

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let (_credential, signer) = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.group_id)?;
    let mut group = MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS 群失败: {error:?}"))?
        .ok_or_else(|| "MLS 群不存在，无法加人".to_string())?;

    // 1989 硬拦(以 MLS 实际成员数为准,Dart 侧另有一道)。
    let current = group.members().count();
    let adding = request.key_packages_hex.len();
    if current + adding > MAX_GROUP_MEMBERS {
        return Err(format!(
            "群成员将达 {}，超过上限 {MAX_GROUP_MEMBERS}",
            current + adding
        ));
    }

    let mut key_packages = Vec::with_capacity(adding);
    for (index, kp_hex) in request.key_packages_hex.iter().enumerate() {
        let bytes = decode_hex_field(&format!("key_packages_hex[{index}]"), kp_hex)?;
        let key_package: KeyPackage = KeyPackageIn::tls_deserialize_exact(bytes)
            .map_err(|error| format!("反序列化 KeyPackage[{index}] 失败: {error}"))?
            .validate(provider.crypto(), ProtocolVersion::default())
            .map_err(|error| format!("验证 KeyPackage[{index}] 失败: {error:?}"))?;
        key_packages.push(key_package);
    }

    let (commit, welcome, _group_info) = group
        .add_members(&provider, &signer, &key_packages)
        .map_err(|error| format!("MLS 加人失败: {error:?}"))?;
    group
        .merge_pending_commit(&provider)
        .map_err(|error| format!("合并 pending commit 失败: {error:?}"))?;

    let commit_wire_hex = hex::encode(
        commit
            .tls_serialize_detached()
            .map_err(|error| format!("序列化 Commit 失败: {error}"))?,
    );
    let welcome_wire_hex = hex::encode(
        welcome
            .tls_serialize_detached()
            .map_err(|error| format!("序列化 Welcome 失败: {error}"))?,
    );
    let ratchet_tree_hex = hex::encode(
        group
            .export_ratchet_tree()
            .tls_serialize_detached()
            .map_err(|error| format!("序列化 ratchet tree 失败: {error}"))?,
    );
    let epoch = group.epoch().as_u64();
    save_provider(state_dir, &provider)?;

    let response = json!({
        "group_id": request.group_id,
        "epoch": epoch,
        "commit_wire_hex": commit_wire_hex,
        "welcome_wire_hex": welcome_wire_hex,
        "ratchet_tree_hex": ratchet_tree_hex,
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn group_remove_members_json(request_json: *const c_char) -> Result<String, String> {
    let request: GroupRemoveMembersRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("group_id", &request.group_id)?;
    if request.member_accounts.is_empty() {
        return Err("group_remove_members 至少需要一个成员账户".to_string());
    }

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let (_credential, signer) = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.group_id)?;
    let mut group = MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS 群失败: {error:?}"))?
        .ok_or_else(|| "MLS 群不存在，无法删人".to_string())?;

    // 按账户移除:该账户在群内的**全部设备叶子**都进移除集。
    let targets: HashSet<&str> = request
        .member_accounts
        .iter()
        .map(|value| value.as_str())
        .collect();
    let mut indices = Vec::new();
    let mut removed_accounts = HashSet::new();
    for member in group.members() {
        let account = account_of(&member.credential);
        if targets.contains(account.as_str()) {
            indices.push(member.index);
            removed_accounts.insert(account);
        }
    }
    if indices.is_empty() {
        return Err("未在群名册中找到要移除的成员".to_string());
    }
    let removed_accounts: Vec<String> = removed_accounts.into_iter().collect();

    let (commit, _welcome, _group_info) = group
        .remove_members(&provider, &signer, &indices)
        .map_err(|error| format!("MLS 删人失败: {error:?}"))?;
    group
        .merge_pending_commit(&provider)
        .map_err(|error| format!("合并 pending commit 失败: {error:?}"))?;

    let commit_wire_hex = hex::encode(
        commit
            .tls_serialize_detached()
            .map_err(|error| format!("序列化 Commit 失败: {error}"))?,
    );
    let epoch = group.epoch().as_u64();
    save_provider(state_dir, &provider)?;

    let response = json!({
        "group_id": request.group_id,
        "epoch": epoch,
        "commit_wire_hex": commit_wire_hex,
        "removed_accounts": removed_accounts,
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn group_create_message_json(request_json: *const c_char) -> Result<String, String> {
    let request: GroupCreateMessageRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("group_id", &request.group_id)?;
    require_non_empty("plaintext_hex", &request.plaintext_hex)?;

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let (_credential, signer) = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.group_id)?;
    let mut group = MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS 群失败: {error:?}"))?
        .ok_or_else(|| "MLS 群不存在，无法发消息".to_string())?;

    let plaintext = decode_hex_field("plaintext_hex", &request.plaintext_hex)?;
    let message = group
        .create_message(&provider, &signer, &plaintext)
        .map_err(|error| format!("创建群 application message 失败: {error:?}"))?;
    let application_wire_hex = hex::encode(
        message
            .tls_serialize_detached()
            .map_err(|error| format!("序列化群 application message 失败: {error}"))?,
    );
    let epoch = group.epoch().as_u64();
    save_provider(state_dir, &provider)?;

    let response = json!({
        "group_id": request.group_id,
        "epoch": epoch,
        "application_wire_hex": application_wire_hex,
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

fn group_process_json(request_json: *const c_char) -> Result<String, String> {
    let request: GroupProcessRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("group_id", &request.group_id)?;
    require_non_empty("wire_message_hex", &request.wire_message_hex)?;

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let _ = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.group_id)?;
    let wire_bytes = decode_hex_field("wire_message_hex", &request.wire_message_hex)?;
    let message_in = MlsMessageIn::tls_deserialize_exact(wire_bytes)
        .map_err(|error| format!("反序列化 MLS wire message 失败: {error}"))?;

    let response = match message_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => {
            let ratchet_tree = match request.ratchet_tree_hex.as_deref() {
                Some(value) if !value.trim().is_empty() => {
                    let tree_bytes = decode_hex_field("ratchet_tree_hex", value)?;
                    Some(
                        RatchetTreeIn::tls_deserialize_exact(tree_bytes)
                            .map_err(|error| format!("反序列化 ratchet tree 失败: {error}"))?,
                    )
                }
                _ => None,
            };
            let group = StagedWelcome::new_from_welcome(
                &provider,
                mls_group_config().join_config(),
                welcome,
                ratchet_tree,
            )
            .map_err(|error| format!("处理群 Welcome 失败: {error:?}"))?
            .into_group(&provider)
            .map_err(|error| format!("从 Welcome 创建群失败: {error:?}"))?;
            if group.group_id() != &group_id {
                return Err("Welcome group_id 与 group_id 不一致".to_string());
            }
            let epoch = group.epoch().as_u64();
            let members: Vec<String> =
                group.members().map(|m| identity_of(&m.credential)).collect();
            save_provider(state_dir, &provider)?;
            json!({
                "group_id": request.group_id,
                "message_kind": "welcome",
                "status": "applied",
                "message_epoch": epoch,
                "group_epoch": epoch,
                "self_removed": false,
                "plaintext_hex": serde_json::Value::Null,
                "member_identities": members,
            })
        }
        MlsMessageBodyIn::PublicMessage(message) => process_group_protocol(
            state_dir,
            &provider,
            &request.group_id,
            group_id,
            message.into(),
        )?,
        MlsMessageBodyIn::PrivateMessage(message) => process_group_protocol(
            state_dir,
            &provider,
            &request.group_id,
            group_id,
            message.into(),
        )?,
        _ => return Err("不支持的群 MLS wire message 类型".to_string()),
    };
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

/// Commit/Application 的 epoch 有序处理。message_epoch>current→out_of_order(不处理,
/// Dart 缓冲);<current 或解密失败→stale;==→应用并回吐名册/自我移除标志。
fn process_group_protocol(
    state_dir: &Path,
    provider: &MlsProvider,
    conversation_id: &str,
    group_id: GroupId,
    protocol_message: ProtocolMessage,
) -> Result<serde_json::Value, String> {
    let message_epoch = protocol_message.epoch().as_u64();
    let mut group = MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS 群失败: {error:?}"))?
        .ok_or_else(|| "群会话不存在，需先处理 Welcome".to_string())?;
    let current = group.epoch().as_u64();

    if message_epoch > current {
        return Ok(json!({
            "group_id": conversation_id,
            "message_kind": "unknown",
            "status": "out_of_order",
            "message_epoch": message_epoch,
            "group_epoch": current,
            "self_removed": false,
            "plaintext_hex": serde_json::Value::Null,
            "member_identities": serde_json::Value::Null,
        }));
    }

    let processed = match group.process_message(provider, protocol_message) {
        Ok(processed) => processed,
        Err(error) => {
            return Ok(json!({
                "group_id": conversation_id,
                "message_kind": "unknown",
                "status": "stale",
                "message_epoch": message_epoch,
                "group_epoch": current,
                "self_removed": false,
                "plaintext_hex": serde_json::Value::Null,
                "member_identities": serde_json::Value::Null,
                "detail": format!("{error:?}"),
            }));
        }
    };

    match processed.into_content() {
        ProcessedMessageContent::ApplicationMessage(message) => {
            let plaintext = message.into_bytes();
            let epoch = group.epoch().as_u64();
            save_provider(state_dir, provider)?;
            Ok(json!({
                "group_id": conversation_id,
                "message_kind": "application",
                "status": "applied",
                "message_epoch": message_epoch,
                "group_epoch": epoch,
                "self_removed": false,
                "plaintext_hex": hex::encode(plaintext),
                "member_identities": serde_json::Value::Null,
            }))
        }
        ProcessedMessageContent::StagedCommitMessage(staged) => {
            let self_removed = staged.self_removed();
            group
                .merge_staged_commit(provider, *staged)
                .map_err(|error| format!("合并群 Commit 失败: {error:?}"))?;
            let epoch = group.epoch().as_u64();
            let members: Vec<String> = if group.is_active() {
                group.members().map(|m| identity_of(&m.credential)).collect()
            } else {
                Vec::new()
            };
            save_provider(state_dir, provider)?;
            Ok(json!({
                "group_id": conversation_id,
                "message_kind": "commit",
                "status": "applied",
                "message_epoch": message_epoch,
                "group_epoch": epoch,
                "self_removed": self_removed,
                "plaintext_hex": serde_json::Value::Null,
                "member_identities": members,
            }))
        }
        _ => Err("群暂不支持独立提案消息".to_string()),
    }
}

fn group_state_json(request_json: *const c_char) -> Result<String, String> {
    let request: GroupStateRequest = parse_request(request_json)?;
    require_non_empty("state_store_dir", &request.state_store_dir)?;
    require_non_empty("owner_account", &request.owner_account)?;
    require_non_empty("device_id", &request.device_id)?;
    require_non_empty("group_id", &request.group_id)?;

    let state_dir = Path::new(&request.state_store_dir);
    let provider = load_provider(state_dir)?;
    let _ = ensure_device_signer(
        &provider,
        state_dir,
        &request.owner_account,
        &request.device_id,
    )?;
    let group_id = group_id_from_conversation(&request.group_id)?;
    let group = MlsGroup::load(provider.storage(), &group_id)
        .map_err(|error| format!("加载 MLS 群失败: {error:?}"))?
        .ok_or_else(|| "MLS 群不存在".to_string())?;
    let epoch = group.epoch().as_u64();
    let members: Vec<String> = group.members().map(|m| identity_of(&m.credential)).collect();

    let response = json!({
        "group_id": request.group_id,
        "epoch": epoch,
        "member_count": members.len(),
        "member_identities": members,
    });
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        create_key_package_json, group_add_members_json, group_create_json,
        group_create_message_json, group_process_json, group_remove_members_json,
        group_state_json, two_party_smoke_json,
    };
    use std::ffi::CString;

    #[test]
    fn creates_real_openmls_key_package() {
        let request = CString::new(r#"{"owner_account":"alice-wallet","device_id":"alice-phone"}"#)
            .expect("request should be valid");
        let response =
            create_key_package_json(request.as_ptr()).expect("key package should be created");
        let json: serde_json::Value =
            serde_json::from_str(&response).expect("response should be json");
        assert_eq!(json["owner_account"], "alice-wallet");
        assert!(json["key_package_hex"].as_str().unwrap().len() > 100);
    }

    #[test]
    fn openmls_two_party_smoke_round_trips_plaintext() {
        let request =
            CString::new(r#"{"plaintext":"hello openmls"}"#).expect("request should be valid");
        let response = two_party_smoke_json(request.as_ptr()).expect("smoke should pass");
        let json: serde_json::Value =
            serde_json::from_str(&response).expect("response should be json");
        assert_eq!(json["plaintext"], "hello openmls");
        assert_eq!(json["decrypted_plaintext"], "hello openmls");
        assert!(json["alice_wire_message_hex"].as_str().unwrap().len() > 100);
    }

    #[test]
    fn group_three_party_round_trip() {
        use serde_json::json;
        use std::fs;
        use std::os::raw::c_char;
        use std::path::Path;

        let base = std::env::temp_dir().join(format!("gmb_group_rt_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let dir_a = base.join("a");
        let dir_b = base.join("b");
        let dir_c = base.join("c");
        for d in [&dir_a, &dir_b, &dir_c] {
            fs::create_dir_all(d).expect("临时目录应可创建");
        }
        let group_id = "grp:acctA:testnonce";
        let path = |p: &Path| p.to_str().unwrap().to_string();

        let invoke = |f: fn(*const c_char) -> Result<String, String>, req: serde_json::Value| {
            let c = CString::new(serde_json::to_string(&req).unwrap()).unwrap();
            let out = f(c.as_ptr()).expect("FFI 调用应成功");
            serde_json::from_str::<serde_json::Value>(&out).expect("响应应为 JSON")
        };

        // A 建群(创建者=唯一成员,epoch 0)。
        let created = invoke(
            group_create_json,
            json!({"state_store_dir": path(&dir_a), "owner_account": "acctA", "device_id": "devA", "group_id": group_id}),
        );
        assert_eq!(created["epoch"].as_u64(), Some(0));

        // B / C 生成 KeyPackage。
        let b_kp = invoke(
            create_key_package_json,
            json!({"owner_account": "acctB", "device_id": "devB", "state_store_dir": path(&dir_b)}),
        )["key_package_hex"]
            .as_str()
            .unwrap()
            .to_string();
        let c_kp = invoke(
            create_key_package_json,
            json!({"owner_account": "acctC", "device_id": "devC", "state_store_dir": path(&dir_c)}),
        )["key_package_hex"]
            .as_str()
            .unwrap()
            .to_string();

        // A 批量加 B、C(1 Commit + 1 Welcome)。
        let added = invoke(
            group_add_members_json,
            json!({"state_store_dir": path(&dir_a), "owner_account": "acctA", "device_id": "devA", "group_id": group_id, "key_packages_hex": [b_kp, c_kp]}),
        );
        let welcome_hex = added["welcome_wire_hex"].as_str().unwrap().to_string();
        let tree_hex = added["ratchet_tree_hex"].as_str().unwrap().to_string();
        assert_eq!(added["epoch"].as_u64(), Some(1));

        // B / C 处理 Welcome 入群,名册应为 3 人。
        for (dir, owner, dev) in [(&dir_b, "acctB", "devB"), (&dir_c, "acctC", "devC")] {
            let joined = invoke(
                group_process_json,
                json!({"state_store_dir": path(dir.as_path()), "owner_account": owner, "device_id": dev, "group_id": group_id, "wire_message_hex": welcome_hex, "ratchet_tree_hex": tree_hex}),
            );
            assert_eq!(joined["message_kind"].as_str(), Some("welcome"));
            assert_eq!(joined["member_identities"].as_array().unwrap().len(), 3);
        }

        // A 发文本,B/C 端到端解密。
        let plaintext_hex = hex::encode("你好，群聊".as_bytes());
        let msg = invoke(
            group_create_message_json,
            json!({"state_store_dir": path(&dir_a), "owner_account": "acctA", "device_id": "devA", "group_id": group_id, "plaintext_hex": plaintext_hex}),
        );
        let app_hex = msg["application_wire_hex"].as_str().unwrap().to_string();
        for (dir, owner, dev) in [(&dir_b, "acctB", "devB"), (&dir_c, "acctC", "devC")] {
            let got = invoke(
                group_process_json,
                json!({"state_store_dir": path(dir.as_path()), "owner_account": owner, "device_id": dev, "group_id": group_id, "wire_message_hex": app_hex}),
            );
            assert_eq!(got["message_kind"].as_str(), Some("application"));
            assert_eq!(got["status"].as_str(), Some("applied"));
            assert_eq!(got["plaintext_hex"].as_str(), Some(plaintext_hex.as_str()));
        }

        // A 移除 C。
        let removed = invoke(
            group_remove_members_json,
            json!({"state_store_dir": path(&dir_a), "owner_account": "acctA", "device_id": "devA", "group_id": group_id, "member_accounts": ["acctC"]}),
        );
        let remove_commit_hex = removed["commit_wire_hex"].as_str().unwrap().to_string();
        assert_eq!(removed["epoch"].as_u64(), Some(2));

        // B 应用 Commit → 名册剩 A、B,未自我移除。
        let b_after = invoke(
            group_process_json,
            json!({"state_store_dir": path(&dir_b), "owner_account": "acctB", "device_id": "devB", "group_id": group_id, "wire_message_hex": remove_commit_hex}),
        );
        assert_eq!(b_after["message_kind"].as_str(), Some("commit"));
        assert_eq!(b_after["self_removed"].as_bool(), Some(false));
        assert_eq!(b_after["member_identities"].as_array().unwrap().len(), 2);

        // C 应用 Commit → 自身被移除(后向保密)。
        let c_after = invoke(
            group_process_json,
            json!({"state_store_dir": path(&dir_c), "owner_account": "acctC", "device_id": "devC", "group_id": group_id, "wire_message_hex": remove_commit_hex}),
        );
        assert_eq!(c_after["self_removed"].as_bool(), Some(true));

        // group_state 名册对账 = A、B。
        let state = invoke(
            group_state_json,
            json!({"state_store_dir": path(&dir_a), "owner_account": "acctA", "device_id": "devA", "group_id": group_id}),
        );
        assert_eq!(state["member_count"].as_u64(), Some(2));

        let _ = fs::remove_dir_all(&base);
    }
}
