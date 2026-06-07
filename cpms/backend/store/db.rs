use sqlx::PgPool;

/// CPMS PostgreSQL Store。
///
/// 中文注释:只持有数据库连接池,所有业务数据仍由各模块自己的表承载。
#[derive(Clone)]
pub(crate) struct StoreDb {
    pool: PgPool,
}

impl StoreDb {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 清理登录和扫码登录短期状态。
    ///
    /// 中文注释:CPMS 不需要 Redis;离线系统用本机 PostgreSQL 表 + 定时清理即可。
    pub(crate) async fn cleanup_auth_runtime(&self, now_ts: i64) {
        let _ = sqlx::query("DELETE FROM sessions WHERE expires_at < $1")
            .bind(now_ts)
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("DELETE FROM login_challenges WHERE expire_at < $1")
            .bind(now_ts)
            .execute(&self.pool)
            .await;
        let cutoff = now_ts - 600;
        let _ = sqlx::query("DELETE FROM qr_login_results WHERE created_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await;
    }
}
