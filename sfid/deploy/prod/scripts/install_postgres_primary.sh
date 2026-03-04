#!/usr/bin/env bash
set -euo pipefail

# Ubuntu/Debian 示例脚本：主库初始化 + 复制账号 + 角色最小化
# 执行前请按实际环境修改变量

PG_VER="16"
REPL_USER="${REPL_USER:-replicator}"
REPL_PASS="${REPL_PASS:-}"
STANDBY_IP="${STANDBY_IP:-10.0.0.22}"
PRIMARY_BIND_ADDR="${PRIMARY_BIND_ADDR:-10.0.0.21}"
APP_DB="${APP_DB:-sfid_prod}"
APP_ROLE="${APP_ROLE:-sfid_app}"
APP_PASS="${APP_PASS:-}"
MIG_ROLE="${MIG_ROLE:-sfid_migrator}"
MIG_PASS="${MIG_PASS:-}"

for secret_name in REPL_PASS APP_PASS MIG_PASS; do
  secret_value="${!secret_name:-}"
  if [[ -z "${secret_value}" ]]; then
    echo "${secret_name} 未设置，请通过环境变量传入后再执行。"
    exit 1
  fi
done

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y postgresql-${PG_VER} postgresql-client-${PG_VER}

PGDATA="/var/lib/postgresql/${PG_VER}/main"
PGCONF="/etc/postgresql/${PG_VER}/main/postgresql.conf"
PGHBA="/etc/postgresql/${PG_VER}/main/pg_hba.conf"

# 主库复制参数
sed -i "s/^#\?listen_addresses.*/listen_addresses = '${PRIMARY_BIND_ADDR}'/" "${PGCONF}"
if ! grep -q "^wal_level" "${PGCONF}"; then echo "wal_level = replica" >> "${PGCONF}"; else sed -i "s/^wal_level.*/wal_level = replica/" "${PGCONF}"; fi
if ! grep -q "^max_wal_senders" "${PGCONF}"; then echo "max_wal_senders = 10" >> "${PGCONF}"; else sed -i "s/^max_wal_senders.*/max_wal_senders = 10/" "${PGCONF}"; fi
if ! grep -q "^max_replication_slots" "${PGCONF}"; then echo "max_replication_slots = 10" >> "${PGCONF}"; else sed -i "s/^max_replication_slots.*/max_replication_slots = 10/" "${PGCONF}"; fi
if ! grep -q "^hot_standby" "${PGCONF}"; then echo "hot_standby = on" >> "${PGCONF}"; else sed -i "s/^hot_standby.*/hot_standby = on/" "${PGCONF}"; fi

# 放行备库复制连接
if ! grep -q "${STANDBY_IP}/32" "${PGHBA}"; then
  echo "host replication ${REPL_USER} ${STANDBY_IP}/32 scram-sha-256" >> "${PGHBA}"
fi

systemctl restart postgresql

sudo -u postgres psql -v ON_ERROR_STOP=1 <<SQL
DO
\$\$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '${REPL_USER}') THEN
    CREATE ROLE ${REPL_USER} WITH REPLICATION LOGIN PASSWORD '${REPL_PASS}';
  ELSE
    ALTER ROLE ${REPL_USER} WITH REPLICATION LOGIN PASSWORD '${REPL_PASS}';
  END IF;

  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '${MIG_ROLE}') THEN
    CREATE ROLE ${MIG_ROLE} WITH LOGIN PASSWORD '${MIG_PASS}';
  ELSE
    ALTER ROLE ${MIG_ROLE} WITH LOGIN PASSWORD '${MIG_PASS}';
  END IF;

  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '${APP_ROLE}') THEN
    CREATE ROLE ${APP_ROLE} WITH LOGIN PASSWORD '${APP_PASS}';
  ELSE
    ALTER ROLE ${APP_ROLE} WITH LOGIN PASSWORD '${APP_PASS}';
  END IF;
END
\$\$;

CREATE DATABASE ${APP_DB};
\c ${APP_DB}

GRANT CONNECT ON DATABASE ${APP_DB} TO ${APP_ROLE}, ${MIG_ROLE};
GRANT USAGE ON SCHEMA public TO ${APP_ROLE}, ${MIG_ROLE};
GRANT CREATE ON SCHEMA public TO ${MIG_ROLE};
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO ${APP_ROLE};
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ${APP_ROLE};
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE ON TABLES TO ${APP_ROLE};
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO ${APP_ROLE};
SQL

echo "主库配置完成。请继续在备库执行 install_postgres_standby.sh"
