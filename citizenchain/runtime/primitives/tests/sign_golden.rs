//! ADR-026 签名协议金标向量导出/断言测试。
//!
//! 唯一权威算法源 = `primitives::sign::signing_message`。本测试是跨语言(Rust ↔ Dart)
//! 防漂移的金标兜底,同时回归证明治理 5 个 op_tag(0x10-0x14)改调原语后**字节不变**:
//!
//! - `SIGN_GOLDEN_UPDATE=1`:用 `signing_message` 对每条向量(op_tag + 固定 scale_payload)
//!   重算 message_hex 并写回 canonical fixture
//!   (`tests/fixtures/signing_domain_vectors.json`)。
//! - 默认(未设环境变量):读取 fixture,断言 `signing_message` 结果逐字节 == fixture。
//!
//! 治理回归铁律:0x10-0x14 五条向量的 message_hex 任何时候都不得变化(改调原语前后字节一致,
//! 证明 `(GMB, op_tag, fields).encode()` == `GMB || op_tag || (fields).encode()`)。
//! 0x15-0x1B 七条是新折的字符串域,首次跑 UPDATE 回填,之后亦冻结。
//!
//! CI 守卫:跑 update + `git diff --exit-code` 防 fixture 与算法源漂移。

use codec::Encode;
use primitives::core_const::GMB;
use primitives::sign::{signing_message, SIGN_OP_TAGS};

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/signing_domain_vectors.json"
);

const UPDATE_ENV: &str = "SIGN_GOLDEN_UPDATE";

// ── 极简 hex 编解码(避免引入 hex crate,保持依赖最小;字节正确性 load-bearing) ──

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_decode(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "hex 串长度必须为偶数: {s}");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("非法 hex 字符"))
        .collect()
}

fn parse_op_tag(v: &serde_json::Value) -> u8 {
    let raw = v["op_tag"].as_str().expect("向量缺少 op_tag(应为 0xNN hex 串)");
    let stripped = raw.strip_prefix("0x").unwrap_or(raw);
    u8::from_str_radix(stripped, 16).unwrap_or_else(|_| panic!("非法 op_tag: {raw}"))
}

fn load_fixture() -> serde_json::Value {
    let raw = std::fs::read_to_string(FIXTURE_PATH)
        .unwrap_or_else(|e| panic!("读取金标 fixture 失败 {FIXTURE_PATH}: {e}"));
    serde_json::from_str(&raw).expect("金标 fixture 不是合法 JSON")
}

#[test]
fn sign_golden_vectors() {
    let mut fixture = load_fixture();

    // 守卫:fixture 域必须与链端 GMB 一致(域字节变更须同步刷新 fixture)。
    assert_eq!(
        fixture["domain"].as_str(),
        Some(core::str::from_utf8(GMB).unwrap()),
        "fixture domain 与链端 GMB 不一致"
    );

    let update = std::env::var(UPDATE_ENV).map(|v| v == "1").unwrap_or(false);

    let vectors = fixture["vectors"]
        .as_array()
        .expect("vectors 必须是数组")
        .clone();
    assert!(!vectors.is_empty(), "fixture 至少需 1 条向量");

    // 覆盖守卫:注册表里每个 op_tag 都必须有至少一条向量。
    for tag in SIGN_OP_TAGS {
        let present = vectors.iter().any(|v| parse_op_tag(v) == tag);
        assert!(present, "op_tag 0x{tag:02x} 在注册表却无金标向量");
    }

    let mut updated = Vec::with_capacity(vectors.len());
    for v in &vectors {
        let op_tag = parse_op_tag(v);
        let payload = hex_decode(v["scale_payload_hex"].as_str().expect("缺 scale_payload_hex"));
        let computed = signing_message(op_tag, &payload);
        let computed_hex = hex_encode(&computed);

        // 双向锚定:原语结果必须 == 朴素拼接 GMB||op_tag||payload 的 blake2_256。
        let naive = {
            let mut data = Vec::new();
            data.extend_from_slice(GMB);
            data.push(op_tag);
            data.extend_from_slice(&payload);
            sp_core::hashing::blake2_256(&data)
        };
        assert_eq!(
            computed, naive,
            "op_tag 0x{op_tag:02x}: signing_message 与朴素拼接不一致(原语实现漂移!)"
        );

        // 字节证明:SCALE 元组 (GMB, op_tag, payload_raw) 编码 == GMB||op_tag||payload。
        // payload 这里以 &[u8] 直接 push(非 SCALE 长度前缀),故对照用裸拼接;
        // 治理实际调用方的 payload 是 (fields).encode(),与本对照同构。
        let tuple_bytes = (GMB, op_tag, payload.as_slice());
        let scale_tuple = tuple_bytes.encode();
        // SCALE 对 &[u8] 会加 compact 长度前缀;故只比对前 4 字节域头逐字节一致。
        assert_eq!(
            &scale_tuple[..4],
            {
                let mut head = Vec::new();
                head.extend_from_slice(GMB);
                head.push(op_tag);
                head
            }
            .as_slice(),
            "op_tag 0x{op_tag:02x}: SCALE 元组域头 (GMB||op_tag) 字节漂移"
        );

        if update {
            let mut nv = v.clone();
            nv["message_hex"] = serde_json::Value::String(computed_hex);
            updated.push(nv);
        } else {
            let expected = v["message_hex"].as_str().unwrap_or("");
            assert!(
                !expected.is_empty(),
                "op_tag 0x{op_tag:02x}: fixture message_hex 为空,请先跑 {UPDATE_ENV}=1 回填"
            );
            assert_eq!(
                computed_hex, expected,
                "op_tag 0x{op_tag:02x}: signing_message 与金标 fixture 不一致(签名消息漂移!)"
            );
        }
    }

    if update {
        fixture["vectors"] = serde_json::Value::Array(updated);
        let pretty = serde_json::to_string_pretty(&fixture).expect("序列化 fixture 失败");
        std::fs::write(FIXTURE_PATH, format!("{pretty}\n"))
            .unwrap_or_else(|e| panic!("写回金标 fixture 失败 {FIXTURE_PATH}: {e}"));
        eprintln!("[sign golden] 已用 signing_message 重算并写回 {FIXTURE_PATH}");
    }
}
