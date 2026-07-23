use qr_protocol::export::export_registry_dart;
use qr_protocol::registry::{actions, fields, reject_reasons, SigningCategory};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const REMOVED_AMBIGUOUS_ACCOUNT_FIELDS: &[&str] = &[
    "wallet_account",
    "admin_account",
    "owner_account",
    "signer_pubkey",
    "credential_signer_pubkey",
    "actor_pubkey",
    "admin_pubkey",
    "operator_account",
    "target_account",
    "institution_account",
    "personal_account",
    "execution_account",
    "funding_account",
    "operation_fee_payer",
    "execution_fee_payer",
    "fee_payer",
    "beneficiary",
    "from",
    "to",
    "who",
    "bank_main",
    "new_bank",
    "account",
    "address",
];

#[test]
fn actions_have_unique_keys_codes_and_chinese_labels() {
    let actions = actions().expect("actions.yaml 必须可解析");
    let mut keys = HashSet::new();
    let mut codes = HashSet::new();

    for action in actions {
        assert!(
            keys.insert(action.action_key.clone()),
            "action_key 重复: {}",
            action.action_key
        );
        assert!(
            codes.insert(action.action_code),
            "action_code 重复: 0x{:04x}",
            action.action_code
        );
        assert!(
            !action.action_label_zh.trim().is_empty(),
            "{} 缺少中文动作名",
            action.action_key
        );
        assert!(
            !action.decoder.trim().is_empty(),
            "{} 缺少 decoder",
            action.action_key
        );
        assert_eq!(
            action.qr_kind, "sign_request",
            "{} 不能新增登录专用或业务专用 QR kind",
            action.action_key
        );
    }
}

#[test]
fn required_fields_all_have_chinese_labels() {
    let actions = actions().expect("actions.yaml 必须可解析");
    let fields = fields().expect("fields.yaml 必须可解析");
    let field_keys: HashSet<_> = fields
        .iter()
        .map(|field| field.field_key.as_str())
        .collect();

    for action in actions {
        for field_key in action.required_fields {
            assert!(
                field_keys.contains(field_key.as_str()),
                "{} required_fields 缺少中文字段登记: {}",
                action.action_key,
                field_key
            );
        }
    }
}

#[test]
fn field_and_reject_reason_keys_are_unique_and_chinese() {
    let fields = fields().expect("fields.yaml 必须可解析");
    let mut field_keys = HashSet::new();
    for field in fields {
        assert!(
            field_keys.insert(field.field_key.clone()),
            "field_key 重复: {}",
            field.field_key
        );
        assert!(
            !field.field_label_zh.trim().is_empty(),
            "{} 缺少中文字段名",
            field.field_key
        );
    }

    let reasons = reject_reasons().expect("reject_reasons.yaml 必须可解析");
    let mut reason_keys = HashSet::new();
    for reason in reasons {
        assert!(
            reason_keys.insert(reason.reject_reason_key.clone()),
            "reject_reason_key 重复: {}",
            reason.reject_reason_key
        );
        assert!(
            !reason.reject_reason_zh.trim().is_empty(),
            "{} 缺少中文拒绝原因",
            reason.reject_reason_key
        );
    }
}

#[test]
fn removed_ambiguous_account_fields_cannot_return() {
    let actions = actions().expect("actions.yaml 必须可解析");
    for action in actions {
        for field_key in action.required_fields {
            assert!(
                !REMOVED_AMBIGUOUS_ACCOUNT_FIELDS.contains(&field_key.as_str()),
                "{} required_fields 恢复了已删除的含糊账户字段: {}",
                action.action_key,
                field_key
            );
        }
    }

    let fields = fields().expect("fields.yaml 必须可解析");
    for field in fields {
        assert!(
            !REMOVED_AMBIGUOUS_ACCOUNT_FIELDS.contains(&field.field_key.as_str()),
            "fields.yaml 恢复了已删除的含糊账户字段: {}",
            field.field_key
        );
    }
}

#[test]
fn hash_only_is_limited_to_runtime_upgrade() {
    let actions = actions().expect("actions.yaml 必须可解析");
    for action in actions {
        if action.hash_only_allowed {
            assert_eq!(
                action.signing_category,
                SigningCategory::RuntimeUpgrade,
                "{} 只有 Runtime 升级允许 hash-only",
                action.action_key
            );
            assert!(
                action.action_key.contains("runtime_upgrade")
                    || action.action_key == "developer_direct_upgrade",
                "{} 不是 Runtime 升级动作,不能 hash-only",
                action.action_key
            );
        }
    }
}

#[test]
fn generated_dart_registries_are_current() {
    let expected = export_registry_dart().expect("Dart registry 必须可生成");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("qr-protocol 必须位于 citizenchain/crates/qr-protocol");

    for path in [
        repo_root.join("citizenapp/lib/qr/generated/qr_action_registry.g.dart"),
        repo_root.join("citizenwallet/lib/qr/generated/qr_action_registry.g.dart"),
    ] {
        let actual = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} 读取失败: {error}", path.display()));
        assert_eq!(
            actual,
            expected,
            "{} 与 qr-protocol registry 不一致,请重新运行 export_registry --dart",
            path.display()
        );
    }
}
