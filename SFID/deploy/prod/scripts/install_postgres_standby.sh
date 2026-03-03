#!/usr/bin/env bash
set -euo pipefail

# Ubuntu/Debian 示例脚本：备库初始化（流复制）

PG_VER="16"
PRIMARY_IP="10.0.0.11"
REPL_USER="replicator"
REPL_PASS="CHANGE_ME_REPL_PASS"

if [[ "${REPL_PASS}" == *"CHANGE_ME"* ]]; then
  echo "REPL_PASS 仍为占位值(CHANGE_ME_REPL_PASS)，请先替换。"
  exit 1
fi

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y postgresql-${PG_VER} postgresql-client-${PG_VER}

PGDATA="/var/lib/postgresql/${PG_VER}/main"
PGCONF="/etc/postgresql/${PG_VER}/main/postgresql.conf"

systemctl stop postgresql
rm -rf "${PGDATA}"/*

export PGPASSWORD="${REPL_PASS}"
sudo -u postgres pg_basebackup \
  -h "${PRIMARY_IP}" \
  -D "${PGDATA}" \
  -U "${REPL_USER}" \
  -Fp -Xs -P -R
unset PGPASSWORD

# 开启热备只读
if ! grep -q "^hot_standby" "${PGCONF}"; then echo "hot_standby = on" >> "${PGCONF}"; else sed -i "s/^hot_standby.*/hot_standby = on/" "${PGCONF}"; fi

chown -R postgres:postgres "${PGDATA}"
chmod 700 "${PGDATA}"

systemctl start postgresql

echo "备库启动完成。验证命令: sudo -u postgres psql -c 'select pg_is_in_recovery();'"
