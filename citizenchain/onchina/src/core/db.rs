//! OnChina 结构化数据库入口。
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

/// postgres::Error 在部分数据库错误上只显示 `db error`。
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
        let pool_size = std::env::var("ONCHINA_PG_POOL_SIZE")
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
            "-- 机构/账户「注册局域注销态」+ 已签发注销凭证(区别于链投影 chain_status）。
             -- 注册局管理员发起注销(冷签特殊档）后写 ISSUED;机构管理员持凭证上链 propose_close,
             -- indexer 收到链上关闭后置 ONCHAIN_CLOSED(投影子项）。见 ADR-023 §6.3。
             -- 链交易冷签会话(ADR-031 D6/D7):prepare 落库,submit 单次消费;
             -- 占号先行 = 链上进块后才建档,会话携带校验哈希防 runtime 漂移。
             CREATE TABLE IF NOT EXISTS chain_sign_sessions (
                request_id   TEXT PRIMARY KEY,
                purpose      TEXT NOT NULL,
                actor_pubkey TEXT NOT NULL,
                call_data    TEXT NOT NULL,
                nonce        BIGINT NOT NULL,
                signing_hash TEXT NOT NULL,
                context      JSONB NOT NULL,
                created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
                expires_at   TIMESTAMPTZ NOT NULL,
                consumed_at  TIMESTAMPTZ
             );
             CREATE INDEX IF NOT EXISTS idx_chain_sign_sessions_expiry
                ON chain_sign_sessions(expires_at) WHERE consumed_at IS NULL;

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
                institution_code TEXT NOT NULL,
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
             SET admin_name = ''
             WHERE admin_name IS NULL OR lower(admin_name) = lower(admin_account);
             ALTER TABLE admins
                ALTER COLUMN admin_account SET NOT NULL,
                ALTER COLUMN admin_name SET NOT NULL,
                ALTER COLUMN institution_code SET NOT NULL;
             CREATE UNIQUE INDEX IF NOT EXISTS admins_admin_account_key ON admins(admin_account);
             CREATE INDEX IF NOT EXISTS idx_admins_institution_code ON admins(institution_code);
             CREATE INDEX IF NOT EXISTS idx_admins_institution_code_city_name ON admins(institution_code, city_name);
             CREATE INDEX IF NOT EXISTS idx_admins_account_lower ON admins(lower(admin_account));
             CREATE INDEX IF NOT EXISTS idx_admins_created_by_lower ON admins(lower(created_by));

             -- 联邦注册局 215 名管理员按链上省级 5 人组归属缓存。
             -- 权限真源仍是 `PublicAdmins::FederalRegistryProvinceGroups`;本表只保存
             -- 列表展示和同省更换预检所需的省名。
             CREATE TABLE IF NOT EXISTS federal_registry_admin_scopes (
                admin_account TEXT PRIMARY KEY,
                province_name TEXT NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             CREATE INDEX IF NOT EXISTS idx_frg_admin_scopes_province
                ON federal_registry_admin_scopes(province_name);

             -- 节点机构归属由 active admin 首次登录绑定,行政区真源为 china.sqlite。

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

             -- 本节点首次由链上 active admin 确认后绑定唯一机构;链上管理员关系仍是真源。
             CREATE TABLE IF NOT EXISTS node_institution_bindings (
                binding_id TEXT PRIMARY KEY,
                candidate_id TEXT NOT NULL,
                institution_code TEXT NOT NULL,
                institution_cid_number TEXT,
                institution_main_account TEXT,
                frg_province_code TEXT,
                cid_full_name TEXT,
                cid_short_name TEXT,
                scope_province_name TEXT,
                scope_city_name TEXT,
                scope_town_name TEXT,
                bound_admin_pubkey TEXT NOT NULL,
                bound_at TIMESTAMPTZ NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'INACTIVE'))
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_node_binding_one_active
                ON node_institution_bindings ((status)) WHERE status = 'ACTIVE';

             CREATE TABLE IF NOT EXISTS node_binding_challenges (
                binding_challenge_id TEXT PRIMARY KEY,
                admin_account TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                consumed BOOLEAN NOT NULL DEFAULT FALSE,
                payload JSONB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_node_binding_challenges_expires
                ON node_binding_challenges(expires_at);

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
                institution_code TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                last_active_at TIMESTAMPTZ NOT NULL,
                payload JSONB NOT NULL
             );
             ALTER TABLE admin_sessions
                ALTER COLUMN admin_account SET NOT NULL,
                ALTER COLUMN institution_code SET NOT NULL;
             CREATE INDEX IF NOT EXISTS idx_admin_sessions_account
                ON admin_sessions(admin_account);
             CREATE INDEX IF NOT EXISTS idx_admin_sessions_expires
                ON admin_sessions(expires_at);

             -- 管理员已注册的 WebAuthn passkey 凭证(passkey 列存 webauthn-rs Passkey 序列化)。
             CREATE TABLE IF NOT EXISTS admin_passkey_credentials (
                credential_id TEXT PRIMARY KEY,
                admin_account TEXT NOT NULL,
                passkey JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkey_credentials_account
                ON admin_passkey_credentials(lower(admin_account));

             -- WebAuthn 注册/断言进行中的 ceremony 状态(一次性,5min TTL)。
             CREATE TABLE IF NOT EXISTS admin_passkey_ceremonies (
                ceremony_id TEXT PRIMARY KEY,
                admin_account TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('REG', 'AUTH')),
                state JSONB NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkey_ceremonies_expires
                ON admin_passkey_ceremonies(expires_at);

             -- passkey 断言一次性证明令牌(重要/特殊操作提交时消费,2min TTL)。
             CREATE TABLE IF NOT EXISTS admin_passkey_assertions (
                assertion_id TEXT PRIMARY KEY,
                admin_account TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_admin_passkey_assertions_expires
                ON admin_passkey_assertions(expires_at);

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

             CREATE TABLE IF NOT EXISTS chain_projection_state (
                projection_key TEXT PRIMARY KEY,
                chain_genesis_hash TEXT NOT NULL,
                chain_block_hash TEXT NOT NULL DEFAULT '',
                chain_block_number BIGINT,
                item_count BIGINT NOT NULL DEFAULT 0,
                account_count BIGINT NOT NULL DEFAULT 0,
                status TEXT NOT NULL CHECK (status IN ('OK', 'FAILED')),
                synced_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             DROP TABLE IF EXISTS gov_manifest;",
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

             -- 行政区名字单一真源是 china.sqlite,subjects 只存 province_code/
             -- city_code/town_code,名字由后端在拼装 DTO 时反查派生(ADR-021)。
             -- private_type/partnership_kind/has_legal_personality/p1/parent_cid_number 是
             -- 私权机构明细,单一真源是 private 表;subjects 暂保留这几列作为通用查询/列表展示的
             -- 冗余镜像(随机构 upsert 同写),后续可下线见 #4 决策。
             CREATE TABLE IF NOT EXISTS subjects (
                cid_number TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('CITIZEN', 'PUBLIC', 'PRIVATE')),
                cid_full_name TEXT,
                cid_short_name TEXT,
                status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'REVOKED')),
                category TEXT,
                p1 TEXT,
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
                legal_representative_account TEXT,
                issuer_cid_number TEXT,
                institution_source_type TEXT,
                register_proposal_id TEXT,
                chain_status TEXT,
                chain_tx_hash TEXT,
                chain_block_number BIGINT,
                created_by TEXT,
                updated_by TEXT,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (province_code, cid_number)
             ) PARTITION BY LIST (province_code);

             -- 机构管理员「链下私密资料」唯一归属表(ADR-030/A2)。
             -- 管理员姓名/职务/任期/cid/来源属链上 AdminProfile,**不在本表**;
             -- 本表只承接链下私密档案(部门/岗位/联系方式/证件照/passkey 绑定等)+ 链投影。
             -- 按 province_code 省级分区,复合主键 (province_code, cid_number, admin_account)。
             CREATE TABLE IF NOT EXISTS institution_admins (
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
                city_code TEXT,
                admin_account TEXT NOT NULL,
                admin_department TEXT,
                admin_job TEXT,
                admin_contact_phone TEXT,
                admin_contact_email TEXT,
                admin_photo_path TEXT,
                admin_photo_name TEXT,
                admin_photo_mime TEXT,
                admin_photo_size BIGINT,
                admin_passkey_credential_id TEXT,
                admin_source_id TEXT,
                admin_profile_status TEXT,
                admin_profile_updated_at TIMESTAMPTZ,
                created_by TEXT,
                chain_status TEXT,
                chain_tx_hash TEXT,
                chain_block_number BIGINT,
                operation_log_id TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (province_code, cid_number, admin_account)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS citizens (
                cid_number TEXT NOT NULL,
                passport_no TEXT NOT NULL DEFAULT '',
                citizen_family_name TEXT NOT NULL DEFAULT '',
                citizen_given_name TEXT NOT NULL DEFAULT '',
                citizen_sex TEXT NOT NULL DEFAULT '',
                citizen_birth_date TEXT NOT NULL DEFAULT '',
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                id BIGINT,
                wallet_pubkey TEXT,
                wallet_address TEXT,
                wallet_sig_alg TEXT,
                wallet_verified_at TIMESTAMPTZ,
                citizen_status TEXT NOT NULL,
                voting_eligible BOOLEAN NOT NULL,
                passport_valid_from TEXT NOT NULL DEFAULT '',
                passport_valid_until TEXT NOT NULL DEFAULT '',
                status_updated_at BIGINT,
                town_code TEXT NOT NULL DEFAULT '',
                birth_province_code TEXT NOT NULL DEFAULT '',
                birth_city_code TEXT NOT NULL DEFAULT '',
                birth_town_code TEXT NOT NULL DEFAULT '',
                archive_hash TEXT,
                onchain_tx_hash TEXT,
                onchain_block_number BIGINT,
                onchain_at TIMESTAMPTZ,
                created_by TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_by TEXT,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (province_code, cid_number)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS citizen_documents (
                id BIGSERIAL,
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                file_name TEXT NOT NULL,
                document_type TEXT NOT NULL,
                file_size BIGINT NOT NULL DEFAULT 0,
                file_path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                uploaded_by TEXT NOT NULL,
                uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (province_code, id)
             ) PARTITION BY LIST (province_code);

             CREATE TABLE IF NOT EXISTS sequence_counters (
                seq_key TEXT PRIMARY KEY,
                next_seq BIGINT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS passport_numbers (
                passport_no TEXT PRIMARY KEY,
                cid_number TEXT NOT NULL UNIQUE,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );

             CREATE TABLE IF NOT EXISTS passport_number_recycle_pool (
                pool_id TEXT PRIMARY KEY,
                passport_no TEXT NOT NULL,
                source_cid_number TEXT NOT NULL DEFAULT '',
                deleted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                released_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                used_at TIMESTAMPTZ,
                used_by_cid_number TEXT
             );
             CREATE INDEX IF NOT EXISTS idx_passport_number_recycle_available
                ON passport_number_recycle_pool (released_at, pool_id) WHERE used_at IS NULL;

             CREATE TABLE IF NOT EXISTS gov (
                cid_number TEXT NOT NULL,
                province_code TEXT NOT NULL,
		                city_code TEXT,
		                town_code TEXT,
		                institution_code TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'CHAIN' CHECK (source IN ('CHAIN', 'MANUAL')),
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

             -- 审计 detail 由自由文本改结构化 JSONB(事实与展示分离,
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
            "UPDATE subjects
             SET category = 'GOV_INSTITUTION'
             WHERE category = ('PUBLIC_' || 'SECURITY');",
        )
        .map_err(|e| {
            format!(
                "fold legacy public security category failed: {}",
                postgres_error_text(&e)
            )
        })?;

        conn.batch_execute(
            "ALTER TABLE gov
                DROP CONSTRAINT IF EXISTS gov_source_check;
             UPDATE gov
             SET source = 'CHAIN'
             WHERE source = 'GENERATED';
             ALTER TABLE gov
                ALTER COLUMN source SET DEFAULT 'CHAIN';
             ALTER TABLE gov
                ADD CONSTRAINT gov_source_check
                CHECK (source IN ('CHAIN', 'MANUAL'));
             UPDATE gov g
             SET source = 'CHAIN'
             FROM subjects s
             WHERE s.province_code = g.province_code
               AND s.cid_number = g.cid_number
               AND s.kind = 'PUBLIC'
               AND s.created_by = 'SYSTEM'
               AND s.category = 'GOV_INSTITUTION'
               AND g.source IS DISTINCT FROM 'CHAIN';",
        )
        .map_err(|e| {
            format!(
                "sync gov source boundary failed: {}",
                postgres_error_text(&e)
            )
        })?;

        conn.batch_execute(
            "ALTER TABLE citizens
                ADD COLUMN IF NOT EXISTS passport_no TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS citizen_family_name TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS citizen_given_name TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS citizen_sex TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS citizen_birth_date TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS wallet_sig_alg TEXT,
                ADD COLUMN IF NOT EXISTS wallet_verified_at TIMESTAMPTZ,
                ADD COLUMN IF NOT EXISTS passport_valid_from TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS passport_valid_until TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS town_code TEXT,
                ADD COLUMN IF NOT EXISTS residence_town_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_province_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_city_code TEXT,
                ADD COLUMN IF NOT EXISTS birth_town_code TEXT,
                ADD COLUMN IF NOT EXISTS archive_hash TEXT,
                ADD COLUMN IF NOT EXISTS onchain_tx_hash TEXT,
                ADD COLUMN IF NOT EXISTS onchain_block_number BIGINT,
                ADD COLUMN IF NOT EXISTS onchain_at TIMESTAMPTZ,
                ADD COLUMN IF NOT EXISTS created_by TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS updated_by TEXT,
                ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
             ALTER TABLE citizens
                DROP CONSTRAINT IF EXISTS citizens_election_scope_level_check,
                DROP CONSTRAINT IF EXISTS citizens_bind_status_check;
             ALTER TABLE citizens
                DROP COLUMN IF EXISTS valid_from,
                DROP COLUMN IF EXISTS valid_until,
                DROP COLUMN IF EXISTS election_scope_level,
                DROP COLUMN IF EXISTS bind_status,
                DROP COLUMN IF EXISTS bound_at,
                DROP COLUMN IF EXISTS bound_by,
                DROP COLUMN IF EXISTS citizen_full_name,
                DROP COLUMN IF EXISTS residence_province_code,
                DROP COLUMN IF EXISTS residence_city_code;
             ALTER TABLE citizens
                ALTER COLUMN wallet_pubkey DROP NOT NULL,
                ALTER COLUMN wallet_address DROP NOT NULL,
                ALTER COLUMN wallet_sig_alg DROP NOT NULL,
                ALTER COLUMN wallet_pubkey DROP DEFAULT,
                ALTER COLUMN wallet_address DROP DEFAULT,
                ALTER COLUMN wallet_sig_alg DROP DEFAULT;
             UPDATE citizens
             SET birth_province_code = COALESCE(NULLIF(birth_province_code, ''), province_code),
                 birth_city_code = COALESCE(NULLIF(birth_city_code, ''), city_code),
                 birth_town_code = COALESCE(birth_town_code, ''),
                 wallet_pubkey = NULLIF(wallet_pubkey, ''),
                 wallet_address = NULLIF(wallet_address, ''),
                 wallet_sig_alg = CASE
                    WHEN NULLIF(wallet_pubkey, '') IS NULL THEN NULL
                    ELSE COALESCE(NULLIF(wallet_sig_alg, ''), 'sr25519')
                 END,
                 passport_no = COALESCE(passport_no, ''),
                 citizen_family_name = COALESCE(citizen_family_name, ''),
                 citizen_given_name = COALESCE(citizen_given_name, ''),
                 citizen_sex = COALESCE(citizen_sex, ''),
                 citizen_birth_date = COALESCE(citizen_birth_date, ''),
                 town_code = COALESCE(NULLIF(town_code, ''), NULLIF(residence_town_code, ''), ''),
                 passport_valid_from = COALESCE(passport_valid_from, ''),
                 passport_valid_until = COALESCE(passport_valid_until, ''),
                 updated_at = COALESCE(updated_at, created_at, now());
             ALTER TABLE citizens
                ALTER COLUMN town_code SET DEFAULT '',
                ALTER COLUMN town_code SET NOT NULL,
                ALTER COLUMN birth_province_code SET NOT NULL,
                ALTER COLUMN birth_city_code SET NOT NULL,
                ALTER COLUMN birth_town_code SET DEFAULT '',
                ALTER COLUMN birth_town_code SET NOT NULL;
             ALTER TABLE citizens
                DROP COLUMN IF EXISTS residence_town_code;",
        )
        .map_err(|e| {
            format!(
                "sync target citizen schema failed: {}",
                postgres_error_text(&e)
            )
        })?;

        conn.batch_execute(
            "CREATE TABLE IF NOT EXISTS sequence_counters (
                seq_key TEXT PRIMARY KEY,
                next_seq BIGINT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS passport_numbers (
                passport_no TEXT PRIMARY KEY,
                cid_number TEXT NOT NULL UNIQUE,
                province_code TEXT NOT NULL,
                city_code TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
             );
             CREATE TABLE IF NOT EXISTS passport_number_recycle_pool (
                pool_id TEXT PRIMARY KEY,
                passport_no TEXT NOT NULL,
                source_cid_number TEXT NOT NULL DEFAULT '',
                deleted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                released_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                used_at TIMESTAMPTZ,
                used_by_cid_number TEXT
             );
             CREATE INDEX IF NOT EXISTS idx_passport_number_recycle_available
                ON passport_number_recycle_pool (released_at, pool_id) WHERE used_at IS NULL;",
        )
        .map_err(|e| {
            format!(
                "sync passport number schema failed: {}",
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

        // subjects 机构级链投影 + 溯源补列(幂等增列,可重复执行)。
        conn.batch_execute(
            "ALTER TABLE subjects
                ADD COLUMN IF NOT EXISTS updated_by TEXT,
                ADD COLUMN IF NOT EXISTS issuer_cid_number TEXT,
                ADD COLUMN IF NOT EXISTS institution_source_type TEXT,
                ADD COLUMN IF NOT EXISTS register_proposal_id TEXT,
                ADD COLUMN IF NOT EXISTS legal_representative_account TEXT,
                ADD COLUMN IF NOT EXISTS chain_status TEXT,
                ADD COLUMN IF NOT EXISTS chain_tx_hash TEXT,
                ADD COLUMN IF NOT EXISTS chain_block_number BIGINT;",
        )
        .map_err(|e| {
            format!(
                "add subjects chain/provenance columns failed: {}",
                postgres_error_text(&e)
            )
        })?;

        // 行政区名字单一真源是 china.sqlite,subjects 不再落地名字副本;
        // 已有部署里的派生名字列幂等删除(分区父表 DROP 自动级联各省分区)。
        conn.batch_execute(
            "ALTER TABLE subjects
                DROP COLUMN IF EXISTS province_name,
                DROP COLUMN IF EXISTS city_name,
                DROP COLUMN IF EXISTS town_name;",
        )
        .map_err(|e| {
            format!(
                "drop derived geo name columns from subjects failed: {}",
                postgres_error_text(&e)
            )
        })?;

        Self::validate_target_subject_schema(conn)?;

        // 教育委员会从公权目录迁入教育机构 tab 后,已生成的国家/市公民教育委员会
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
                ON subjects (category, province_code, city_code, created_at DESC, cid_number DESC);
             CREATE INDEX IF NOT EXISTS idx_subjects_exact_lookup
                ON subjects (category, province_code, city_code, cid_number, cid_full_name, cid_short_name);
             CREATE INDEX IF NOT EXISTS idx_subjects_legal_rep
                ON subjects (province_code, legal_rep_cid_number);
             CREATE INDEX IF NOT EXISTS idx_subjects_education
                ON subjects (province_code, city_code, institution_code, education_type, status);
             CREATE INDEX IF NOT EXISTS idx_citizens_scope_created
                ON citizens (province_code, city_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_province_created
                ON citizens (province_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_exact_lookup
                ON citizens (province_code, city_code, cid_number, passport_no, wallet_pubkey, wallet_address);
             CREATE INDEX IF NOT EXISTS idx_citizens_passport_no
                ON citizens (passport_no);
             DROP INDEX IF EXISTS idx_citizens_wallet_pubkey;
             CREATE INDEX IF NOT EXISTS idx_citizens_wallet_pubkey
                ON citizens (lower(wallet_pubkey)) WHERE wallet_pubkey IS NOT NULL;
             DROP INDEX IF EXISTS idx_citizens_residence_scope;
             CREATE INDEX IF NOT EXISTS idx_citizens_town_scope
                ON citizens (province_code, city_code, town_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizens_birth_scope
                ON citizens (birth_province_code, birth_city_code, birth_town_code, created_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizen_documents_cid
                ON citizen_documents (province_code, cid_number, uploaded_at DESC, id DESC);
             CREATE INDEX IF NOT EXISTS idx_citizen_documents_type
                ON citizen_documents (province_code, cid_number, document_type);
             CREATE INDEX IF NOT EXISTS idx_gov_city
                ON gov (province_code, city_code, institution_code);
             CREATE INDEX IF NOT EXISTS idx_private_city
                ON private (province_code, city_code, private_type, code);
             CREATE INDEX IF NOT EXISTS idx_accounts_cid
                ON accounts (province_code, cid_number);
             CREATE INDEX IF NOT EXISTS idx_docs_cid
                ON docs (province_code, cid_number, uploaded_at DESC);
             CREATE INDEX IF NOT EXISTS idx_audit_scope_time
                ON audit (province_code, city_code, created_at DESC);
             CREATE INDEX IF NOT EXISTS idx_institution_admins_cid
                ON institution_admins (province_code, cid_number);
             CREATE INDEX IF NOT EXISTS idx_institution_admins_account
                ON institution_admins (province_code, lower(admin_account));",
        )
        .map_err(|e| {
            format!(
                "init subject partition indexes failed: {}",
                postgres_error_text(&e)
            )
        })?;

        for province in crate::cid::china::provinces().iter() {
            Self::create_subject_partitions(conn, province.province_code)?;
        }
        Self::delete_ineligible_citizen_residuals(conn)?;
        Ok(())
    }

    fn delete_ineligible_citizen_residuals(conn: &mut postgres::Client) -> Result<(), String> {
        // 公民改为注册局直接录入并直接发护照——NORMAL 且在有效期内即已签发护照,
        // 选举资格(voting_eligible)与钱包绑定均为可选项,不再作为是否保留的判据。
        // 这里只清理身份已注销(citizen_status <> 'NORMAL')的历史残留索引行。
        conn.batch_execute(
            "WITH doomed AS (
                SELECT province_code, cid_number
                FROM citizens
                WHERE citizen_status <> 'NORMAL'
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
             ),
             deleted_citizen_documents AS (
                DELETE FROM citizen_documents cd
                USING doomed d
                WHERE cd.province_code = d.province_code
                  AND cd.cid_number = d.cid_number
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

    // 把启动期失败提前到清晰的目标状态校验,避免后续索引或业务 SQL 报隐晦字段错误。
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
        // 行政区名字已收口 china.sqlite,subjects 必须无名字副本列。
        for column in ["province_name", "city_name", "town_name"] {
            Self::ensure_column_state(conn, "subjects", column, false)?;
        }
        // 机构级链投影 + 溯源补列必须就位。
        for column in [
            "issuer_cid_number",
            "institution_source_type",
            "register_proposal_id",
            "legal_representative_account",
            "chain_status",
            "updated_by",
        ] {
            Self::ensure_column_state(conn, "subjects", column, true)?;
        }
        for column in ["private_type", "partnership_kind", "has_legal_personality"] {
            Self::ensure_column_state(conn, "private", column, true)?;
        }
        for column in [
            "passport_no",
            "citizen_family_name",
            "citizen_given_name",
            "citizen_sex",
            "citizen_birth_date",
            "wallet_sig_alg",
            "passport_valid_from",
            "passport_valid_until",
            "town_code",
            "birth_province_code",
            "birth_city_code",
            "birth_town_code",
            "archive_hash",
            "created_by",
            "updated_at",
        ] {
            Self::ensure_column_state(conn, "citizens", column, true)?;
        }
        for column in [
            "valid_from",
            "valid_until",
            "election_scope_level",
            "bind_status",
            "bound_at",
            "bound_by",
            "citizen_full_name",
            "residence_province_code",
            "residence_city_code",
            "residence_town_code",
        ] {
            Self::ensure_column_state(conn, "citizens", column, false)?;
        }
        for column in [
            "cid_number",
            "province_code",
            "city_code",
            "file_name",
            "document_type",
            "file_size",
            "file_path",
            "file_hash",
            "uploaded_by",
            "uploaded_at",
        ] {
            Self::ensure_column_state(conn, "citizen_documents", column, true)?;
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
        for table in crate::institution::subjects::schema::PARTITIONED_TABLES {
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
