//! # CPMS 编号工具模块 (number)
//!
//! 档案号和护照号统一在这里生成。档案业务模块只消费结果，不再内嵌编号算法。

use axum::{http::StatusCode, Json};
use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use rand::{rngs::OsRng, RngCore};
use sqlx::Row;

use crate::common::{err, ApiError};

type Blake2b256 = Blake2b<U32>;

const ARCHIVE_NO_MAX_RETRY: u32 = 20;
const ARCHIVE_NO_BODY_BYTES: usize = 16;
const PASSPORT_NO_MAX_RETRY: u32 = 1000;
const PASSPORT_BODY_SYMBOLS: usize = 8;
const PASSPORT_CITY_NAMESPACE_COUNT: u64 = 512;
const PASSPORT_BODY_SPACE: u64 = 1u64 << (PASSPORT_BODY_SYMBOLS * 5);
const PASSPORT_LOCAL_CAPACITY: u64 = PASSPORT_BODY_SPACE / PASSPORT_CITY_NAMESPACE_COUNT;
const PASSPORT_CITY_NAMESPACE_MULTIPLIER: u64 = 137;
const PASSPORT_SEQ_MULTIPLIER: u64 = 1_103_515_245;
const ARCHIVE_BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub(crate) struct ArchiveNumbers {
    pub(crate) archive_no: String,
    pub(crate) passport_no: String,
}

/// 同步生成一对一绑定的档案号与护照号。
///
/// 档案号供 CID 绑定使用；护照号印刷在护照上。两个编号在创建档案时
/// 同时生成并写入同一条档案记录，后续注销也跟随档案状态一起处理。
pub(crate) async fn generate_archive_numbers_with_retry(
    conn: &mut sqlx::PgConnection,
    new_archive_id: &str,
    install_secret: &str,
    province_code: &str,
    city_code: &str,
    terminal_id: &str,
    admin_account: &str,
) -> Result<ArchiveNumbers, (StatusCode, Json<ApiError>)> {
    if let Some(numbers) = claim_recycled_archive_numbers(conn, new_archive_id).await? {
        return Ok(numbers);
    }
    let archive_no =
        generate_archive_no_with_retry(conn, install_secret, terminal_id, admin_account).await?;
    let passport_no = generate_passport_no_with_retry(conn, province_code, city_code).await?;
    Ok(ArchiveNumbers {
        archive_no,
        passport_no,
    })
}

async fn generate_archive_no_with_retry(
    conn: &mut sqlx::PgConnection,
    install_secret: &str,
    terminal_id: &str,
    admin_account: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let mut counter = allocate_sequence(conn, "archive_no").await?;

    for _ in 0..ARCHIVE_NO_MAX_RETRY {
        let mut random = [0u8; 32];
        OsRng.fill_bytes(&mut random);
        let body = archive_no_body(
            install_secret,
            terminal_id,
            admin_account,
            counter,
            random.as_slice(),
        );
        let checksum = archive_no_checksum(&body);
        // 档案号不携带协议前缀，避免把示例前缀固化成业务含义。
        let archive_no = format!("{}-{}", body, checksum);

        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM archives WHERE archive_no = $1)")
                .bind(&archive_no)
                .fetch_one(&mut *conn)
                .await
                .map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "archive lookup failed",
                    )
                })?;

        if !exists {
            return Ok(archive_no);
        }
        counter += 1;
    }

    Err(err(
        StatusCode::CONFLICT,
        3005,
        "archive_no conflict, retry exhausted",
    ))
}

async fn generate_passport_no_with_retry(
    conn: &mut sqlx::PgConnection,
    province_code: &str,
    city_code: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    validate_passport_area_codes(province_code, city_code)?;
    let seq_key = format!(
        "passport_no:{}{}",
        province_code.trim().to_ascii_uppercase(),
        city_code.trim()
    );

    for _ in 0..PASSPORT_NO_MAX_RETRY {
        let seq = allocate_sequence(conn, &seq_key).await?;
        let passport_no = build_passport_no(province_code, city_code, seq)?;
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM archives WHERE passport_no = $1)")
                .bind(&passport_no)
                .fetch_one(&mut *conn)
                .await
                .map_err(|_| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "passport lookup failed",
                    )
                })?;
        if !exists {
            return Ok(passport_no);
        }
    }

    Err(err(
        StatusCode::CONFLICT,
        3005,
        "passport_no conflict, retry exhausted",
    ))
}

async fn allocate_sequence(
    conn: &mut sqlx::PgConnection,
    seq_key: &str,
) -> Result<i64, (StatusCode, Json<ApiError>)> {
    sqlx::query_scalar(
        "INSERT INTO sequence_counters (seq_key, next_seq)
         VALUES ($1, 2)
         ON CONFLICT (seq_key) DO UPDATE SET next_seq = sequence_counters.next_seq + 1
         RETURNING next_seq - 1",
    )
    .bind(seq_key)
    .fetch_one(&mut *conn)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "sequence alloc failed",
        )
    })
}

async fn claim_recycled_archive_numbers(
    conn: &mut sqlx::PgConnection,
    new_archive_id: &str,
) -> Result<Option<ArchiveNumbers>, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT pool_id, archive_no, passport_no
         FROM archive_number_recycle_pool
         WHERE used_at IS NULL
         ORDER BY released_at, pool_id
         LIMIT 1
         FOR UPDATE SKIP LOCKED",
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "archive number recycle lookup failed",
        )
    })?;

    let Some(row) = row else {
        return Ok(None);
    };
    let pool_id: String = row.get("pool_id");
    let archive_no: String = row.get("archive_no");
    let passport_no: String = row.get("passport_no");

    // 档案号和护照号必须作为一对领取；事务回滚时领取状态同步回滚。
    let result = sqlx::query(
        "UPDATE archive_number_recycle_pool
         SET used_at = EXTRACT(EPOCH FROM NOW())::BIGINT, used_by_archive_id = $1
         WHERE pool_id = $2 AND used_at IS NULL",
    )
    .bind(new_archive_id)
    .bind(pool_id)
    .execute(&mut *conn)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "archive number recycle claim failed",
        )
    })?;
    if result.rows_affected() != 1 {
        return Err(err(
            StatusCode::CONFLICT,
            3005,
            "archive number recycle claim failed",
        ));
    }

    Ok(Some(ArchiveNumbers {
        archive_no,
        passport_no,
    }))
}

fn archive_no_body(
    install_secret: &str,
    terminal_id: &str,
    admin_account: &str,
    counter: i64,
    random: &[u8],
) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update(b"cid-cpms-v1|archive-no|");
    hasher.update(install_secret.as_bytes());
    hasher.update(b"|");
    hasher.update(terminal_id.as_bytes());
    hasher.update(b"|");
    hasher.update(admin_account.as_bytes());
    hasher.update(b"|");
    hasher.update(counter.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(random);
    let digest = hasher.finalize();
    base32_no_padding(&digest[..ARCHIVE_NO_BODY_BYTES])
}

pub(crate) fn archive_no_checksum(body: &str) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update(b"cid-cpms-v1|archive-no-check|");
    hasher.update(body.as_bytes());
    let digest = hasher.finalize();
    base32_no_padding(&digest[..4]).chars().take(2).collect()
}

fn build_passport_no(
    province_code: &str,
    city_code: &str,
    sequence: i64,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let province = province_code.trim().to_ascii_uppercase();
    let seq0 = u64::try_from(sequence.saturating_sub(1))
        .map_err(|_| err(StatusCode::CONFLICT, 3005, "passport_no capacity exhausted"))?;
    if seq0 >= PASSPORT_LOCAL_CAPACITY {
        return Err(err(
            StatusCode::CONFLICT,
            3005,
            "passport_no capacity exhausted",
        ));
    }

    let namespace = passport_city_namespace(&province, city_code)?;
    let scrambled_seq = scramble_passport_sequence(&province, city_code, seq0);
    let body_number = scrambled_seq * PASSPORT_CITY_NAMESPACE_COUNT + namespace;
    let body = crockford_fixed(body_number, PASSPORT_BODY_SYMBOLS);
    let source = format!("{province}{body}");
    let check = passport_check_char(&source);
    Ok(format!("{source}{check}"))
}

fn validate_passport_area_codes(
    province_code: &str,
    city_code: &str,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let province = province_code.trim();
    let city = city_code.trim();
    if province.len() != 2 || !province.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid passport province_code",
        ));
    }
    passport_city_value(city).map(|_| ())
}

fn passport_city_namespace(
    province_code: &str,
    city_code: &str,
) -> Result<u64, (StatusCode, Json<ApiError>)> {
    let city_value = passport_city_value(city_code)?;
    let offset = province_namespace_offset(province_code);
    // 用 0..511 置换把省内唯一 city_code 映射成城市隔离编号；
    // 生成结果参与号码空间分割，但护照号明文不直接暴露原始市代码。
    Ok((city_value * PASSPORT_CITY_NAMESPACE_MULTIPLIER + offset) % PASSPORT_CITY_NAMESPACE_COUNT)
}

fn passport_city_value(city_code: &str) -> Result<u64, (StatusCode, Json<ApiError>)> {
    let city = city_code.trim();
    if city.len() != 3 || !city.chars().all(|c| c.is_ascii_digit()) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid passport city_code",
        ));
    }
    let city_value = city
        .parse::<u64>()
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid passport city_code"))?;
    if city_value == 0 || city_value >= PASSPORT_CITY_NAMESPACE_COUNT {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid passport city_code",
        ));
    }
    Ok(city_value)
}

fn province_namespace_offset(province_code: &str) -> u64 {
    let mut hasher = Blake2b256::new();
    hasher.update(b"cid-cpms-v1|passport-city-offset|");
    hasher.update(province_code.trim().to_ascii_uppercase().as_bytes());
    let digest = hasher.finalize();
    u16::from_le_bytes([digest[0], digest[1]]) as u64 % PASSPORT_CITY_NAMESPACE_COUNT
}

fn scramble_passport_sequence(province_code: &str, city_code: &str, seq0: u64) -> u64 {
    let mut hasher = Blake2b256::new();
    hasher.update(b"cid-cpms-v1|passport-sequence-salt|");
    hasher.update(province_code.trim().to_ascii_uppercase().as_bytes());
    hasher.update(b"|");
    hasher.update(city_code.trim().as_bytes());
    let digest = hasher.finalize();
    let salt = u64::from_le_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ]) & (PASSPORT_LOCAL_CAPACITY - 1);
    seq0.wrapping_mul(PASSPORT_SEQ_MULTIPLIER)
        .wrapping_add(salt)
        & (PASSPORT_LOCAL_CAPACITY - 1)
}

fn passport_check_char(source: &str) -> char {
    let mut hasher = Blake2b256::new();
    hasher.update(b"cid-cpms-v1|passport-no-check|");
    hasher.update(source.as_bytes());
    let digest = hasher.finalize();
    CROCKFORD_ALPHABET[(digest[0] & 0x1f) as usize] as char
}

fn crockford_fixed(value: u64, width: usize) -> String {
    let mut out = String::with_capacity(width);
    for pos in (0..width).rev() {
        let idx = ((value >> (pos * 5)) & 0x1f) as usize;
        out.push(CROCKFORD_ALPHABET[idx] as char);
    }
    out
}

fn base32_no_padding(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut buffer: u32 = 0;
    let mut bits_left: u8 = 0;
    for byte in bytes {
        buffer = (buffer << 8) | (*byte as u32);
        bits_left += 8;
        while bits_left >= 5 {
            let idx = ((buffer >> (bits_left - 5)) & 0x1f) as usize;
            out.push(ARCHIVE_BASE32_ALPHABET[idx] as char);
            bits_left -= 5;
        }
    }
    if bits_left > 0 {
        let idx = ((buffer << (5 - bits_left)) & 0x1f) as usize;
        out.push(ARCHIVE_BASE32_ALPHABET[idx] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        build_passport_no, generate_archive_numbers_with_retry, passport_city_namespace,
        PASSPORT_CITY_NAMESPACE_COUNT, PASSPORT_LOCAL_CAPACITY,
    };
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[test]
    fn passport_city_namespace_is_unique_for_city_codes_in_scope() {
        let mut seen = std::collections::BTreeSet::new();
        for city in 1..=392 {
            let code = format!("{city:03}");
            let namespace = match passport_city_namespace("GD", &code) {
                Ok(v) => v,
                Err(_) => panic!("namespace"),
            };
            assert!(namespace < PASSPORT_CITY_NAMESPACE_COUNT);
            assert!(seen.insert(namespace));
        }
    }

    #[test]
    fn passport_no_uses_province_body_and_check_char() {
        let no = match build_passport_no("gd", "001", 1) {
            Ok(v) => v,
            Err(_) => panic!("passport no"),
        };
        assert_eq!(no.len(), 11);
        assert!(no.starts_with("GD"));
        assert!(no
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn passport_no_supports_last_local_sequence() {
        let no = match build_passport_no("GD", "392", PASSPORT_LOCAL_CAPACITY as i64) {
            Ok(v) => v,
            Err(_) => panic!("passport no"),
        };
        assert_eq!(no.len(), 11);
    }

    #[tokio::test]
    async fn db_generate_claims_recycled_number_pair() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let case_id = format!("test_claim_{}", Uuid::new_v4().simple());
        let pool_id = format!("pool_{case_id}");
        let source_archive_id = format!("old_{case_id}");
        let new_archive_id = format!("new_{case_id}");
        let archive_no = format!("AN-{case_id}");
        let passport_no = format!("PP{case_id}");

        cleanup_case(&pool, &case_id).await;
        insert_recycle_pool_row(
            &pool,
            &pool_id,
            &source_archive_id,
            &archive_no,
            &passport_no,
        )
        .await;

        let mut conn = pool.acquire().await.expect("acquire connection");
        let numbers = match generate_archive_numbers_with_retry(
            &mut conn,
            &new_archive_id,
            "0123456789abcdef0123456789abcdef",
            "GD",
            "001",
            "terminal-test",
            "admin-pubkey-test",
        )
        .await
        {
            Ok(numbers) => numbers,
            Err(_) => panic!("generate numbers"),
        };

        assert_eq!(numbers.archive_no, archive_no);
        assert_eq!(numbers.passport_no, passport_no);
        let used_by: Option<String> = sqlx::query_scalar(
            "SELECT used_by_archive_id FROM archive_number_recycle_pool WHERE pool_id = $1",
        )
        .bind(&pool_id)
        .fetch_one(&pool)
        .await
        .expect("used by");
        assert_eq!(used_by, Some(new_archive_id));
        cleanup_case(&pool, &case_id).await;
    }

    #[tokio::test]
    async fn db_recycled_number_claim_rolls_back_with_transaction() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let case_id = format!("test_rollback_{}", Uuid::new_v4().simple());
        let pool_id = format!("pool_{case_id}");
        let source_archive_id = format!("old_{case_id}");
        let new_archive_id = format!("new_{case_id}");
        let archive_no = format!("AN-{case_id}");
        let passport_no = format!("PP{case_id}");

        cleanup_case(&pool, &case_id).await;
        insert_recycle_pool_row(
            &pool,
            &pool_id,
            &source_archive_id,
            &archive_no,
            &passport_no,
        )
        .await;

        let mut tx = pool.begin().await.expect("begin tx");
        let numbers = match generate_archive_numbers_with_retry(
            tx.as_mut(),
            &new_archive_id,
            "0123456789abcdef0123456789abcdef",
            "GD",
            "001",
            "terminal-test",
            "admin-pubkey-test",
        )
        .await
        {
            Ok(numbers) => numbers,
            Err(_) => panic!("generate numbers"),
        };
        assert_eq!(numbers.archive_no, archive_no);
        tx.rollback().await.expect("rollback tx");

        let used_by: Option<String> = sqlx::query_scalar(
            "SELECT used_by_archive_id FROM archive_number_recycle_pool WHERE pool_id = $1",
        )
        .bind(&pool_id)
        .fetch_one(&pool)
        .await
        .expect("used by");
        assert_eq!(used_by, None);
        cleanup_case(&pool, &case_id).await;
    }

    #[tokio::test]
    async fn db_recycle_pool_allows_same_numbers_after_previous_claim() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let case_id = format!("test_recycle_again_{}", Uuid::new_v4().simple());
        let archive_no = format!("AN-{case_id}");
        let passport_no = format!("PP{case_id}");

        cleanup_case(&pool, &case_id).await;
        sqlx::query(
            "INSERT INTO archive_number_recycle_pool
             (pool_id, archive_no, passport_no, source_archive_id, deleted_at, released_at, used_at, used_by_archive_id)
             VALUES ($1, $2, $3, $4, 1, 1, 2, $5)",
        )
        .bind(format!("pool_used_{case_id}"))
        .bind(&archive_no)
        .bind(&passport_no)
        .bind(format!("old_first_{case_id}"))
        .bind(format!("new_first_{case_id}"))
        .execute(&pool)
        .await
        .expect("insert used recycle pool row");

        insert_recycle_pool_row(
            &pool,
            &format!("pool_available_{case_id}"),
            &format!("old_second_{case_id}"),
            &archive_no,
            &passport_no,
        )
        .await;

        let available_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM archive_number_recycle_pool WHERE archive_no = $1",
        )
        .bind(&archive_no)
        .fetch_one(&pool)
        .await
        .expect("count recycle rows");
        assert_eq!(available_count, 2);
        cleanup_case(&pool, &case_id).await;
    }

    #[tokio::test]
    async fn db_generate_creates_new_numbers_when_recycle_pool_is_empty() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let case_id = format!("test_empty_{}", Uuid::new_v4().simple());
        cleanup_case(&pool, &case_id).await;

        let mut conn = pool.acquire().await.expect("acquire connection");
        let numbers = match generate_archive_numbers_with_retry(
            &mut conn,
            &format!("new_{case_id}"),
            "0123456789abcdef0123456789abcdef",
            "GD",
            "001",
            "terminal-test",
            "admin-pubkey-test",
        )
        .await
        {
            Ok(numbers) => numbers,
            Err(_) => panic!("generate numbers"),
        };

        assert!(numbers.archive_no.contains('-'));
        assert!(numbers.passport_no.starts_with("GD"));
        cleanup_case(&pool, &case_id).await;
    }

    async fn test_pool() -> Option<sqlx::PgPool> {
        let Ok(database_url) = std::env::var("CPMS_TEST_DATABASE_URL") else {
            return None;
        };
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect(&database_url)
            .await
            .expect("connect CPMS_TEST_DATABASE_URL");
        sqlx::raw_sql(include_str!("../db/schema.sql"))
            .execute(&pool)
            .await
            .expect("apply schema");
        Some(pool)
    }

    async fn insert_recycle_pool_row(
        pool: &sqlx::PgPool,
        pool_id: &str,
        source_archive_id: &str,
        archive_no: &str,
        passport_no: &str,
    ) {
        sqlx::query(
            "INSERT INTO archive_number_recycle_pool
             (pool_id, archive_no, passport_no, source_archive_id, deleted_at, released_at)
             VALUES ($1, $2, $3, $4, 1, 1)",
        )
        .bind(pool_id)
        .bind(archive_no)
        .bind(passport_no)
        .bind(source_archive_id)
        .execute(pool)
        .await
        .expect("insert recycle pool row");
    }

    async fn cleanup_case(pool: &sqlx::PgPool, case_id: &str) {
        sqlx::query("DELETE FROM archive_number_recycle_pool WHERE pool_id LIKE $1 OR source_archive_id LIKE $1 OR used_by_archive_id LIKE $1 OR archive_no LIKE $1 OR passport_no LIKE $1")
            .bind(format!("%{case_id}%"))
            .execute(pool)
            .await
            .expect("cleanup recycle pool");
        sqlx::query("DELETE FROM archives WHERE archive_id LIKE $1 OR archive_no LIKE $1 OR passport_no LIKE $1")
            .bind(format!("%{case_id}%"))
            .execute(pool)
            .await
            .expect("cleanup archives");
    }
}
