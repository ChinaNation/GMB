// 通用 keystore 操作：扫描链目录、写入/删除/检测密钥文件。
use crate::shared::security;
use std::{
    fs,
    path::PathBuf,
};
use tauri::AppHandle;

const DEFAULT_CHAIN_ID: &str = "citizenchain";

/// 返回节点数据根目录 `<app_data>/node-data`，不存在时自动创建。
pub(crate) fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = security::app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&path).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(path)
}

/// 扫描 `<node-data>/chains/*/keystore` 目录列表，始终包含默认链 ID 对应的目录。
/// 跳过符号链接，确保 keystore 目录已创建。
pub(crate) fn keystore_dirs(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let chains_root = node_data_dir(app)?.join("chains");
    fs::create_dir_all(&chains_root).map_err(|e| format!("create chains dir failed: {e}"))?;

    let mut dirs: Vec<PathBuf> = Vec::new();
    if chains_root.exists() {
        let entries =
            fs::read_dir(&chains_root).map_err(|e| format!("read chains dir failed: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read chain dir entry failed: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("read chain dir file type failed: {e}"))?;
            if file_type.is_symlink() || !file_type.is_dir() {
                continue;
            }
            let candidate = entry.path().join("keystore");
            if let Ok(meta) = fs::symlink_metadata(&candidate) {
                if meta.file_type().is_symlink() {
                    continue;
                }
            }
            dirs.push(candidate);
        }
    }

    dirs.push(chains_root.join(DEFAULT_CHAIN_ID).join("keystore"));
    dirs.sort();
    dirs.dedup();

    for dir in &dirs {
        fs::create_dir_all(dir).map_err(|e| {
            format!(
                "create keystore dir failed ({}): {e}",
                dir.display()
            )
        })?;
    }

    Ok(dirs)
}

/// 根据密钥类型前缀和公钥生成 keystore 文件名。
pub(crate) fn keystore_filename(key_type_prefix: &str, pubkey_hex: &str) -> String {
    format!("{key_type_prefix}{pubkey_hex}")
}

/// 扫描所有 keystore 目录，返回匹配指定前缀的文件路径列表。
pub(crate) fn scan_keystore_files(
    dirs: &[PathBuf],
    key_type_prefix: &str,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("read keystore dir failed ({}): {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read keystore entry failed: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("read keystore file type failed: {e}"))?;
            if file_type.is_symlink() || !file_type.is_file() {
                continue;
            }
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if name.starts_with(key_type_prefix) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// 将密钥写入所有 keystore 目录，并移除同类型的其他旧密钥文件。
pub(crate) fn write_key_to_keystore(
    dirs: &[PathBuf],
    key_type_prefix: &str,
    pubkey_hex: &str,
    secret_content: &str,
) -> Result<(), String> {
    let filename = keystore_filename(key_type_prefix, pubkey_hex);
    for dir in dirs {
        let path = dir.join(&filename);
        security::write_secret_text_atomic(&path, secret_content).map_err(|e| {
            format!(
                "write keystore file failed ({}): {e}",
                path.display()
            )
        })?;
    }
    remove_other_keys(dirs, key_type_prefix, &filename)?;
    Ok(())
}

/// 移除 keystore 中同类型但不匹配 keep_filename 的旧密钥文件。
pub(crate) fn remove_other_keys(
    dirs: &[PathBuf],
    key_type_prefix: &str,
    keep_filename: &str,
) -> Result<(), String> {
    for path in scan_keystore_files(dirs, key_type_prefix)? {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == keep_filename {
            continue;
        }
        fs::remove_file(&path).map_err(|e| {
            format!(
                "remove stale keystore file failed ({}): {e}",
                path.display()
            )
        })?;
    }
    Ok(())
}

/// 检查指定公钥的 keystore 文件是否存在于任意 keystore 目录中。
pub(crate) fn has_key_in_keystore(
    dirs: &[PathBuf],
    key_type_prefix: &str,
    pubkey_hex: &str,
) -> bool {
    let filename = keystore_filename(key_type_prefix, pubkey_hex);
    dirs.iter().any(|dir| dir.join(&filename).is_file())
}
