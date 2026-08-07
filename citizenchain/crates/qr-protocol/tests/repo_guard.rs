// 仓库结构守卫读取失败必须立即中止测试，断言式解包仅限本测试目标。
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn qr_signing_protocol_has_no_second_registry_or_third_state() {
    let repo_root = repo_root();
    let mut violations = Vec::new();

    // 扫描根必须覆盖**全部**参与 QR/签名协议的端。漏一个端 = 守卫报绿而该端悄悄漂移:
    // 2026-08-06 的单字母键统一就因为这里缺了两个 `frontend/`,导致桌面矿工端
    // 「扫码识别账户」链路断掉而守卫毫无反应。新增任何端必须同步加进本列表。
    for root in [
        "citizenapp/lib",
        "citizenwallet/lib",
        "citizenchain/onchina/src",
        "citizenchain/onchina/frontend",
        "citizenchain/node/src",
        "citizenchain/node/frontend",
        "citizenchain/crates",
    ] {
        collect_files(&repo_root.join(root), &mut |path| {
            if should_scan(path) {
                scan_file(&repo_root, path, &mut violations);
            }
        });
    }

    assert!(
        violations.is_empty(),
        "QR 签名协议 guard 发现第二真源或第三状态残留:\n{}",
        violations.join("\n")
    );
}

/// 两个 Web 前端各持一份 `citizenQr.ts`,必须**逐字节相同**。
///
/// 它们分属两个独立 Vite 应用、无法共享同一物理文件,只能靠本守卫钉死
/// (与 `qr_action_registry.g.dart` 两端各一份、逐字节相同同一范式)。
/// 两份曾各自演化:字段名一样但 SS58 依赖路径不同(`../ss58` vs `../utils/ss58`),
/// 同一处 bug 要修两遍、改一份忘另一份不会有任何提示。
#[test]
fn web_frontend_qr_modules_are_byte_identical() {
    let repo_root = repo_root();
    let node = repo_root.join("citizenchain/node/frontend/shared/qr/citizenQr.ts");
    let onchina = repo_root.join("citizenchain/onchina/frontend/core/citizenQr.ts");

    let node_text =
        fs::read_to_string(&node).unwrap_or_else(|e| panic!("读取 {} 失败: {e}", node.display()));
    let onchina_text = fs::read_to_string(&onchina)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", onchina.display()));

    assert_eq!(
        node_text,
        onchina_text,
        "两个 Web 前端的 citizenQr.ts 必须逐字节相同;改一份必须同步另一份\n  {}\n  {}",
        node.display(),
        onchina.display()
    );
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("qr-protocol 必须位于 citizenchain/crates/qr-protocol")
        .to_path_buf()
}

fn collect_files(dir: &Path, visit: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if should_skip_path(&path) {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, visit);
        } else {
            visit(&path);
        }
    }
}

fn should_skip_path(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.contains("/target/")
        || text.contains("/build/")
        || text.contains("/dist/")
        || text.contains("/.dart_tool/")
        || text.contains("/node_modules/")
        || text.contains("/generated/")
        || text.ends_with(".g.dart")
        || text.contains("/citizenchain/crates/qr-protocol/registry/")
        || text.contains("/citizenchain/crates/qr-protocol/tests/")
        || text.ends_with("/citizenchain/crates/qr-protocol/src/export.rs")
}

fn should_scan(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("dart" | "rs" | "ts" | "tsx" | "js" | "jsx")
    )
}

fn scan_file(repo_root: &Path, path: &Path, violations: &mut Vec<String>) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    let display = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();

    // 移动端只能消费 GeneratedQrActionRegistry,不得恢复手写 action/字段中文表。
    reject_contains(
        &text,
        &display,
        "actionLabels = {",
        "禁止恢复手写 actionLabels 中文表,必须消费 qr-protocol 生成产物",
        violations,
    );
    reject_contains(
        &text,
        &display,
        "actionKeyByCode = {",
        "禁止恢复手写 actionKeyByCode,必须消费 qr-protocol 生成产物",
        violations,
    );
    reject_contains(
        &text,
        &display,
        "actionLabelZhByKey = {",
        "禁止恢复手写 actionLabelZhByKey,必须消费 qr-protocol 生成产物",
        violations,
    );
    reject_contains(
        &text,
        &display,
        "fieldLabelZhByKey = {",
        "禁止恢复手写 fieldLabelZhByKey,必须消费 qr-protocol 生成产物",
        violations,
    );
    reject_contains(
        &text,
        &display,
        "_squareFieldLabels",
        "禁止在广场动作解码器恢复局部字段中文表",
        violations,
    );

    if text.contains("fieldLabelTextOrNull(String key)") && text.contains("return switch (key)") {
        violations.push(format!(
            "{display}: 禁止在 fieldLabelTextOrNull 中恢复手写字段 switch"
        ));
    }

    // OnChina 非链 action code 必须从 registry 读取,不得恢复 1/2/3 硬编码常量。
    for (symbol, value) in [
        ("ACTION_LOGIN", "1"),
        ("ACTION_CITIZEN_IDENTITY", "2"),
        ("ACTION_ONCHINA_ADMIN", "3"),
    ] {
        for (line_no, line) in text.lines().enumerate() {
            if line.contains(symbol)
                && !line.contains("_CODE")
                && (line.contains(&format!("= {value}"))
                    || line.contains(&format!(": u16 = {value}")))
            {
                violations.push(format!(
                    "{display}:{}: 禁止恢复 {symbol} = {value} 硬编码,必须从 qr-protocol registry 读取",
                    line_no + 1
                ));
            }
        }
    }
    reject_contains(
        &text,
        &display,
        "QR_ACTION_ONCHINA_ADMIN",
        "禁止恢复旧 QR_ACTION_ONCHINA_ADMIN 别名",
        violations,
    );
    reject_contains(
        &text,
        &display,
        "QR_ACTION_SQUARE_ACCOUNT",
        "禁止恢复旧 QR_ACTION_SQUARE_ACCOUNT 别名",
        violations,
    );

    // Runtime hash-only 允许列表来自 registry 生成集合,不得在端侧手写 action == A || action == B。
    if text.contains("isRuntimeHashOnly(int action) =>\n      action ==") {
        violations.push(format!(
            "{display}: 禁止恢复手写 hash-only action 列表,必须消费 GeneratedQrActionRegistry.isHashOnlyAction"
        ));
    }

    // 离线签名只允许 normal / reject 两态。
    if text.contains("enum SignDecisionStatus")
        && !text.contains("enum SignDecisionStatus { normal, reject }")
    {
        violations.push(format!(
            "{display}: SignDecisionStatus 只能是 normal/reject 两态"
        ));
    }
    for forbidden in ["decodeFailed", "partialRecognized", "warningButSignable"] {
        if text.contains(forbidden) {
            violations.push(format!(
                "{display}: 禁止恢复签名第三状态或可签名警告态 {forbidden}"
            ));
        }
    }

    // 用户确认页不得把不可理解内容兜底显示给用户后继续签名。
    for (line_no, line) in text.lines().enumerate() {
        if line.contains("载荷 ${") && line.contains("字节") {
            violations.push(format!(
                "{display}:{}: 禁止展示“载荷 N 字节”作为签名确认兜底",
                line_no + 1
            ));
        }
        let displays_numeric_action = (line.contains("动作 ${") || line.contains("动作：${"))
            && (line.contains("body.action")
                || line.contains("actionCode")
                || line.contains("action.toString")
                || line.contains("request.body.action"));
        if displays_numeric_action {
            violations.push(format!(
                "{display}:{}: 禁止展示动作数字作为签名确认兜底",
                line_no + 1
            ));
        }
    }
}

fn reject_contains(
    text: &str,
    display: &str,
    needle: &str,
    reason: &str,
    violations: &mut Vec<String>,
) {
    if text.contains(needle) {
        violations.push(format!("{display}: {reason}"));
    }
}
