//! SQLite 行政区划只读层。

use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};

use super::model::{CityCode, ProvinceCode, TownCode};

const CHINA_DB_DEV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/china/china.sqlite");

static CHINA_DB_PATH: OnceLock<PathBuf> = OnceLock::new();
static CHINA_SQLITE_HASH_CACHE: OnceLock<String> = OnceLock::new();
static PROVINCE_CACHE: OnceLock<&'static [ProvinceCode]> = OnceLock::new();

fn leak_text(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn china_db_path() -> &'static Path {
    CHINA_DB_PATH.get_or_init(|| {
        // 中文注释:行政区以开发库 `citizencode/backend/china/china.sqlite` 为权威源。
        // 生产环境只允许通过 CID_CHINA_DB 指向随包只读 SQLite,不得在运行中复制或改写。
        if let Some(raw) = std::env::var("CID_CHINA_DB")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return PathBuf::from(raw);
        }
        let dev = PathBuf::from(CHINA_DB_DEV_PATH);
        if dev.exists() {
            return dev;
        }
        let exe = std::env::current_exe().expect("resolve cid backend executable path");
        exe.parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("/opt/citizencode"))
            .join("china/china.sqlite")
    })
}

fn open_china_db() -> Connection {
    Connection::open_with_flags(china_db_path(), OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open read-only china sqlite database")
}

/// 返回当前行政区划 SQLite 文件哈希。
///
/// 中文注释:该哈希只用于部署期确定性目录完整性校验。运行时只读打开数据库,
/// 不会因为哈希变化自动写库或发布新版本。
pub fn china_sqlite_hash() -> Result<String, String> {
    // 中文注释:china_db_path 本身已在进程内固定,部署期完整性哈希也随之缓存,
    // 避免 gov changed-only 逐省检查时重复读取并哈希同一个只读 SQLite 文件。
    if let Some(hash) = CHINA_SQLITE_HASH_CACHE.get() {
        return Ok(hash.clone());
    }
    let bytes = fs::read(china_db_path()).map_err(|e| format!("read china sqlite failed: {e}"))?;
    let digest = Sha256::digest(bytes);
    let hash = hex::encode(digest);
    let _ = CHINA_SQLITE_HASH_CACHE.set(hash.clone());
    Ok(CHINA_SQLITE_HASH_CACHE.get().cloned().unwrap_or(hash))
}

fn load_provinces() -> &'static [ProvinceCode] {
    let conn = open_china_db();
    let mut province_stmt = conn
        .prepare("SELECT code, name FROM provinces ORDER BY sort_order")
        .expect("prepare province query");
    let province_rows = province_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .expect("query provinces");

    let mut provinces = Vec::new();
    for row in province_rows {
        let (province_code, province_name) = row.expect("read province row");
        let mut city_stmt = conn
            .prepare("SELECT code, name FROM cities WHERE province_code = ?1 ORDER BY sort_order")
            .expect("prepare city query");
        let cities: Vec<CityCode> = city_stmt
            .query_map([province_code.as_str()], |row| {
                let city_code: String = row.get(0)?;
                let mut town_stmt = conn
                    .prepare(
                        "SELECT code, name FROM towns
                         WHERE province_code = ?1 AND city_code = ?2
                         ORDER BY sort_order",
                    )
                    .expect("prepare town query");
                let towns: Vec<TownCode> = town_stmt
                    .query_map([province_code.as_str(), city_code.as_str()], |town_row| {
                        Ok(TownCode {
                            code: leak_text(town_row.get::<_, String>(0)?),
                            name: leak_text(town_row.get::<_, String>(1)?),
                        })
                    })
                    .expect("query towns")
                    .map(|town_row| town_row.expect("read town row"))
                    .collect();
                Ok(CityCode {
                    code: leak_text(city_code),
                    name: leak_text(row.get::<_, String>(1)?),
                    towns: Box::leak(towns.into_boxed_slice()),
                })
            })
            .expect("query cities")
            .map(|row| row.expect("read city row"))
            .collect();

        provinces.push(ProvinceCode {
            code: leak_text(province_code),
            name: leak_text(province_name),
            cities: Box::leak(cities.into_boxed_slice()),
        });
    }

    // 铁律(ADR-021):省名、城市名全国唯一；市/镇 code 不可变且不可复用。
    // 这里在服务启动读取时同步校验,让错误数据立即暴露。
    let mut province_names = std::collections::HashSet::new();
    let mut city_names = std::collections::HashSet::new();
    let mut town_triples = std::collections::HashSet::new();
    for p in &provinces {
        assert!(
            province_names.insert(p.name),
            "china.sqlite 省名重复(违反 ADR-021 全国唯一铁律): {}",
            p.name
        );
        for c in p.cities {
            assert!(
                city_names.insert(c.name),
                "china.sqlite 市名重复(违反 ADR-021 全国唯一铁律): {}",
                c.name
            );
            for t in c.towns {
                assert!(
                    town_triples.insert((p.code, c.code, t.code)),
                    "china.sqlite 行政区 code 重复(违反 ADR-021 不可变不复用铁律): {}/{}/{}",
                    p.code,
                    c.code,
                    t.code
                );
            }
        }
    }
    Box::leak(provinces.into_boxed_slice())
}

/// 返回全部省份。首次调用从只读 SQLite 加载并缓存到进程内存。
pub fn provinces() -> &'static [ProvinceCode] {
    PROVINCE_CACHE.get_or_init(load_provinces)
}

pub fn province_code_by_name(name: &str) -> Option<&'static str> {
    provinces().iter().find(|p| p.name == name).map(|p| p.code)
}

pub fn city_code_by_name(province_name: &str, city_name: &str) -> Option<&'static str> {
    let p = provinces().iter().find(|p| p.name == province_name)?;
    p.cities
        .iter()
        .find(|c| c.name == city_name)
        .map(|c| c.code)
}

/// 中文注释:镇目录创建/对账入口需要按名称反查镇代码,当前查询链路暂未直接调用。
#[allow(dead_code)]
pub fn town_code_by_name(
    province_name: &str,
    city_name: &str,
    town_name: &str,
) -> Option<&'static str> {
    let p = provinces().iter().find(|p| p.name == province_name)?;
    let c = p.cities.iter().find(|c| c.name == city_name)?;
    c.towns.iter().find(|t| t.name == town_name).map(|t| t.code)
}

/// 中文注释:镇目录详情和后续导入工具需要按代码还原行政区名称,当前查询链路暂未直接调用。
#[allow(dead_code)]
pub fn area_name_by_codes(
    province_code: &str,
    city_code: Option<&str>,
    town_code: Option<&str>,
) -> Option<(&'static str, Option<&'static str>, Option<&'static str>)> {
    let province = provinces()
        .iter()
        .find(|p| p.code.eq_ignore_ascii_case(province_code))?;
    let city = city_code
        .filter(|code| !code.is_empty() && *code != "000")
        .and_then(|code| {
            province
                .cities
                .iter()
                .find(|c| c.code.eq_ignore_ascii_case(code))
        });
    let town = match (city, town_code) {
        (Some(city), Some(code)) if !code.is_empty() => city
            .towns
            .iter()
            .find(|t| t.code.eq_ignore_ascii_case(code))
            .map(|t| t.name),
        _ => None,
    };
    Some((province.name, city.map(|c| c.name), town))
}

pub fn province_name_by_code(code: &str) -> Option<&'static str> {
    provinces()
        .iter()
        .find(|p| p.code.eq_ignore_ascii_case(code))
        .map(|p| p.name)
}

/// 判定 (省,市,镇) 三元组是否对应当前行政区划真源里的一个活镇。
///
/// 中文注释:孤儿机构清理(ADR-021 §B5)的唯一判定依据。被删/退役复用 code 下的旧机构
/// 其 `town_code` 会指向一个已不存在于 china.sqlite 的镇。空 `tc` 永远返回 true(市级机构、
/// 储委会、部委等合法态没有镇维度),由调用方负责跳过空 town_code 行;此处也对空 tc 直接判存在以防误删。
pub fn town_exists(pc: &str, cc: &str, tc: &str) -> bool {
    if tc.trim().is_empty() {
        return true;
    }
    let Some(province) = provinces().iter().find(|p| p.code.eq_ignore_ascii_case(pc)) else {
        return false;
    };
    let Some(city) = province
        .cities
        .iter()
        .find(|c| c.code.eq_ignore_ascii_case(cc))
    else {
        return false;
    };
    city.towns.iter().any(|t| t.code.eq_ignore_ascii_case(tc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_town_triple() -> (&'static str, &'static str, &'static str) {
        for p in provinces() {
            for c in p.cities {
                if let Some(t) = c.towns.first() {
                    return (p.code, c.code, t.code);
                }
            }
        }
        panic!("china.sqlite 至少应有一个镇供测试");
    }

    #[test]
    fn town_exists_true_for_real_triple() {
        let (pc, cc, tc) = first_town_triple();
        assert!(town_exists(pc, cc, tc));
    }

    #[test]
    fn town_exists_false_for_retired_town_code() {
        let (pc, cc, _tc) = first_town_triple();
        // 中文注释:同省同市但镇 code 不存在(模拟退役/被删镇),应判孤儿。
        assert!(!town_exists(pc, cc, "ZZZ_NOT_A_TOWN"));
    }

    #[test]
    fn town_exists_false_for_unknown_province() {
        assert!(!town_exists("__NOPE__", "001", "001"));
    }

    #[test]
    fn town_exists_true_for_empty_town_code() {
        // 中文注释:空镇 code = 市级机构/储委会/部委合法态,永远不是孤儿。
        assert!(town_exists("ZS", "001", ""));
        assert!(town_exists("ZS", "", ""));
        assert!(town_exists("anything", "anything", "   "));
    }
}
