use crate::registry::{actions, fields, reject_reasons, RegistryError};

/// 导出 registry JSON，供后续生成 Dart/TypeScript 产物。
///
/// JSON 主要用于人工审计；移动端实际消费的 Dart 常量由
/// [export_registry_dart] 生成，避免 App / Wallet 各自维护第二套动作表。
pub fn export_registry_json() -> Result<String, RegistryError> {
    let value = serde_json::json!({
        "actions": actions()?,
        "fields": fields()?,
        "reject_reasons": reject_reasons()?,
    });
    Ok(serde_json::to_string_pretty(&value)?)
}

/// 导出 Dart registry 生成文件。
///
/// 该产物是 CitizenApp / CitizenWallet 的唯一扫码动作与中文字段表来源：
/// 两端 UI 样式可以不同，但动作码、action_key、中文动作名、字段中文名必须逐字节一致。
pub fn export_registry_dart() -> Result<String, RegistryError> {
    let mut actions = actions()?;
    actions.sort_by_key(|action| action.action_code);

    let mut fields = fields()?;
    fields.sort_by(|left, right| left.field_key.cmp(&right.field_key));

    let mut reject_reasons = reject_reasons()?;
    reject_reasons.sort_by(|left, right| left.reject_reason_key.cmp(&right.reject_reason_key));

    let mut out = String::new();
    out.push_str("// 本文件由 citizenchain/crates/qr-protocol 生成，禁止手改。\n");
    out.push_str(
        "// 扫码签名动作、中文动作名、字段中文名和固定展示值的唯一真源在 registry/*.yaml。\n\n",
    );
    out.push_str("class GeneratedQrActionRegistry {\n");
    out.push_str("  const GeneratedQrActionRegistry._();\n\n");

    out.push_str("  static const Map<int, String> actionKeyByCode = {\n");
    for action in &actions {
        out.push_str(&format!(
            "    {}: {},\n",
            dart_action_code(action.action_code),
            dart_string(&action.action_key)
        ));
    }
    out.push_str("  };\n\n");

    out.push_str("  static const Map<String, int> actionCodeByKey = {\n");
    for action in &actions {
        out.push_str(&format!(
            "    {}: {},\n",
            dart_string(&action.action_key),
            dart_action_code(action.action_code)
        ));
    }
    out.push_str("  };\n\n");

    out.push_str("  static const Map<String, String> actionLabelZhByKey = {\n");
    for action in &actions {
        out.push_str(&format!(
            "    {}: {},\n",
            dart_string(&action.action_key),
            dart_string(&action.action_label_zh)
        ));
    }
    out.push_str("  };\n\n");

    out.push_str("  static const Map<String, String> fieldLabelZhByKey = {\n");
    for field in &fields {
        out.push_str(&format!(
            "    {}: {},\n",
            dart_string(&field.field_key),
            dart_string(&field.field_label_zh)
        ));
    }
    out.push_str("  };\n\n");

    out.push_str("  static const Map<String, String> fieldValueZhByKey = {\n");
    for field in fields.iter().filter(|field| field.field_value_zh.is_some()) {
        out.push_str(&format!(
            "    {}: {},\n",
            dart_string(&field.field_key),
            dart_string(field.field_value_zh.as_ref().expect("checked above"))
        ));
    }
    out.push_str("  };\n\n");

    out.push_str("  static const Map<String, String> rejectReasonZhByKey = {\n");
    for reason in &reject_reasons {
        out.push_str(&format!(
            "    {}: {},\n",
            dart_string(&reason.reject_reason_key),
            dart_string(&reason.reject_reason_zh)
        ));
    }
    out.push_str("  };\n\n");

    out.push_str("  static const Set<int> hashOnlyActionCodes = {\n");
    for action in actions.iter().filter(|action| action.hash_only_allowed) {
        out.push_str(&format!("    {},\n", dart_action_code(action.action_code)));
    }
    out.push_str("  };\n\n");

    out.push_str("  static String? actionKeyForCode(int actionCode) =>\n");
    out.push_str("      actionKeyByCode[actionCode];\n\n");
    out.push_str(
        "  static int? actionCodeForKey(String actionKey) => actionCodeByKey[actionKey];\n\n",
    );
    out.push_str("  static String? actionLabelForKey(String actionKey) =>\n");
    out.push_str("      actionLabelZhByKey[actionKey];\n\n");
    out.push_str("  static String? actionLabelForCode(int actionCode) {\n");
    out.push_str("    final key = actionKeyForCode(actionCode);\n");
    out.push_str("    if (key == null) return null;\n");
    out.push_str("    return actionLabelForKey(key);\n");
    out.push_str("  }\n\n");
    out.push_str("  static String? fieldLabelForKey(String fieldKey) =>\n");
    out.push_str("      fieldLabelZhByKey[fieldKey];\n\n");
    out.push_str("  static bool hasFieldLabel(String fieldKey) =>\n");
    out.push_str("      fieldLabelForKey(fieldKey) != null;\n\n");
    out.push_str("  static String? fieldValueForKey(\n");
    out.push_str("    String fieldKey,\n");
    out.push_str("    Map<String, String> values,\n");
    out.push_str("  ) {\n");
    out.push_str("    var template = fieldValueZhByKey[fieldKey];\n");
    out.push_str("    if (template == null) return null;\n");
    out.push_str("    for (final entry in values.entries) {\n");
    out.push_str("      template = template!.replaceAll('{${entry.key}}', entry.value);\n");
    out.push_str("    }\n");
    out.push_str("    return template;\n");
    out.push_str("  }\n\n");
    out.push_str("  static String? rejectReasonForKey(String reasonKey) =>\n");
    out.push_str("      rejectReasonZhByKey[reasonKey];\n\n");
    out.push_str("  static bool isHashOnlyAction(int actionCode) =>\n");
    out.push_str("      hashOnlyActionCodes.contains(actionCode);\n");
    out.push_str("}\n");

    Ok(out)
}

fn dart_action_code(action_code: u16) -> String {
    if action_code >= 0x0100 {
        format!("0x{action_code:04x}")
    } else {
        action_code.to_string()
    }
}

fn dart_string(value: &str) -> String {
    let mut out = String::from("'");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '$' => out.push_str("\\$"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('\'');
    out
}
