//! 中文注释:CID 结构化数据库入口。
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
        let pool_size = std::env::var("CID_PG_POOL_SIZE")
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

             -- 中文注释:机构/账户「注册局域注销态」+ 已签发注销凭证(区别于链投影 chain_status）。
             -- 注册局管理员发起注销(PasskeyChallenge）后写 ISSUED;机构管理员持凭证上链 propose_close,
             -- indexer 收到链上关闭后置 ONCHAIN_CLOSED(投影子项）。见 ADR-023 §6.3。
             CREATE TABLE IF NOT EXISTS institution_deregistrations (
                id               BIGSERIAL PRIMARY KEY,
                cid_number       TEXT NOT NULL,
                account_name     TEXT NOT NULL,
                scope            SMALLINT NOT NULL,
                target_account   TEXT NOT NULL,
                deregister_nonce TEXT NOT NULL UNIQUE,
                signature        TEXT,
                issuer_cid_number   TEXT NOT NULL DEFAULT '',
                issuer_main_account TEXT NOT NULL DEFAULT '',
                signer_pubkey       TEXT NOT NULL DEFAULT '',
                status           TEXT NOT NULL DEFAULT 'ISSUED'
                    CHECK (status IN ('ISSUED', 'ONCHAIN_CLOSED')),
                issued_by        TEXT NOT NULL,
                issued_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
                closed_at        TIMESTAMPTZ
             );
             CREATE INDEX IF NOT EXISTS idx_inst_dereg_cid
                ON institution_deregistrations(cid_number, status);
             CREATE UNIQUE INDEX IF NOT EXISTS idx_inst_dereg_target_active
                ON institution_deregistrations(lower(target_account)) WHERE status = 'ISSUED';

             CREATE TABLE IF NOT EXISTS admins (
                admin_id BIGINT PRIMARY KEY,
                admin_account TEXT NOT NULL UNIQUE,
                admin_name TEXT NOT NULL,
                registry_org_code TEXT NOT NULL CHECK (registry_org_code IN ('FEDERAL_REGISTRY', 'CITY_REGISTRY')),
                built_in BOOLEAN NOT NULL DEFAULT FALSE,
                created_by TEXT NOT NULL DEFAULT 'SYSTEM',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ,
                city_name TEXT NOT NULL DEFAULT ''
             );
             DO $$
             BEGIN
                IF EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'admins' AND column_name = 'admin_display_name'
                ) AND NOT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'admins' AND column_name = 'admin_name'
                ) THEN
                    ALTER TABLE admins RENAME COLUMN admin_display_name TO admin_name;
                END IF;
             END $$;
             UPDATE admins
             SET admin_name = admin_account
             WHERE admin_name IS NULL OR admin_name = '';
             ALTER TABLE admins
                ALTER COLUMN admin_account SET NOT NULL,
                ALTER COLUMN admin_name SET NOT NULL,
                ALTER COLUMN registry_org_code SET NOT NULL,
                DROP CONSTRAINT IF EXISTS admins_registry_org_code_check,
                ADD CONSTRAINT admins_registry_org_code_check
                CHECK (registry_org_code IN ('FEDERAL_REGISTRY', 'CITY_REGISTRY'));
             CREATE UNIQUE INDEX IF NOT EXISTS admins_admin_account_key ON admins(admin_account);
             CREATE INDEX IF NOT EXISTS idx_admins_registry_org_code ON admins(registry_org_code);
             CREATE INDEX IF NOT EXISTS idx_admins_registry_org_code_city_name ON admins(registry_org_code, city_name);
             CREATE INDEX IF NOT EXISTS idx_admins_account_lower ON admins(lower(admin_account));
             CREATE INDEX IF NOT EXISTS idx_admins_created_by_lower ON admins(lower(created_by));

             CREATE TABLE IF NOT EXISTS federal_registry_scope (
                admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
                province_name TEXT NOT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT
             );
             CREATE INDEX IF NOT EXISTS idx_federal_registry_scope_province_name
                ON federal_registry_scope(province_name);

             CREATE TABLE IF NOT EXISTS admin_passkeys (
                credential_id TEXT PRIMARY KEY,
                admin_account TEXT NOT NULL,
                label TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'REVOKED')),
                payload JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                last_used_at TIMESTAMPTZ
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkeys_account_status
                ON admin_passkeys(admin_account, status);

             CREATE TABLE IF NOT EXISTS admin_passkey_challenges (
                registration_id TEXT PRIMARY KEY,
                admin_account TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkey_challenges_expires
                ON admin_passkey_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS admin_action_challenges (
                action_id TEXT PRIMARY KEY,
                actor_account TEXT NOT NULL,
                action_type TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_action_challenges_expires
                ON admin_action_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS admin_security_grants (
                grant_id TEXT PRIMARY KEY,
                actor_account TEXT NOT NULL,
                action_type TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_security_grants_expires
                ON admin_security_grants(expires_at);

             CREATE TABLE IF NOT EXISTS admin_login_sign_requests (
                challenge_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                admin_account TEXT NOT NULL DEFAULT '',
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_login_sign_requests_expires
                ON admin_login_sign_requests(expires_at);

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
                admin_account TEXT NOT NULL,
                registry_org_code TEXT NOT NULL CHECK (registry_org_code IN ('FEDERAL_REGISTRY', 'CITY_REGISTRY')),
                expires_at TIMESTAMPTZ NOT NULL,
                last_active_at TIMESTAMPTZ NOT NULL,
                payload JSONB NOT NULL
             );
             ALTER TABLE admin_sessions
                ALTER COLUMN admin_account SET NOT NULL,
                ALTER COLUMN registry_org_code SET NOT NULL,
                DROP CONSTRAINT IF EXISTS admin_sessions_registry_org_code_check,
                ADD CONSTRAINT admin_sessions_registry_org_code_check
                CHECK (registry_org_code IN ('FEDERAL_REGISTRY', 'CITY_REGISTRY'));
             CREATE INDEX IF NOT EXISTS idx_admin_sessions_account
                ON admin_sessions(admin_account);
             CREATE INDEX IF NOT EXISTS idx_admin_sessions_expires
                ON admin_sessions(expires_at);

             CREATE TABLE IF NOT EXISTS cpms_sites (
                cid_number TEXT PRIMARY KEY,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                status TEXT NOT NULL,
                install_token_status TEXT NOT NULL,
                cpms_pubkey_hash TEXT,
                created_by TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_cpms_sites_scope
                ON cpms_sites(province_code, city_code, status);

             CREATE TABLE IF NOT EXISTS citizen_bind_challenges (
                challenge_id TEXT PRIMARY KEY,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                wallet_pubkey TEXT NOT NULL,
                archive_no TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_citizen_bind_challenges_expires
                ON citizen_bind_challenges(expires_at);

             CREATE TABLE IF NOT EXISTS citizen_status_imports (
                cid_number TEXT NOT NULL,
                export_year INT NOT NULL,
                export_batch_id TEXT NOT NULL,
                records_hash TEXT NOT NULL,
                imported_at TIMESTAMPTZ NOT NULL,
                imported_by TEXT NOT NULL,
                payload JSONB NOT NULL,
                PRIMARY KEY (cid_number, export_year)
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
                cid_number TEXT PRIMARY KEY,
                kind TEXT NOT NULL CHECK (kind IN ('CITIZEN', 'PUBLIC', 'PRIVATE')),
                province_code TEXT NOT NULL,
                city_code TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );

             CREATE TABLE IF NOT EXISTS subjects (
                cid_number TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('CITIZEN', 'PUBLIC', 'PRIVATE')),
                cid_full_name TEXT,
                cid_short_name TEXT,
                status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'REVOKED')),
                category TEXT,
                p1 TEXT,
                province_name TEXT,
                city_name TEXT,
                town_name TEXT,
                province_code TEXT NOT NULL,
                city_code TEXT,
                town_code TEXT,
                institution_code TEXT,
                education_type TEXT,
                private_type TEXT,
                partnership_kind TEXT,
                has_legal_personality BOOLEAN,
                parent_cid_number TEXT,
                legal_rep_name TEXT,
                legal_rep_cid_number TEXT,
                legal_rep_photo_path TEXT,
                legal_rep_photo_name TEXT,
                legal_rep_photo_mime TEXT,
                legal_rep_photo_size BIGINT,
                created_by TEXT,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (province_code, cid_number)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS citizens (
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                id BIGINT,
                archive_no TEXT,
                wallet_pubkey TEXT,
                wallet_address TEXT,
                citizen_status TEXT NOT NULL,
                voting_eligible BOOLEAN NOT NULL,
                valid_from TEXT,
                valid_until TEXT,
                status_updated_at BIGINT,
                residence_province_code TEXT,
                residence_city_code TEXT,
                residence_town_code TEXT,
                birth_province_code TEXT,
                birth_city_code TEXT,
                birth_town_code TEXT,
                election_scope_level TEXT NOT NULL DEFAULT 'PROVINCE' CHECK (election_scope_level IN ('PROVINCE', 'CITY', 'TOWN')),
                bind_status TEXT NOT NULL DEFAULT 'BOUND' CHECK (bind_status IN ('PENDING', 'BOUND')),
                bound_at TIMESTAMPTZ,
                bound_by TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (province_code, cid_number)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS gov (
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
		                city_code TEXT,
		                town_code TEXT,
		                institution_code TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'MANUAL' CHECK (source IN ('GENERATED', 'MANUAL')),
                home_p TEXT,
                home_c TEXT,
                PRIMARY KEY (province_code, cid_number)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS private (
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                code TEXT NOT NULL,
                private_type TEXT NOT NULL CHECK (private_type IN ('SOLE', 'PARTNERSHIP', 'COMPANY', 'CORPORATION', 'WELFARE', 'ASSOCIATION')),
                partnership_kind TEXT CHECK (partnership_kind IN ('GENERAL', 'LIMITED')),
                has_legal_personality BOOLEAN NOT NULL,
                p1 TEXT NOT NULL CHECK (p1 IN ('0', '1')),
                parent_cid_number TEXT,
                PRIMARY KEY (province_code, cid_number)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS accounts (
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
                city_code TEXT,
                account_name TEXT NOT NULL,
                account TEXT,
                chain_status TEXT NOT NULL CHECK (chain_status IN ('NOT_ON_CHAIN', 'PENDING_ON_CHAIN', 'ACTIVE_ON_CHAIN', 'REVOKED_ON_CHAIN')),
                created_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (province_code, cid_number, account_name)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS docs (
                id BIGSERIAL,
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
                city_code TEXT,
                file_name TEXT NOT NULL,
                doc_type TEXT NOT NULL,
                file_size BIGINT NOT NULL DEFAULT 0,
                file_path TEXT NOT NULL,
                uploaded_by TEXT NOT NULL,
                uploaded_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (province_code, id)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS audit (
                id BIGSERIAL,
                province_code TEXT NOT NULL,
                city_code TEXT,
                actor TEXT NOT NULL,
                action TEXT NOT NULL,
                target_cid TEXT,
                detail JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (province_code, id)
             ) PARTITION BY LIST (province_code);

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

        conn.batch_execute(
            "ALTER TABLE gov
                DROP CONSTRAINT IF EXISTS gov_source_check;
             ALTER TABLE gov
                ADD CONSTRAINT gov_source_check
                CHECK (source IN ('GENERATED', 'MANUAL'));
             UPDATE gov g
             SET source = 'GENERATED'
             FROM subjects s
             WHERE s.province_code = g.province_code
               AND s.cid_number = g.cid_number
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
                ADD COLUMN IF NOT EXISTS residence_province_code TEXT,
                ADD COLUMN IF NOT EXISTS residence_city_code TEXT,
                ADD COLUMN IF NOT EXISTS residence_town_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_province_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_city_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_town_code TEXT,
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

        conn.batch_execute("ALTER TABLE subjects DROP COLUMN IF EXISTS name;")
            .map_err(|e| {
                format!(
                    "drop deprecated subjects legacy display column failed: {}",
                    postgres_error_text(&e)
                )
            })?;

        Self::validate_target_subject_schema(conn)?;

        // 中文注释:教育委员会从公权目录迁入教育机构 tab 后,已生成的国家/市公民教育委员会
        // 需要有稳定业务分类;该分类只用于展示与查询,不参与 cid_number 生成。
        conn.batch_execute(
            "UPDATE subjects
             SET education_type = CASE
                WHEN institution_code = 'NED' THEN 'NATIONAL_CITIZEN_EDU_COMMITTEE'
                WHEN institution_code = 'CEDU' THEN 'CITY_CITIZEN_EDU_COMMITTEE'
                ELSE education_type
             END
             WHERE institution_code IN ('NED', 'CEDU')
               AND education_type IS DISTINCT FROM CASE
                    WHEN institution_code = 'NED' THEN 'NATIONAL_CITIZEN_EDU_COMMITTEE'
                    WHEN institution_code = 'CEDU' THEN 'CITY_CITIZEN_EDU_COMMITTEE'
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
                ON subjects (province_code, city_code, kind, status, cid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_town
                ON subjects (province_code, city_code, town_code, kind, status, cid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_scope_created
                ON subjects (category, province_name, city_name, created_at DESC, cid_number DESC);
             CREATE INDEX IF NOT EXISTS idx_subjects_exact_lookup
                ON subjects (category, province_name, city_name, cid_number, cid_full_name, cid_short_name);
             CREATE INDEX IF NOT EXISTS idx_subjects_legal_rep
                ON subjects (province_code, legal_rep_cid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_education
                ON subjects (province_code, city_code, institution_code, education_type, status);
             CREATE INDEX IF NOT EXISTS idx_citizens_scope_created
                ON citizens (province_code, city_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_province_created
                ON citizens (province_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_exact_lookup
                ON citizens (province_code, city_code, archive_no, cid_number, wallet_pubkey, wallet_address);
             CREATE INDEX IF NOT EXISTS idx_citizens_residence_scope
                ON citizens (residence_province_code, residence_city_code, residence_town_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_birth_scope
                ON citizens (birth_province_code, birth_city_code, birth_town_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_gov_city
                ON gov (province_code, city_code, institution_code);
             CREATE INDEX IF NOT EXISTS idx_private_city
                ON private (province_code, city_code, private_type, code);
             CREATE INDEX IF NOT EXISTS idx_accounts_cid
                ON accounts (province_code, cid_number);
             CREATE INDEX IF NOT EXISTS idx_docs_cid
                ON docs (province_code, cid_number, uploaded_at DESC);
             CREATE INDEX IF NOT EXISTS idx_audit_scope_time
                ON audit (province_code, city_code, created_at DESC);",
        )
        .map_err(|e| {
            format!(
                "init subject partition indexes failed: {}",
                postgres_error_text(&e)
            )
        })?;

        for province in crate::china::provinces().iter() {
            Self::create_subject_partitions(conn, province.province_code)?;
        }
        Self::delete_ineligible_citizen_residuals(conn)?;
        Ok(())
    }

    fn delete_ineligible_citizen_residuals(conn: &mut postgres::Client) -> Result<(), String> {
        // 中文注释:CID 公民库目标状态是“可投票公民库”;历史残留的注销、无选举资格或无钱包记录不再保留。
        conn.batch_execute(
            "WITH doomed AS (
                SELECT province_code, cid_number, archive_no
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
                WHERE s.province_code = d.province_code
                  AND s.cid_number = d.cid_number
                  AND s.kind = 'CITIZEN'
                RETURNING 1
             ),
             deleted_ids AS (
                DELETE FROM ids i
                USING doomed d
                WHERE i.cid_number = d.cid_number
                  AND i.kind = 'CITIZEN'
                RETURNING 1
             )
             DELETE FROM citizens c
             USING doomed d
             WHERE c.province_code = d.province_code
               AND c.cid_number = d.cid_number;",
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
            "cid_full_name",
            "cid_short_name",
            "legal_rep_name",
            "legal_rep_cid_number",
            "legal_rep_photo_path",
            "legal_rep_photo_name",
            "legal_rep_photo_mime",
            "legal_rep_photo_size",
            "private_type",
            "partnership_kind",
            "has_legal_personality",
            "education_type",
        ] {
            Self::ensure_column_state(conn, "subjects", column, true)?;
        }
        Self::ensure_column_state(conn, "subjects", "name", false)?;
        for column in ["private_type", "partnership_kind", "has_legal_personality"] {
            Self::ensure_column_state(conn, "private", column, true)?;
        }
        for column in [
            "residence_province_code",
            "residence_city_code",
            "residence_town_code",
            "birth_province_code",
            "birth_city_code",
            "birth_town_code",
            "election_scope_level",
        ] {
            Self::ensure_column_state(conn, "citizens", column, true)?;
        }
        Self::ensure_column_state(conn, "gov", "source", true)?;
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

    fn create_subject_partitions(
        conn: &mut postgres::Client,
        province_code: &str,
    ) -> Result<(), String> {
        for table in crate::subjects::schema::PARTITIONED_TABLES {
            let partition_name = format!("{}_{}", table, province_code.to_ascii_lowercase());
            let sql = format!(
                "CREATE TABLE IF NOT EXISTS {partition_name} PARTITION OF {table} FOR VALUES IN ('{province_code}')"
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
