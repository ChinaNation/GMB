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

validate_genesis_state_package() {
  local package_root="$1" path relative
  [[ -f "$package_root/manifest.json" && -d "$package_root/chains/citizenchain/db" ]] || return 1
  while IFS= read -r -d '' path; do
    relative="${path#"$package_root"/}"
    [[ ! -L "$path" ]] || { echo "[prepack][error] 创世状态包禁止符号链接:$relative" >&2; return 1; }
    case "$relative" in
      manifest.json|chains|chains/citizenchain|chains/citizenchain/db|chains/citizenchain/db/*) ;;
      *) echo "[prepack][error] 创世状态包包含白名单外残留:$relative" >&2; return 1 ;;
    esac
  done < <(find "$package_root" -mindepth 1 -print0)
  python3 - "$package_root/manifest.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as f:
    manifest = json.load(f)
required = ("genesis_hash", "state_root", "chainspec_hash", "runtime_wasm_hash", "runtime_wasm_ci_run_id", "runtime_wasm_ci_head_sha", "light_sync_state_hash", "public_institution_root")
missing = [key for key in required if not manifest.get(key)]
if manifest.get("package_format") != "citizenchain-genesis-state-v1" or manifest.get("chain_id") != "citizenchain":
    raise SystemExit("创世状态包 manifest 身份无效")
if manifest.get("included_paths") != ["chains/citizenchain/db"]:
    raise SystemExit("创世状态包 manifest.included_paths 无效")
if missing:
    raise SystemExit(f"创世状态包 manifest 缺少字段:{','.join(missing)}")
PY
}

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
if validate_genesis_state_package "$GENESIS_STATE_SOURCE"; then
  rm -rf "$HERE/resources/genesis-state"
  mkdir -p "$HERE/resources/genesis-state/chains/citizenchain"
  install -m 0644 "$GENESIS_STATE_SOURCE/manifest.json" "$HERE/resources/genesis-state/manifest.json"
  cp -a "$GENESIS_STATE_SOURCE/chains/citizenchain/db" "$HERE/resources/genesis-state/chains/citizenchain/db"
  echo "[prepack] 创世链状态包已组装:$GENESIS_STATE_SOURCE"
else
  echo "[prepack][error] 创世链状态包缺失、字段无效或包含白名单外残留:$GENESIS_STATE_SOURCE" >&2
  echo "                 正式安装包必须先执行 bake-chainspec.sh --finalize --wasm <CI_WASM> --wasm-ci-run-id <RUN_ID> --wasm-ci-head-sha <HEAD_SHA>。" >&2
  exit 1
fi

echo "[prepack] done. 接着在 node/ 执行: npm run tauri build"
