//! 中文注释:SFID 结构化数据库入口。
//!
//! 本模块只负责 PostgreSQL 连接池、当前 schema 初始化和短事务封装。
//! 业务主数据必须落到各模块自己的结构化表,不得再恢复旧快照表。

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};

/// 中文注释:postgres::Error 在部分数据库错误上只显示 `db error`。
/// 展开 SQLSTATE、message、detail 和 hint,保证启动期和登录链路能看到真实 SQL 原因。
pub(crate) fn postgres_error_text(err: &postgres::Error) -> String {
    let Some(db_error) = err.as_db_error() else {
        return err.to_string();
    };
    let mut parts = vec![format!("{} {}", db_error.code().code(), db_error.message())];
    if let Some(detail) = db_error.detail() {
        parts.push(format!("detail: {detail}"));
    }
    if let Some(hint) = db_error.hint() {
        parts.push(format!("hint: {hint}"));
    }
    parts.join("; ")
}

#[derive(Clone)]
pub(crate) struct Db {
    clients: Arc<Vec<Mutex<postgres::Client>>>,
    next_client_idx: Arc<AtomicUsize>,
}

impl Db {
    pub(crate) fn from_database_url(database_url: &str) -> Result<Self, String> {
        let db_url = database_url.to_string();
        let pool_size = std::env::var("SFID_PG_POOL_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(4);
        let handle = thread::spawn(move || {
            let mut bootstrap = postgres::Client::connect(db_url.as_str(), postgres::NoTls)
                .map_err(|e| format!("connect postgres failed: {}", postgres_error_text(&e)))?;
            Self::init_current_schema(&mut bootstrap)?;
            let mut clients = Vec::with_capacity(pool_size);
            clients.push(Mutex::new(bootstrap));
            for _ in 1..pool_size {
                let conn =
                    postgres::Client::connect(db_url.as_str(), postgres::NoTls).map_err(|e| {
                        format!(
                            "connect postgres pool client failed: {}",
                            postgres_error_text(&e)
                        )
                    })?;
                clients.push(Mutex::new(conn));
            }
            Ok::<Vec<Mutex<postgres::Client>>, String>(clients)
        });
        let clients = match handle.join() {
            Ok(v) => v?,
            Err(_) => return Err("postgres init thread panicked".to_string()),
        };
        Ok(Self {
            clients: Arc::new(clients),
            next_client_idx: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub(crate) fn with_client<R>(
        &self,
        op: impl FnOnce(&mut postgres::Client) -> Result<R, String> + Send,
    ) -> Result<R, String>
    where
        R: Send,
    {
        if self.clients.is_empty() {
            return Err("postgres client pool is empty".to_string());
        }
        let idx = self.next_client_idx.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        let selected = Arc::clone(&self.clients);
        thread::scope(|scope| {
            let handle = scope.spawn(|| {
                let mut conn = selected[idx]
                    .lock()
                    .map_err(|_| "postgres client lock poisoned".to_string())?;
                op(&mut conn)
            });
            match handle.join() {
                Ok(v) => v,
                Err(_) => Err("postgres worker thread panicked".to_string()),
            }
        })
    }

    fn init_current_schema(conn: &mut postgres::Client) -> Result<(), String> {
        conn.batch_execute(
            "CREATE TABLE IF NOT EXISTS provinces (
                province_name TEXT PRIMARY KEY
             );

             CREATE TABLE IF NOT EXISTS admins (
                admin_id BIGINT PRIMARY KEY,
                admin_pubkey TEXT NOT NULL UNIQUE,
                admin_name TEXT NOT NULL,
                role TEXT NOT NULL CHECK (role IN ('FEDERAL_ADMIN', 'CITY_ADMIN')),
                built_in BOOLEAN NOT NULL DEFAULT FALSE,
                created_by TEXT NOT NULL DEFAULT 'SYSTEM',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ,
                city TEXT NOT NULL DEFAULT ''
             );
             CREATE INDEX IF NOT EXISTS idx_admins_role ON admins(role);
             CREATE INDEX IF NOT EXISTS idx_admins_role_city ON admins(role, city);
             CREATE INDEX IF NOT EXISTS idx_admins_pubkey_lower ON admins(lower(admin_pubkey));
             CREATE INDEX IF NOT EXISTS idx_admins_created_by_lower ON admins(lower(created_by));

             CREATE TABLE IF NOT EXISTS federal_admin_scope (
                admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
                province_name TEXT NOT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT
             );
             CREATE INDEX IF NOT EXISTS idx_federal_admin_scope_province_name
                ON federal_admin_scope(province_name);

             CREATE TABLE IF NOT EXISTS admin_passkeys (
                credential_id TEXT PRIMARY KEY,
                admin_pubkey TEXT NOT NULL,
                label TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'REVOKED')),
                payload JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                last_used_at TIMESTAMPTZ
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkeys_pubkey_status
                ON admin_passkeys(admin_pubkey, status);

             CREATE TABLE IF NOT EXISTS admin_passkey_challenges (
                registration_id TEXT PRIMARY KEY,
                admin_pubkey TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkey_challenges_expires
                ON admin_passkey_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS admin_action_challenges (
                action_id TEXT PRIMARY KEY,
                actor_pubkey TEXT NOT NULL,
                action_type TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_action_challenges_expires
                ON admin_action_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS admin_security_grants (
                grant_id TEXT PRIMARY KEY,
                actor_pubkey TEXT NOT NULL,
                action_type TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_security_grants_expires
                ON admin_security_grants(expires_at);

             CREATE TABLE IF NOT EXISTS admin_login_challenges (
                challenge_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                admin_pubkey TEXT NOT NULL DEFAULT '',
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_login_challenges_expires
                ON admin_login_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS admin_qr_login_results (
                challenge_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                access_token TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                payload JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_qr_login_results_session
                ON admin_qr_login_results(session_id, expires_at);

             CREATE TABLE IF NOT EXISTS admin_sessions (
                token TEXT PRIMARY KEY,
                admin_pubkey TEXT NOT NULL,
                role TEXT NOT NULL CHECK (role IN ('FEDERAL_ADMIN', 'CITY_ADMIN')),
                expires_at TIMESTAMPTZ NOT NULL,
                last_active_at TIMESTAMPTZ NOT NULL,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_sessions_pubkey
                ON admin_sessions(admin_pubkey);
             CREATE INDEX IF NOT EXISTS idx_admin_sessions_expires
                ON admin_sessions(expires_at);

             CREATE TABLE IF NOT EXISTS cpms_sites (
                sfid_number TEXT PRIMARY KEY,
                p_code TEXT NOT NULL,
                c_code TEXT NOT NULL,
                status TEXT NOT NULL,
                install_token_status TEXT NOT NULL,
                cpms_pubkey_hash TEXT,
                created_by TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_cpms_sites_scope
                ON cpms_sites(p_code, c_code, status);

             CREATE TABLE IF NOT EXISTS citizen_bind_challenges (
                challenge_id TEXT PRIMARY KEY,
                p_code TEXT NOT NULL,
                c_code TEXT NOT NULL,
                wallet_pubkey TEXT NOT NULL,
                archive_no TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_citizen_bind_challenges_expires
                ON citizen_bind_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS citizen_status_imports (
                sfid_number TEXT NOT NULL,
                export_year INT NOT NULL,
                export_batch_id TEXT NOT NULL,
                records_hash TEXT NOT NULL,
                imported_at TIMESTAMPTZ NOT NULL,
                imported_by TEXT NOT NULL,
                payload JSONB NOT NULL,
                PRIMARY KEY (sfid_number, export_year)
             );

             CREATE TABLE IF NOT EXISTS qr_consumed (
                qr_id TEXT PRIMARY KEY,
                consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                expires_at TIMESTAMPTZ NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_qr_consumed_expires
                ON qr_consumed(expires_at);

             CREATE TABLE IF NOT EXISTS chain_requests (
                route_key TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                nonce TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                received_at TIMESTAMPTZ NOT NULL,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_chain_requests_received
                ON chain_requests(received_at);

             CREATE TABLE IF NOT EXISTS chain_nonces (
                nonce TEXT PRIMARY KEY,
                seen_at TIMESTAMPTZ NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_chain_nonces_seen
                ON chain_nonces(seen_at);

             CREATE TABLE IF NOT EXISTS tx_records (
                id BIGSERIAL PRIMARY KEY,
                block_number BIGINT NOT NULL,
                extrinsic_index SMALLINT,
                event_index SMALLINT NOT NULL,
                tx_type TEXT NOT NULL,
                from_address TEXT,
                to_address TEXT,
                amount_fen BIGINT NOT NULL,
                fee_fen BIGINT,
                block_timestamp TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             CREATE INDEX IF NOT EXISTS idx_tx_records_from
                ON tx_records (from_address, block_number DESC);
             CREATE INDEX IF NOT EXISTS idx_tx_records_to
                ON tx_records (to_address, block_number DESC);
             CREATE INDEX IF NOT EXISTS idx_tx_records_block
                ON tx_records (block_number DESC);
             CREATE INDEX IF NOT EXISTS idx_tx_records_type
                ON tx_records (tx_type);

             CREATE TABLE IF NOT EXISTS tx_indexer_state (
                id INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
                last_indexed_block BIGINT NOT NULL DEFAULT 0,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             INSERT INTO tx_indexer_state (id, last_indexed_block)
             VALUES (1, 0)
             ON CONFLICT (id) DO NOTHING;

             CREATE TABLE IF NOT EXISTS gov_manifest (
                scope_key TEXT PRIMARY KEY,
                china_hash TEXT NOT NULL,
                catalog_hash TEXT NOT NULL,
                template_version TEXT NOT NULL,
                target_count BIGINT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('OK', 'INCOMPLETE')),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );",
        )
        .map_err(|e| format!("init core schema failed: {}", postgres_error_text(&e)))?;
        Self::init_subject_partition_schema(conn)?;
        Ok(())
    }

    fn init_subject_partition_schema(conn: &mut postgres::Client) -> Result<(), String> {
        conn.batch_execute(
            "CREATE TABLE IF NOT EXISTS ids (
                sfid_number TEXT PRIMARY KEY,
                kind TEXT NOT NULL CHECK (kind IN ('CITIZEN', 'PUBLIC', 'PRIVATE')),
                p_code TEXT NOT NULL,
                c_code TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );

             CREATE TABLE IF NOT EXISTS subjects (
                sfid_number TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('CITIZEN', 'PUBLIC', 'PRIVATE')),
	                name TEXT,
	                sfid_name TEXT,
	                short_name TEXT,
	                p_code TEXT NOT NULL,
	                c_code TEXT,
	                t_code TEXT,
	                status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'REVOKED')),
                category TEXT,
                subject_property TEXT,
                p1 TEXT,
	                province TEXT,
	                city TEXT,
	                town TEXT,
	                province_code TEXT,
	                city_code TEXT,
	                town_code TEXT,
	                institution_code TEXT,
	                org_code TEXT,
                education_type TEXT,
                private_type TEXT,
                partnership_kind TEXT,
                has_legal_personality BOOLEAN,
                parent_sfid_number TEXT,
                legal_rep_name TEXT,
                legal_rep_sfid_number TEXT,
                legal_rep_photo_path TEXT,
                legal_rep_photo_name TEXT,
                legal_rep_photo_mime TEXT,
                legal_rep_photo_size BIGINT,
                created_by TEXT,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (p_code, sfid_number)
             ) PARTITION BY LIST (p_code);

             CREATE TABLE IF NOT EXISTS citizens (
                sfid_number TEXT NOT NULL,
                p_code TEXT NOT NULL,
                c_code TEXT NOT NULL,
                id BIGINT,
                archive_no TEXT,
                wallet_pubkey TEXT,
                wallet_address TEXT,
                citizen_status TEXT NOT NULL,
                voting_eligible BOOLEAN NOT NULL,
                valid_from TEXT,
                valid_until TEXT,
                status_updated_at BIGINT,
                residence_p_code TEXT,
                residence_c_code TEXT,
                residence_t_code TEXT,
                birth_p_code TEXT,
                birth_c_code TEXT,
                birth_t_code TEXT,
                election_scope_level TEXT NOT NULL DEFAULT 'PROVINCE' CHECK (election_scope_level IN ('PROVINCE', 'CITY', 'TOWN')),
                bind_status TEXT NOT NULL DEFAULT 'BOUND' CHECK (bind_status IN ('PENDING', 'BOUND')),
                bound_at TIMESTAMPTZ,
                bound_by TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (p_code, sfid_number)
             ) PARTITION BY LIST (p_code);

             CREATE TABLE IF NOT EXISTS gov (
                sfid_number TEXT NOT NULL,
                p_code TEXT NOT NULL,
		                c_code TEXT,
		                t_code TEXT,
		                institution_code TEXT NOT NULL,
		                org_code TEXT,
                source TEXT NOT NULL DEFAULT 'MANUAL' CHECK (source IN ('GENERATED', 'MANUAL')),
                home_p TEXT,
                home_c TEXT,
                PRIMARY KEY (p_code, sfid_number)
             ) PARTITION BY LIST (p_code);

             CREATE TABLE IF NOT EXISTS private (
                sfid_number TEXT NOT NULL,
                p_code TEXT NOT NULL,
                c_code TEXT NOT NULL,
                code TEXT NOT NULL,
                private_type TEXT NOT NULL CHECK (private_type IN ('SOLE', 'PARTNERSHIP', 'COMPANY', 'CORPORATION', 'WELFARE', 'ASSOCIATION')),
                partnership_kind TEXT CHECK (partnership_kind IN ('GENERAL', 'LIMITED')),
                has_legal_personality BOOLEAN NOT NULL,
                subject_property TEXT NOT NULL CHECK (subject_property IN ('S', 'F')),
                p1 TEXT NOT NULL CHECK (p1 IN ('0', '1')),
                parent_sfid_number TEXT,
                PRIMARY KEY (p_code, sfid_number)
             ) PARTITION BY LIST (p_code);

             CREATE TABLE IF NOT EXISTS accounts (
                sfid_number TEXT NOT NULL,
                p_code TEXT NOT NULL,
                c_code TEXT,
                account_name TEXT NOT NULL,
                duoqian_address TEXT,
                chain_status TEXT NOT NULL CHECK (chain_status IN ('NOT_ON_CHAIN', 'PENDING_ON_CHAIN', 'ACTIVE_ON_CHAIN', 'REVOKED_ON_CHAIN')),
                created_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (p_code, sfid_number, account_name)
             ) PARTITION BY LIST (p_code);

             CREATE TABLE IF NOT EXISTS docs (
                id BIGSERIAL,
                sfid_number TEXT NOT NULL,
                p_code TEXT NOT NULL,
                c_code TEXT,
                file_name TEXT NOT NULL,
                doc_type TEXT NOT NULL,
                file_size BIGINT NOT NULL DEFAULT 0,
                file_path TEXT NOT NULL,
                uploaded_by TEXT NOT NULL,
                uploaded_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (p_code, id)
             ) PARTITION BY LIST (p_code);

             CREATE TABLE IF NOT EXISTS audit (
                id BIGSERIAL,
                p_code TEXT NOT NULL,
                c_code TEXT,
                actor TEXT NOT NULL,
                action TEXT NOT NULL,
                target_sfid TEXT,
                detail JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (p_code, id)
             ) PARTITION BY LIST (p_code);

             -- 中文注释:审计 detail 由自由文本改结构化 JSONB(事实与展示分离,
             -- 展示翻译归前端)。旧 TEXT 列存的是写死文案无法结构化,按用户确认
             -- 直接清空重建列类型(开发期运行痕迹,不留旧方案);收敛块幂等。
             DO $$ BEGIN
                IF EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'audit' AND column_name = 'detail'
                      AND data_type = 'text'
                ) THEN
                    TRUNCATE audit;
                    ALTER TABLE audit ALTER COLUMN detail TYPE JSONB USING detail::jsonb;
                END IF;
             END $$;",
        )
	        .map_err(|e| {
	            format!(
	                "init subject partition parent schema failed: {}",
	                postgres_error_text(&e)
	            )
	        })?;

        // 中文注释:旧表已存在时 CREATE TABLE IF NOT EXISTS 不会补新列,必须先把父表收敛到目标字段。
        conn.batch_execute(
            "ALTER TABLE subjects
                ADD COLUMN IF NOT EXISTS subject_property TEXT,
                ADD COLUMN IF NOT EXISTS sfid_name TEXT,
                ADD COLUMN IF NOT EXISTS education_type TEXT,
                ADD COLUMN IF NOT EXISTS legal_rep_name TEXT,
                ADD COLUMN IF NOT EXISTS legal_rep_sfid_number TEXT,
                ADD COLUMN IF NOT EXISTS legal_rep_photo_path TEXT,
                ADD COLUMN IF NOT EXISTS legal_rep_photo_name TEXT,
                ADD COLUMN IF NOT EXISTS legal_rep_photo_mime TEXT,
                ADD COLUMN IF NOT EXISTS legal_rep_photo_size BIGINT,
                DROP COLUMN IF EXISTS chain_status,
                DROP COLUMN IF EXISTS full_name,
                DROP COLUMN IF EXISTS a3;
             ALTER TABLE private
                ADD COLUMN IF NOT EXISTS subject_property TEXT,
                ADD COLUMN IF NOT EXISTS private_type TEXT,
                ADD COLUMN IF NOT EXISTS partnership_kind TEXT,
                ADD COLUMN IF NOT EXISTS has_legal_personality BOOLEAN,
                DROP COLUMN IF EXISTS kind,
                DROP COLUMN IF EXISTS sub_type,
                DROP COLUMN IF EXISTS a3;
             ALTER TABLE subjects
                ADD COLUMN IF NOT EXISTS private_type TEXT,
                ADD COLUMN IF NOT EXISTS partnership_kind TEXT,
                ADD COLUMN IF NOT EXISTS has_legal_personality BOOLEAN,
                DROP COLUMN IF EXISTS sub_type;
             ALTER TABLE gov
                ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'MANUAL',
                DROP COLUMN IF EXISTS chain_status;",
        )
        .map_err(|e| {
            format!(
                "sync target subject schema failed: {}",
                postgres_error_text(&e)
            )
        })?;

        conn.batch_execute(
            "ALTER TABLE gov
                DROP CONSTRAINT IF EXISTS gov_source_check;
             ALTER TABLE gov
                ADD CONSTRAINT gov_source_check
                CHECK (source IN ('GENERATED', 'MANUAL'));
             UPDATE gov g
             SET source = 'GENERATED'
             FROM subjects s
             WHERE s.p_code = g.p_code
               AND s.sfid_number = g.sfid_number
               AND s.kind = 'PUBLIC'
               AND s.created_by = 'SYSTEM'
               AND s.category IN ('GOV_INSTITUTION', 'PUBLIC_SECURITY')
               AND g.source IS DISTINCT FROM 'GENERATED';",
        )
        .map_err(|e| {
            format!(
                "sync gov source boundary failed: {}",
                postgres_error_text(&e)
            )
        })?;

        conn.batch_execute(
            "ALTER TABLE citizens
                ADD COLUMN IF NOT EXISTS residence_p_code TEXT,
                ADD COLUMN IF NOT EXISTS residence_c_code TEXT,
                ADD COLUMN IF NOT EXISTS residence_t_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_p_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_c_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_t_code TEXT,
                ADD COLUMN IF NOT EXISTS election_scope_level TEXT NOT NULL DEFAULT 'PROVINCE';
             ALTER TABLE citizens
                DROP CONSTRAINT IF EXISTS citizens_election_scope_level_check;
             ALTER TABLE citizens
                ADD CONSTRAINT citizens_election_scope_level_check
                CHECK (election_scope_level IN ('PROVINCE', 'CITY', 'TOWN'));",
        )
        .map_err(|e| {
            format!(
                "sync target citizen schema failed: {}",
                postgres_error_text(&e)
            )
        })?;

        Self::validate_target_subject_schema(conn)?;

        // 中文注释:教育委员会从公权目录迁入教育机构 tab 后,已生成的国家/市公民教育委员会
        // 需要有稳定业务分类;该分类只用于展示与查询,不参与 sfid_number 生成。
        conn.batch_execute(
            "UPDATE subjects
             SET education_type = CASE
                WHEN org_code = 'NATIONAL_EDU' THEN 'NATIONAL_CITIZEN_EDU_COMMITTEE'
                WHEN org_code = 'CITY_EDU' THEN 'CITY_CITIZEN_EDU_COMMITTEE'
                ELSE education_type
             END
             WHERE institution_code = 'JY'
               AND org_code IN ('NATIONAL_EDU', 'CITY_EDU')
               AND education_type IS DISTINCT FROM CASE
                    WHEN org_code = 'NATIONAL_EDU' THEN 'NATIONAL_CITIZEN_EDU_COMMITTEE'
                    WHEN org_code = 'CITY_EDU' THEN 'CITY_CITIZEN_EDU_COMMITTEE'
                    ELSE education_type
               END;",
        )
        .map_err(|e| {
            format!(
                "backfill education institution type failed: {}",
                postgres_error_text(&e)
            )
        })?;

        conn.batch_execute(
            "CREATE INDEX IF NOT EXISTS idx_subjects_city
                ON subjects (p_code, c_code, kind, status, sfid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_town
                ON subjects (p_code, c_code, t_code, kind, status, sfid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_name
                ON subjects (p_code, c_code, name);
             CREATE INDEX IF NOT EXISTS idx_subjects_scope_created
                ON subjects (category, province, city, created_at DESC, sfid_number DESC);
             CREATE INDEX IF NOT EXISTS idx_subjects_exact_lookup
                ON subjects (category, province, city, sfid_number, name);
             CREATE INDEX IF NOT EXISTS idx_subjects_legal_rep
                ON subjects (p_code, legal_rep_sfid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_education
                ON subjects (p_code, c_code, institution_code, education_type, status);
             CREATE INDEX IF NOT EXISTS idx_citizens_scope_created
                ON citizens (p_code, c_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_province_created
                ON citizens (p_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_exact_lookup
                ON citizens (p_code, c_code, archive_no, sfid_number, wallet_pubkey, wallet_address);
             CREATE INDEX IF NOT EXISTS idx_citizens_residence_scope
                ON citizens (residence_p_code, residence_c_code, residence_t_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_birth_scope
                ON citizens (birth_p_code, birth_c_code, birth_t_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_gov_city
                ON gov (p_code, c_code, institution_code);
             CREATE INDEX IF NOT EXISTS idx_gov_org
                ON gov (p_code, org_code);
             CREATE INDEX IF NOT EXISTS idx_private_city
                ON private (p_code, c_code, private_type, code);
             CREATE INDEX IF NOT EXISTS idx_accounts_sfid
                ON accounts (p_code, sfid_number);
             CREATE INDEX IF NOT EXISTS idx_docs_sfid
                ON docs (p_code, sfid_number, uploaded_at DESC);
             CREATE INDEX IF NOT EXISTS idx_audit_scope_time
                ON audit (p_code, c_code, created_at DESC);",
        )
        .map_err(|e| {
            format!(
                "init subject partition indexes failed: {}",
                postgres_error_text(&e)
            )
        })?;

        for province in crate::china::provinces().iter() {
            Self::create_subject_partitions(conn, province.code)?;
        }
        Self::delete_ineligible_citizen_residuals(conn)?;
        Ok(())
    }

    fn delete_ineligible_citizen_residuals(conn: &mut postgres::Client) -> Result<(), String> {
        // 中文注释:SFID 公民库目标状态是“可投票公民库”;历史残留的注销、无选举资格或无钱包记录不再保留。
        conn.batch_execute(
            "WITH doomed AS (
                SELECT p_code, sfid_number, archive_no
                FROM citizens
                WHERE bind_status <> 'BOUND'
                   OR citizen_status <> 'NORMAL'
                   OR voting_eligible IS DISTINCT FROM true
                   OR COALESCE(wallet_pubkey, '') = ''
                   OR COALESCE(wallet_address, '') = ''
                   OR COALESCE(archive_no, '') = ''
             ),
             deleted_challenges AS (
                DELETE FROM citizen_bind_challenges ch
                USING doomed d
                WHERE ch.archive_no = d.archive_no
                RETURNING 1
             ),
             deleted_subjects AS (
                DELETE FROM subjects s
                USING doomed d
                WHERE s.p_code = d.p_code
                  AND s.sfid_number = d.sfid_number
                  AND s.kind = 'CITIZEN'
                RETURNING 1
             ),
             deleted_ids AS (
                DELETE FROM ids i
                USING doomed d
                WHERE i.sfid_number = d.sfid_number
                  AND i.kind = 'CITIZEN'
                RETURNING 1
             )
             DELETE FROM citizens c
             USING doomed d
             WHERE c.p_code = d.p_code
               AND c.sfid_number = d.sfid_number;",
        )
        .map_err(|e| {
            format!(
                "delete ineligible citizen residuals failed: {}",
                postgres_error_text(&e)
            )
        })?;
        Ok(())
    }

    // 中文注释:把启动期失败提前到清晰的目标状态校验,避免后续索引或业务 SQL 报隐晦字段错误。
    fn validate_target_subject_schema(conn: &mut postgres::Client) -> Result<(), String> {
        for column in [
            "sfid_name",
            "legal_rep_name",
            "legal_rep_sfid_number",
            "legal_rep_photo_path",
            "legal_rep_photo_name",
            "legal_rep_photo_mime",
            "legal_rep_photo_size",
            "subject_property",
            "private_type",
            "partnership_kind",
            "has_legal_personality",
            "education_type",
        ] {
            Self::ensure_column_state(conn, "subjects", column, true)?;
        }
        Self::ensure_column_state(conn, "private", "subject_property", true)?;
        for column in ["private_type", "partnership_kind", "has_legal_personality"] {
            Self::ensure_column_state(conn, "private", column, true)?;
        }
        for column in [
            "residence_p_code",
            "residence_c_code",
            "residence_t_code",
            "birth_p_code",
            "birth_c_code",
            "birth_t_code",
            "election_scope_level",
        ] {
            Self::ensure_column_state(conn, "citizens", column, true)?;
        }
        Self::ensure_column_state(conn, "gov", "source", true)?;
        // 中文注释:旧 SFID 方案残列和旧私权分类列必须不存在。
        for (table, column) in [
            ("subjects", "chain_status"),
            ("gov", "chain_status"),
            ("subjects", "full_name"),
            ("subjects", "a3"),
            ("subjects", "sub_type"),
            ("private", "a3"),
            ("private", "kind"),
            ("private", "sub_type"),
        ] {
            Self::ensure_column_state(conn, table, column, false)?;
        }
        Ok(())
    }

    fn ensure_column_state(
        conn: &mut postgres::Client,
        table: &str,
        column: &str,
        must_exist: bool,
    ) -> Result<(), String> {
        let row = conn
            .query_one(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = current_schema()
                      AND table_name = $1
                      AND column_name = $2
                )",
                &[&table, &column],
            )
            .map_err(|e| {
                format!(
                    "inspect column {table}.{column} failed: {}",
                    postgres_error_text(&e)
                )
            })?;
        let exists: bool = row.get(0);
        if must_exist && !exists {
            return Err(format!("target schema missing column {table}.{column}"));
        }
        if !must_exist && exists {
            return Err(format!(
                "target schema still has deprecated column {table}.{column}"
            ));
        }
        Ok(())
    }

    fn create_subject_partitions(conn: &mut postgres::Client, p_code: &str) -> Result<(), String> {
        for table in crate::subjects::schema::PARTITIONED_TABLES {
            let partition_name = format!("{}_{}", table, p_code.to_ascii_lowercase());
            let sql = format!(
                "CREATE TABLE IF NOT EXISTS {partition_name} PARTITION OF {table} FOR VALUES IN ('{p_code}')"
            );
            conn.batch_execute(sql.as_str()).map_err(|e| {
                format!(
                    "init partition {partition_name} failed: {}",
                    postgres_error_text(&e)
                )
            })?;
        }
        Ok(())
    }
}
