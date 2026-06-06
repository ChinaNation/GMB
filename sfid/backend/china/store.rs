//! SQLite 行政区划读取层。

use std::{fs, sync::OnceLock};

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use super::model::{CityCode, ProvinceCode, TownCode};

const CHINA_DB_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/china/data/china.sqlite");

static PROVINCE_CACHE: OnceLock<Vec<ProvinceCode>> = OnceLock::new();

fn leak_text(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn open_china_db() -> Connection {
    Connection::open(CHINA_DB_PATH).expect("open china sqlite database")
}

/// 返回当前行政区划 SQLite 文件哈希。
///
/// 中文注释:该哈希只用于部署期确定性目录完整性校验。编译阶段不会读取数据库,
/// 服务正常启动也不会因为哈希变化自动全量写库。
pub fn china_sqlite_hash() -> Result<String, String> {
    let bytes = fs::read(CHINA_DB_PATH).map_err(|e| format!("read china sqlite failed: {e}"))?;
    let digest = Sha256::digest(bytes);
    Ok(hex::encode(digest))
}

fn load_provinces() -> Vec<ProvinceCode> {
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
    provinces
}

/// 返回全部省份。首次调用从 SQLite 加载并缓存到进程内存。
pub fn provinces() -> &'static [ProvinceCode] {
    PROVINCE_CACHE.get_or_init(load_provinces).as_slice()
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
