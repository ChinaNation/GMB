//! 账户派生金标测试。
//! `ACCOUNT_DERIVE_UPDATE=1` 重写 fixture;默认只断言派生结果不漂移。

use primitives::account_derive::AccountKind;
use primitives::core_const::{GMB, SS58_FORMAT};

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/account_derive_vectors.json"
);

const UPDATE_ENV: &str = "ACCOUNT_DERIVE_UPDATE";

// 极简 hex 编解码。

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

fn hex_decode_32(s: &str) -> [u8; 32] {
    let v = hex_decode(s);
    assert_eq!(v.len(), 32, "期望 32 字节 hex,得到 {} 字节: {s}", v.len());
    let mut out = [0u8; 32];
    out.copy_from_slice(&v);
    out
}

// 用 account_derive 唯一入口派生向量。

/// 解析向量并计算地址。
fn derive_vector(v: &serde_json::Value) -> [u8; 32] {
    let kind = v["kind"].as_str().expect("向量缺少 kind");
    let cid = v.get("cid_number").and_then(|x| x.as_str());
    let name = v.get("account_name").and_then(|x| x.as_str());
    let creator_hex = v.get("creator_hex").and_then(|x| x.as_str());

    match kind {
        "InstitutionMain" => AccountKind::InstitutionMain {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "InstitutionFee" => AccountKind::InstitutionFee {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "InstitutionStake" => AccountKind::InstitutionStake {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "InstitutionSafetyFund" => AccountKind::InstitutionSafetyFund {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "InstitutionHe" => AccountKind::InstitutionHe {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "InstitutionClearing" => AccountKind::InstitutionClearing {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "InstitutionNamed" => AccountKind::InstitutionNamed {
            cid_number: cid.expect("缺 cid_number").as_bytes(),
            account_name: name.expect("缺 account_name").as_bytes(),
        }
        .derive(SS58_FORMAT),
        "Personal" => {
            let creator_account_id = hex_decode_32(creator_hex.expect("Personal 缺 creator_hex"));
            AccountKind::Personal {
                creator_account_id: &creator_account_id,
                account_name: name.expect("缺 account_name").as_bytes(),
            }
            .derive(SS58_FORMAT)
        }
        other => panic!("未知向量 kind: {other}"),
    }
}

fn load_fixture() -> serde_json::Value {
    let raw = std::fs::read_to_string(FIXTURE_PATH)
        .unwrap_or_else(|e| panic!("读取金标 fixture 失败 {FIXTURE_PATH}: {e}"));
    serde_json::from_str(&raw).expect("金标 fixture 不是合法 JSON")
}

/// 查询 china_*.rs 字面地址常量。
fn china_literal(cid: &str, kind: &str) -> Option<[u8; 32]> {
    use primitives::cid::china::china_cb::{CHINA_CB, NRC_HE_ACCOUNT, SAFETY_FUND_ACCOUNT};
    use primitives::cid::china::china_ch::CHINA_CH;
    use primitives::cid::china::citizenchain::CITIZENCHAIN_FOUNDATION;

    match kind {
        "InstitutionMain" => CHINA_CB
            .iter()
            .find(|c| c.cid_number == cid)
            .map(|c| c.main_account)
            .or_else(|| {
                CHINA_CH
                    .iter()
                    .find(|c| c.cid_number == cid)
                    .map(|c| c.main_account)
            })
            .or_else(|| {
                (CITIZENCHAIN_FOUNDATION.cid_number == cid)
                    .then_some(CITIZENCHAIN_FOUNDATION.main_account)
            }),
        "InstitutionFee" => CHINA_CB
            .iter()
            .find(|c| c.cid_number == cid)
            .map(|c| c.fee_account)
            .or_else(|| {
                CHINA_CH
                    .iter()
                    .find(|c| c.cid_number == cid)
                    .map(|c| c.fee_account)
            })
            .or_else(|| {
                (CITIZENCHAIN_FOUNDATION.cid_number == cid)
                    .then_some(CITIZENCHAIN_FOUNDATION.fee_account)
            }),
        "InstitutionStake" => CHINA_CH
            .iter()
            .find(|c| c.cid_number == cid)
            .map(|c| c.stake_account),
        "InstitutionSafetyFund" => Some(SAFETY_FUND_ACCOUNT),
        "InstitutionHe" => Some(NRC_HE_ACCOUNT),
        _ => None,
    }
}

#[test]
fn account_derive_golden_vectors() {
    // fixture 头部必须与链端常量一致。
    let mut fixture = load_fixture();
    assert_eq!(
        fixture["ss58_format"].as_u64(),
        Some(SS58_FORMAT as u64),
        "fixture ss58_format 与链端 SS58_FORMAT 不一致"
    );
    assert_eq!(
        fixture["domain"].as_str(),
        Some(core::str::from_utf8(GMB).unwrap()),
        "fixture domain 与链端 GMB 不一致(域字节变更须同步刷新 fixture)"
    );

    let update = std::env::var(UPDATE_ENV).map(|v| v == "1").unwrap_or(false);

    let vectors = fixture["vectors"]
        .as_array()
        .expect("vectors 必须是数组")
        .clone();
    assert!(!vectors.is_empty(), "fixture 至少需 1 条向量");

    let mut updated = Vec::with_capacity(vectors.len());
    for v in &vectors {
        let computed = derive_vector(v);
        let computed_hex = hex_encode(&computed);
        let kind = v["kind"].as_str().unwrap();
        let cid = v.get("cid_number").and_then(|x| x.as_str()).unwrap_or("");

        // china 来源必须等于源码字面常量。
        if let Some(lit) = china_literal(cid, kind) {
            assert_eq!(
                computed, lit,
                "kind={kind} cid={cid}: account_derive 结果与 china_*.rs 字面常量不一致(行为非中性!)"
            );
        }

        if update {
            let mut nv = v.clone();
            nv["address_hex"] = serde_json::Value::String(computed_hex);
            updated.push(nv);
        } else {
            let expected = v["address_hex"].as_str().unwrap_or("");
            assert!(
                !expected.is_empty(),
                "kind={kind} cid={cid}: fixture address_hex 为空,请先跑 {UPDATE_ENV}=1 回填"
            );
            assert_eq!(
                computed_hex, expected,
                "kind={kind} cid={cid}: account_derive 派生地址与金标 fixture 不一致(地址漂移!)"
            );
        }
    }

    if update {
        fixture["vectors"] = serde_json::Value::Array(updated);
        let pretty = serde_json::to_string_pretty(&fixture).expect("序列化 fixture 失败");
        std::fs::write(FIXTURE_PATH, format!("{pretty}\n"))
            .unwrap_or_else(|e| panic!("写回金标 fixture 失败 {FIXTURE_PATH}: {e}"));
        eprintln!("[account_derive golden] 已用 account_derive 重算并写回 {FIXTURE_PATH}");
    }
}
