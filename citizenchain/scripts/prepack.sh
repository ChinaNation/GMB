#!/usr/bin/env bash
# Card 05 打包前置(macOS / Linux):把 registry 二进制 + 前端产物 + china.sqlite + PostgreSQL
# 官方二进制组装到 node/{binaries,resources}。之后在 node/ 跑 `npm run tauri build` 产安装包。
#
# 用法:
#   export CITIZENCHAIN_PG_DIST=<postgresql.org 官方二进制解压目录(含 bin/lib/share)>
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

echo "[prepack] build registry (release)"
( cd "$ROOT" && cargo build -p registry --release )

echo "[prepack] build registry frontend"
( cd "$ROOT/registry/frontend" && npm ci && npm run build )

echo "[prepack] assemble node/resources"
mkdir -p "$HERE/resources/registry-bin" "$HERE/resources/registry-frontend" "$HERE/resources/postgres"
# registry 二进制随包(Tauri resources/registry-bin),registry_proc 从资源目录解析(见 node/src/registry_proc)。
cp "$ROOT/target/release/registry" "$HERE/resources/registry-bin/registry"
chmod +x "$HERE/resources/registry-bin/registry"
cp "$ROOT/registry/src/china/china.sqlite" "$HERE/resources/china.sqlite"
rm -rf "$HERE/resources/registry-frontend/dist"
cp -R "$ROOT/registry/frontend/dist" "$HERE/resources/registry-frontend/dist"

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

echo "[prepack] done. 接着在 node/ 执行: npm run tauri build"
