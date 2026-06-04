//! SQLite 行政区划读取层。

use std::sync::OnceLock;

use rusqlite::Connection;

use super::model::{CityCode, ProvinceCode};

const CHINA_DB_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/china/data/china.sqlite");

static PROVINCE_CACHE: OnceLock<Vec<ProvinceCode>> = OnceLock::new();

fn leak_text(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn open_china_db() -> Connection {
    Connection::open(CHINA_DB_PATH).expect("open china sqlite database")
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
                Ok(CityCode {
                    code: leak_text(row.get::<_, String>(0)?),
                    name: leak_text(row.get::<_, String>(1)?),
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

pub fn province_name_by_code(code: &str) -> Option<&'static str> {
    provinces()
        .iter()
        .find(|p| p.code.eq_ignore_ascii_case(code))
        .map(|p| p.name)
}
