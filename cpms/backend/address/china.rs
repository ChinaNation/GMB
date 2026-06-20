//! 行政区只读读取层。
//!
//! 中文注释:行政区唯一源是 SFID 维护的 `china.sqlite`;CPMS 安装包随附其只读拷贝,
//! 不在 CPMS 侧保存或维护第二套行政区源。路径优先环境变量 `CPMS_CHINA_DB`,
//! 默认二进制同目录 `./china.sqlite`,与 `CPMS_FRONTEND_DIR` 的约定一致。
//!
//! 本模块只按安装码所属省市做窄查询,绝不把全国镇下地址段全量载入内存。

use std::env;

use rusqlite::{Connection, OpenFlags, OptionalExtension};

/// 安装码所属省市的名称还原。
pub(crate) struct CityArea {
    pub province_name: String,
    pub city_name: String,
}

/// 单个省。
pub(crate) struct ProvinceArea {
    pub code: String,
    pub name: String,
}

/// 单个市。
pub(crate) struct CityCodeArea {
    pub code: String,
    pub name: String,
}

/// 单个镇及其下辖地址段。
pub(crate) struct TownArea {
    pub code: String,
    pub name: String,
    pub address_units: Vec<AddressUnitArea>,
}

/// 单个地址段。中文注释:地址段是镇下面的地名段,不是行政区 code。
pub(crate) struct AddressUnitArea {
    pub id: String,
    pub name: String,
}

/// 解析 china.sqlite 路径，三层兜底。
///
/// 中文注释：
/// 1) 环境变量 `CPMS_CHINA_DB`——生产由 install_host 写入 `/opt/cpms/data/china.sqlite`，
///    dev 也可手动覆盖；设了就信任原值（设错即 fail-loud）。
/// 2) 二进制旁 `<exe_dir>/../data/china.sqlite`——部署自定位（`/opt/cpms/bin` → `/opt/cpms/data`），
///    即使 env 丢失也能靠自身位置找到随附拷贝。
/// 3) 编译期 `CARGO_MANIFEST_DIR` 相对的 SFID 唯一源——本地 dev `cargo run` 零配置即通，
///    镜像 SFID `china::store` 的定位约定。该路径仅作 dev 兜底，生产被前两层覆盖。
fn china_db_path() -> String {
    if let Ok(p) = env::var("CPMS_CHINA_DB") {
        if !p.trim().is_empty() {
            return p;
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(install_root) = exe.parent().and_then(|dir| dir.parent()) {
            let beside = install_root.join("data").join("china.sqlite");
            if beside.is_file() {
                return beside.to_string_lossy().into_owned();
            }
        }
    }
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../sfid/backend/china/china.sqlite"
    )
    .to_string()
}

fn open() -> Result<Connection, String> {
    let path = china_db_path();
    Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("open china sqlite {path} failed: {e}"))
}

/// 读取全国省份。中文注释：出生地选择只读 CPMS 随包的 SFID 行政区唯一真源拷贝。
pub(crate) fn provinces() -> Result<Vec<ProvinceArea>, String> {
    let conn = open()?;
    let mut stmt = conn
        .prepare("SELECT code, name FROM provinces ORDER BY sort_order")
        .map_err(|e| format!("prepare china provinces failed: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ProvinceArea {
                code: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| format!("query china provinces failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read china province row failed: {e}"))?;
    Ok(rows)
}

/// 读取某省全部市。
pub(crate) fn cities(province_code: &str) -> Result<Vec<CityCodeArea>, String> {
    let conn = open()?;
    let mut stmt = conn
        .prepare(
            "SELECT code, name FROM cities
             WHERE province_code = ?1
             ORDER BY sort_order",
        )
        .map_err(|e| format!("prepare china cities failed: {e}"))?;
    let rows = stmt
        .query_map([province_code.trim()], |row| {
            Ok(CityCodeArea {
                code: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| format!("query china cities failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read china city row failed: {e}"))?;
    Ok(rows)
}

/// 读取某市全部镇。出生地不需要镇下地址段数据。
pub(crate) fn towns(province_code: &str, city_code: &str) -> Result<Vec<TownArea>, String> {
    let conn = open()?;
    let mut stmt = conn
        .prepare(
            "SELECT code, name FROM towns
             WHERE province_code = ?1 AND city_code = ?2
             ORDER BY sort_order",
        )
        .map_err(|e| format!("prepare china towns failed: {e}"))?;
    let rows = stmt
        .query_map([province_code.trim(), city_code.trim()], |row| {
            Ok(TownArea {
                code: row.get(0)?,
                name: row.get(1)?,
                address_units: Vec::new(),
            })
        })
        .map_err(|e| format!("query china towns failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read china town row failed: {e}"))?;
    Ok(rows)
}

/// 按省市代码还原省市名称;省或市不存在返回 `Ok(None)`。
pub(crate) fn find_city(province_code: &str, city_code: &str) -> Result<Option<CityArea>, String> {
    let conn = open()?;
    let province_name: Option<String> = conn
        .query_row(
            "SELECT name FROM provinces WHERE code = ?1",
            [province_code],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("query china province {province_code} failed: {e}"))?;
    let Some(province_name) = province_name else {
        return Ok(None);
    };
    let city_name: Option<String> = conn
        .query_row(
            "SELECT name FROM cities WHERE province_code = ?1 AND code = ?2",
            [province_code, city_code],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("query china city {province_code}{city_code} failed: {e}"))?;
    let Some(city_name) = city_name else {
        return Ok(None);
    };
    Ok(Some(CityArea {
        province_name,
        city_name,
    }))
}

pub(crate) fn find_town(
    province_code: &str,
    city_code: &str,
    town_code: &str,
) -> Result<bool, String> {
    let conn = open()?;
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM towns
             WHERE province_code = ?1 AND city_code = ?2 AND code = ?3",
            [province_code.trim(), city_code.trim(), town_code.trim()],
            |row| row.get(0),
        )
        .map_err(|e| {
            format!(
                "query china town {}{}{} failed: {e}",
                province_code.trim(),
                city_code.trim(),
                town_code.trim()
            )
        })?;
    Ok(exists > 0)
}

/// 取该市全部镇及其下辖地址段。单查镇 + 单查全市地址段后按 town_code 归组,避免逐镇 N+1。
pub(crate) fn city_towns_with_address_units(
    province_code: &str,
    city_code: &str,
) -> Result<Vec<TownArea>, String> {
    let conn = open()?;

    let mut town_stmt = conn
        .prepare(
            "SELECT code, name FROM towns
             WHERE province_code = ?1 AND city_code = ?2
             ORDER BY sort_order",
        )
        .map_err(|e| format!("prepare china towns failed: {e}"))?;
    let mut towns: Vec<TownArea> = town_stmt
        .query_map([province_code, city_code], |row| {
            Ok(TownArea {
                code: row.get::<_, String>(0)?,
                name: row.get::<_, String>(1)?,
                address_units: Vec::new(),
            })
        })
        .map_err(|e| format!("query china towns failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read china town row failed: {e}"))?;

    let mut address_unit_stmt = conn
        .prepare(
            "SELECT town_code, address_unit_id, name FROM address_units
             WHERE province_code = ?1 AND city_code = ?2
             ORDER BY town_code, sort_order",
        )
        .map_err(|e| format!("prepare china address_units failed: {e}"))?;
    let address_units = address_unit_stmt
        .query_map([province_code, city_code], |row| {
            Ok((
                row.get::<_, String>(0)?,
                AddressUnitArea {
                    id: row.get::<_, String>(1)?,
                    name: row.get::<_, String>(2)?,
                },
            ))
        })
        .map_err(|e| format!("query china address_units failed: {e}"))?;

    let town_index: std::collections::HashMap<String, usize> = towns
        .iter()
        .enumerate()
        .map(|(i, t)| (t.code.clone(), i))
        .collect();
    for address_unit in address_units {
        let (town_code, address_unit) =
            address_unit.map_err(|e| format!("read china address_unit row failed: {e}"))?;
        if let Some(&i) = town_index.get(&town_code) {
            towns[i].address_units.push(address_unit);
        }
    }

    Ok(towns)
}
