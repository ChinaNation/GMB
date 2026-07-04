#!/usr/bin/env bash
# Card 05 打包前置(macOS / Linux):把 onchina 二进制 + 前端产物 + china.sqlite + PostgreSQL
# 官方二进制 + 创世链状态包组装到 node/{binaries,resources}。之后在 node/ 跑
# `npm run tauri build` 产安装包。
#
# 用法:
#   export CITIZENCHAIN_PG_DIST=<postgresql.org 官方二进制解压目录(含 bin/lib/share)>
#   export CITIZENCHAIN_GENESIS_STATE_DIR=<bake-chainspec.sh 生成的 genesis-state 目录>
#   citizenchain/scripts/prepack.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"   # citizenchain/scripts
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"          # citizenchain/
HERE="$ROOT/node"                             # citizenchain/node
case "$(uname -s)" in
  Darwin) OS=macos ;;
  Linux) OS=linux ;;
  *) OS=linux ;;
esac

echo "[prepack] build onchina (release)"
( cd "$ROOT" && cargo build -p onchina --release )

echo "[prepack] build onchina frontend"
( cd "$ROOT/onchina/frontend" && npm ci && npm run build )

echo "[prepack] assemble node/resources"
mkdir -p "$HERE/resources/onchina-bin" "$HERE/resources/onchina-frontend" "$HERE/resources/postgres" "$HERE/resources/genesis-state"
# onchina 二进制随包(Tauri resources/onchina-bin),onchina_proc 从资源目录解析(见 node/src/onchina_proc)。
cp "$ROOT/target/release/onchina" "$HERE/resources/onchina-bin/onchina"
chmod +x "$HERE/resources/onchina-bin/onchina"
rm -rf "$HERE/resources/onchina-frontend/dist"
cp -R "$ROOT/onchina/frontend/dist" "$HERE/resources/onchina-frontend/dist"

# PostgreSQL 官方二进制(postgresql.org):把已解压的 PG 安装目录(含 bin/lib/share)
# 指向 CITIZENCHAIN_PG_DIST,脚本拷进 resources/postgres/$OS;未提供则告警(安装包将缺内嵌 PG)。
if [ -n "${CITIZENCHAIN_PG_DIST:-}" ] && [ -d "$CITIZENCHAIN_PG_DIST/bin" ]; then
  rm -rf "$HERE/resources/postgres/$OS"
  mkdir -p "$HERE/resources/postgres/$OS"
  cp -R "$CITIZENCHAIN_PG_DIST/." "$HERE/resources/postgres/$OS/"
  echo "[prepack] PostgreSQL 已组装($OS)"
else
  echo "[prepack][warn] 未提供 CITIZENCHAIN_PG_DIST。"
  echo "                请从 https://www.postgresql.org/download/ 取本平台官方二进制(含 bin/lib/share),"
  echo "                解压后 export CITIZENCHAIN_PG_DIST=<解压目录> 再重跑;否则安装包不含内嵌 PG。"
fi

# 创世链状态包来自 bake-chainspec.sh 的输出,是正式安装包首启免全量物化的基础。
GENESIS_STATE_SOURCE="${CITIZENCHAIN_GENESIS_STATE_DIR:-$ROOT/target/chainspec/genesis-state}"
if [ -f "$GENESIS_STATE_SOURCE/manifest.json" ] && [ -d "$GENESIS_STATE_SOURCE/chains/citizenchain/db" ]; then
  rm -rf "$HERE/resources/genesis-state"
  mkdir -p "$HERE/resources/genesis-state"
  cp -R "$GENESIS_STATE_SOURCE/." "$HERE/resources/genesis-state/"
  echo "[prepack] 创世链状态包已组装:$GENESIS_STATE_SOURCE"
else
  echo "[prepack][warn] 未找到创世链状态包:$GENESIS_STATE_SOURCE"
  echo "                正式安装包必须先执行 bake-chainspec.sh --finalize --wasm <CI_WASM>,"
  echo "                并让 CITIZENCHAIN_GENESIS_STATE_DIR 指向生成的 genesis-state 目录。"
fi

echo "[prepack] done. 接着在 node/ 执行: npm run tauri build"
