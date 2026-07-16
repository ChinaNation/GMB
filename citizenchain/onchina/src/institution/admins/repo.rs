//! 机构管理员链下私密资料仓储层(CRUD + 按 cid/账户查 + scope 过滤)。
//!
//! 仓储只读写 `institution_admins` 省级分区表。行政区名字不入库,
//! 读出后由本层按 china.sqlite 派生省/市名字回填,供 scope 过滤与 DTO 展示。
//! scope 过滤复用统一入口 `scope::get_visible_scope` / `scope::filter_by_scope`。

#![allow(dead_code)]

use crate::cid::china::area_display_names;
use crate::core::db::Db;
use crate::institution::admins::model::InstitutionAdmin;
use crate::scope::{filter_by_scope, rules::VisibleScope};

const SELECT_COLUMNS: &str = "cid_number, province_code, city_code, admin_account,
    admin_department, admin_job, admin_contact_phone, admin_contact_email,
    admin_photo_path, admin_photo_name, admin_photo_mime, admin_photo_size,
    admin_passkey_credential_id, admin_source_id, admin_status,
    admin_updated_at, created_by, operation_log_id, created_at";

fn row_to_admin(row: &postgres::Row) -> InstitutionAdmin {
    let province_code: String = row.get(1);
    let city_code: Option<String> = row.get(2);
    // 名字按 code 现场派生(单源 china.sqlite),库里不存名字副本。
    let (province_name, city_name, _town_name) =
        area_display_names(province_code.as_str(), city_code.as_deref(), None);
    let photo_size: Option<i64> = row.get(11);
    InstitutionAdmin {
        cid_number: row.get(0),
        province_code,
        city_code,
        admin_account: row.get(3),
        admin_department: row.get(4),
        admin_job: row.get(5),
        admin_contact_phone: row.get(6),
        admin_contact_email: row.get(7),
        admin_photo_path: row.get(8),
        admin_photo_name: row.get(9),
        admin_photo_mime: row.get(10),
        admin_photo_size: photo_size.and_then(|v| u64::try_from(v).ok()),
        admin_passkey_credential_id: row.get(12),
        admin_source_id: row.get(13),
        admin_status: row.get(14),
        admin_updated_at: row.get(15),
        created_by: row.get(16),
        operation_log_id: row.get(17),
        created_at: row.get(18),
        province_name,
        city_name,
    }
}

/// 新增或更新一条机构管理员链下私密资料(按复合 key upsert)。
pub(crate) fn upsert_institution_admin(db: &Db, admin: &InstitutionAdmin) -> Result<(), String> {
    let admin = admin.clone();
    db.with_client(move |conn| {
        let photo_size = admin.admin_photo_size.and_then(|v| i64::try_from(v).ok());
        conn.execute(
            "INSERT INTO institution_admins (
                cid_number, province_code, city_code, admin_account,
                admin_department, admin_job, admin_contact_phone, admin_contact_email,
                admin_photo_path, admin_photo_name, admin_photo_mime, admin_photo_size,
                admin_passkey_credential_id, admin_source_id, admin_status,
                admin_updated_at, created_by, operation_log_id
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18
             )
             ON CONFLICT (province_code, cid_number, admin_account) DO UPDATE SET
                city_code = EXCLUDED.city_code,
                admin_department = EXCLUDED.admin_department,
                admin_job = EXCLUDED.admin_job,
                admin_contact_phone = EXCLUDED.admin_contact_phone,
                admin_contact_email = EXCLUDED.admin_contact_email,
                admin_photo_path = EXCLUDED.admin_photo_path,
                admin_photo_name = EXCLUDED.admin_photo_name,
                admin_photo_mime = EXCLUDED.admin_photo_mime,
                admin_photo_size = EXCLUDED.admin_photo_size,
                admin_passkey_credential_id = EXCLUDED.admin_passkey_credential_id,
                admin_source_id = EXCLUDED.admin_source_id,
                admin_status = EXCLUDED.admin_status,
                admin_updated_at = EXCLUDED.admin_updated_at,
                created_by = EXCLUDED.created_by,
                operation_log_id = EXCLUDED.operation_log_id",
            &[
                &admin.cid_number,
                &admin.province_code,
                &admin.city_code,
                &admin.admin_account,
                &admin.admin_department,
                &admin.admin_job,
                &admin.admin_contact_phone,
                &admin.admin_contact_email,
                &admin.admin_photo_path,
                &admin.admin_photo_name,
                &admin.admin_photo_mime,
                &photo_size,
                &admin.admin_passkey_credential_id,
                &admin.admin_source_id,
                &admin.admin_status,
                &admin.admin_updated_at,
                &admin.created_by,
                &admin.operation_log_id,
            ],
        )
        .map_err(|e| format!("upsert institution admin failed: {e}"))?;
        Ok(())
    })
}

/// 按 cid_number 列出该机构全部管理员链下资料(分区扫描)。
pub(crate) fn list_institution_admins_by_cid(
    db: &Db,
    cid_number: &str,
) -> Result<Vec<InstitutionAdmin>, String> {
    let cid_number = cid_number.trim().to_string();
    db.with_client(move |conn| {
        let sql = format!(
            "SELECT {SELECT_COLUMNS}
             FROM institution_admins
             WHERE cid_number = $1
             ORDER BY admin_account ASC"
        );
        let rows = conn
            .query(sql.as_str(), &[&cid_number])
            .map_err(|e| format!("list institution admins by cid failed: {e}"))?;
        Ok(rows.iter().map(row_to_admin).collect())
    })
}

/// 在已有连接上按 cid_number 列出机构管理员(供 prepare 阶段在同一事务里组装上链参数)。
pub(crate) fn list_institution_admins_by_cid_conn(
    conn: &mut postgres::Client,
    cid_number: &str,
) -> Result<Vec<InstitutionAdmin>, String> {
    let sql = format!(
        "SELECT {SELECT_COLUMNS}
         FROM institution_admins
         WHERE cid_number = $1
         ORDER BY admin_account ASC"
    );
    let rows = conn
        .query(sql.as_str(), &[&cid_number.trim()])
        .map_err(|e| format!("list institution admins by cid failed: {e}"))?;
    Ok(rows.iter().map(row_to_admin).collect())
}

/// 按 (cid_number, admin_account) 取单条管理员链下资料。
pub(crate) fn get_institution_admin(
    db: &Db,
    cid_number: &str,
    admin_account: &str,
) -> Result<Option<InstitutionAdmin>, String> {
    let cid_number = cid_number.trim().to_string();
    let admin_account = admin_account.trim().to_string();
    db.with_client(move |conn| {
        let sql = format!(
            "SELECT {SELECT_COLUMNS}
             FROM institution_admins
             WHERE cid_number = $1 AND admin_account = $2
             LIMIT 1"
        );
        let row = conn
            .query_opt(sql.as_str(), &[&cid_number, &admin_account])
            .map_err(|e| format!("get institution admin failed: {e}"))?;
        Ok(row.as_ref().map(row_to_admin))
    })
}

/// 删除一条机构管理员链下资料(按复合 key)。
pub(crate) fn delete_institution_admin(
    db: &Db,
    province_code: &str,
    cid_number: &str,
    admin_account: &str,
) -> Result<bool, String> {
    let province_code = province_code.trim().to_string();
    let cid_number = cid_number.trim().to_string();
    let admin_account = admin_account.trim().to_string();
    db.with_client(move |conn| {
        let affected = conn
            .execute(
                "DELETE FROM institution_admins
                 WHERE province_code = $1 AND cid_number = $2 AND admin_account = $3",
                &[&province_code, &cid_number, &admin_account],
            )
            .map_err(|e| format!("delete institution admin failed: {e}"))?;
        Ok(affected > 0)
    })
}

/// 按 cid_number 列出该机构管理员,并按登录管理员可见域(VisibleScope)过滤。
///
/// scope 过滤复用统一入口;记录的省/市名字已在 row_to_admin 里派生回填,
/// 与 `HasProvinceCity` 一致。
pub(crate) fn list_institution_admins_in_scope(
    db: &Db,
    cid_number: &str,
    scope: &VisibleScope,
) -> Result<Vec<InstitutionAdmin>, String> {
    let rows = list_institution_admins_by_cid(db, cid_number)?;
    Ok(filter_by_scope(&rows, scope))
}
