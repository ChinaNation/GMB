#!/usr/bin/env bash
# 烘焙 CitizenChain 冻结 chainspec(plain 形态,ADR-031 D5)。
#
# 当前创世只直铸国家/省/市公权机构;镇级和新增机构运行期注册上链。
# 冻结 SSOT 为 plain JSON(runtime WASM + genesis patch + bootnodes)。脚本启动临时节点物化块 0,
# 同时导出安装包内置的 genesis-state 链数据库包;CitizenApp/smoldot 用 stateRootHash 轻形态。
#
# 默认模式只生成预览文件到 target/chainspec,不覆盖冻结 SSOT。
# 正式创世必须在 GitHub WASM CI 成功后执行:
#   citizenchain/scripts/bake-chainspec.sh --finalize \
#     --wasm /path/to/citizenchain.compact.compressed.wasm \
#     --wasm-ci-run-id <RUN_ID> --wasm-ci-head-sha <HEAD_SHA>
#
# 正式模式会同步:
#   1. citizenchain/node/chainspecs/citizenchain.plain.json   (节点冻结 SSOT)
#   2. citizenapp/assets/chainspec.json                        (smoldot 轻形态:stateRootHash)
#   3. citizenapp/assets/light_sync_state.json                 (smoldot checkpoint)
#
# 流程:导出 plain spec → 临时节点物化创世(记录耗时)→ RPC 宪法创世检查
#       → 读块 0 头生成轻形态 → 生成 lightSyncState → 导出 genesis-state → finalize 同步。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(dirname "$CHAIN_ROOT")"
OUT="$CHAIN_ROOT/target/chainspec/citizenchain.plain.json"
APP_OUT="$CHAIN_ROOT/target/chainspec/chainspec.app.json"
APP_LIGHT_SYNC_STATE_OUT="$CHAIN_ROOT/target/chainspec/light_sync_state.json"
GENESIS_STATE_OUT="$CHAIN_ROOT/target/chainspec/genesis-state"
FINALIZE=0
SKIP_CHECK=0
WASM_FILE_ARG=""
WASM_CI_RUN_ID=""
WASM_CI_HEAD_SHA=""
PUBLIC_INSTITUTION_ROOT=""
RPC_PORT=19944

validate_genesis_state_package() {
    local package_root="$1" path relative
    [[ -f "$package_root/manifest.json" ]] || { echo "错误:创世状态包缺少 manifest.json:$package_root" >&2; return 1; }
    [[ -d "$package_root/chains/citizenchain/db" ]] || { echo "错误:创世状态包缺少链数据库:$package_root" >&2; return 1; }

    # 正式包不得携带临时节点生成的 TLS、network、keystore 或日志目录；只允许清单和链数据库。
    while IFS= read -r -d '' path; do
        relative="${path#"$package_root"/}"
        if [[ -L "$path" ]]; then
            echo "错误:创世状态包禁止符号链接:$relative" >&2
            return 1
        fi
        case "$relative" in
            manifest.json|chains|chains/citizenchain|chains/citizenchain/db|chains/citizenchain/db/*) ;;
            *)
                echo "错误:创世状态包包含白名单外残留:$relative" >&2
                return 1
                ;;
        esac
    done < <(find "$package_root" -mindepth 1 -print0)

    python3 - "$package_root/manifest.json" <<'PYEOF'
import json
import sys

manifest_path = sys.argv[1]
with open(manifest_path, encoding="utf-8") as f:
    manifest = json.load(f)
required = (
    "package_format", "chain_id", "genesis_hash", "state_root", "chainspec_hash",
    "runtime_wasm_hash", "runtime_wasm_ci_run_id", "runtime_wasm_ci_head_sha",
    "light_sync_state_hash", "public_institution_root",
)
missing = [key for key in required if not manifest.get(key)]
if missing:
    raise SystemExit(f"创世状态包 manifest 缺少字段:{','.join(missing)}")
if manifest["package_format"] != "citizenchain-genesis-state-v1":
    raise SystemExit("创世状态包 manifest.package_format 无效")
if manifest["chain_id"] != "citizenchain":
    raise SystemExit("创世状态包 manifest.chain_id 无效")
if manifest.get("included_paths") != ["chains/citizenchain/db"]:
    raise SystemExit("创世状态包 manifest.included_paths 必须精确等于 chains/citizenchain/db")
if not str(manifest["runtime_wasm_ci_run_id"]).isdigit():
    raise SystemExit("创世状态包 manifest.runtime_wasm_ci_run_id 无效")
head_sha = manifest["runtime_wasm_ci_head_sha"]
if not isinstance(head_sha, str) or len(head_sha) != 40 or any(c not in "0123456789abcdef" for c in head_sha):
    raise SystemExit("创世状态包 manifest.runtime_wasm_ci_head_sha 无效")
PYEOF
}

usage() {
    cat <<'EOF'
Usage:
  citizenchain/scripts/bake-chainspec.sh [--out FILE] [--skip-check]
  citizenchain/scripts/bake-chainspec.sh --finalize --wasm FILE --wasm-ci-run-id ID --wasm-ci-head-sha SHA [--out FILE]

Options:
  --out FILE       生成 plain chainspec 的输出路径。默认 citizenchain/target/chainspec/citizenchain.plain.json
  --genesis-state-out DIR
                   生成已物化创世链状态包的输出目录。默认 citizenchain/target/chainspec/genesis-state
  --wasm FILE      GitHub WASM CI 产出的 runtime wasm。正式创世必须提供
  --wasm-ci-run-id ID
                   该 WASM artifact 所属 GitHub Actions run id
  --wasm-ci-head-sha SHA
                   该 WASM artifact 所属提交 SHA
  --public-institution-root HASH
                   CitizenApp 公权机构快照包根哈希,写入创世链状态包 manifest
  --finalize       覆盖冻结 SSOT: node/chainspecs/citizenchain.plain.json 与 citizenapp/assets/chainspec.json
  --skip-check     跳过宪法创世检查。只用于排障,正式创世不得使用
  -h, --help       显示帮助
EOF
}

while (($#)); do
    case "$1" in
        --out)
            OUT="${2:?--out 需要文件路径}"
            shift 2
            ;;
        --genesis-state-out)
            GENESIS_STATE_OUT="${2:?--genesis-state-out 需要目录路径}"
            shift 2
            ;;
        --wasm)
            WASM_FILE_ARG="${2:?--wasm 需要 wasm 文件路径}"
            shift 2
            ;;
        --wasm-ci-run-id)
            WASM_CI_RUN_ID="${2:?--wasm-ci-run-id 需要 run id}"
            shift 2
            ;;
        --wasm-ci-head-sha)
            WASM_CI_HEAD_SHA="${2:?--wasm-ci-head-sha 需要提交 SHA}"
            shift 2
            ;;
        --public-institution-root)
            PUBLIC_INSTITUTION_ROOT="${2:?--public-institution-root 需要 HASH}"
            shift 2
            ;;
        --finalize)
            FINALIZE=1
            shift
            ;;
        --skip-check)
            SKIP_CHECK=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "未知参数: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ "$FINALIZE" == "1" && -z "$WASM_FILE_ARG" ]]; then
    echo "错误: --finalize 必须同时提供 --wasm FILE,确保 :code 来自已通过 CI 的 WASM。" >&2
    exit 2
fi
if [[ "$FINALIZE" == "1" && ( -z "$WASM_CI_RUN_ID" || -z "$WASM_CI_HEAD_SHA" ) ]]; then
    echo "错误: --finalize 必须提供 --wasm-ci-run-id 与 --wasm-ci-head-sha,记录 CI artifact 来源。" >&2
    exit 2
fi
if [[ -n "$WASM_CI_RUN_ID" && ! "$WASM_CI_RUN_ID" =~ ^[0-9]+$ ]]; then
    echo "错误: --wasm-ci-run-id 必须为纯数字。" >&2
    exit 2
fi
if [[ -n "$WASM_CI_HEAD_SHA" && ! "$WASM_CI_HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]; then
    echo "错误: --wasm-ci-head-sha 必须为 40 位小写十六进制提交 SHA。" >&2
    exit 2
fi

if [[ -n "$WASM_FILE_ARG" ]]; then
    if [[ ! -s "$WASM_FILE_ARG" ]]; then
        echo "错误: WASM 文件不存在或为空: $WASM_FILE_ARG" >&2
        exit 2
    fi
    WASM_FILE="$(cd "$(dirname "$WASM_FILE_ARG")" && pwd)/$(basename "$WASM_FILE_ARG")"
    export WASM_FILE
    unset WASM_BUILD_FROM_SOURCE
    echo "==> 使用指定 WASM_FILE: $WASM_FILE"
else
    export WASM_BUILD_FROM_SOURCE=1
    unset WASM_FILE
    echo "==> 未指定 --wasm,仅做本地预览:从源码构建 runtime WASM"
fi

mkdir -p "$(dirname "$OUT")"
TMP="$(mktemp "$CHAIN_ROOT/target/chainspec/.citizenchain.plain.XXXXXX.json")"
NODE_TMP_DIR="$(mktemp -d "$CHAIN_ROOT/target/chainspec/.bakenode.XXXXXX")"
NODE_PID=""
cleanup() {
    [[ -n "$NODE_PID" ]] && kill "$NODE_PID" 2>/dev/null || true
    rm -f "$TMP"
    rm -rf "$NODE_TMP_DIR"
}
trap cleanup EXIT

echo "==> 导出 fresh plain chainspec..."
(
    cd "$CHAIN_ROOT"
    cargo run -p node -- export-chain-spec --chain citizenchain-fresh > "$TMP"
)

rpc() {
    # RPC 轮询必须有限时；解析错误交给调用点决定是否继续等待或立即失败。
    curl -fsS --max-time 10 -H 'content-type: application/json' \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}" \
        "http://127.0.0.1:$RPC_PORT" | python3 -c '
import json
import sys

data = json.load(sys.stdin)
error = data.get("error")
if error:
    raise SystemExit(f"RPC error: {error}")
if data.get("result") is None:
    raise SystemExit("RPC result is null")
print(json.dumps(data["result"], ensure_ascii=False))
'
}

echo "==> 启动临时节点物化创世(国家/省/市公权机构,记录耗时)..."
GENESIS_T0=$(date +%s)
(
    cd "$CHAIN_ROOT"
    CITIZENCHAIN_HEADLESS=1 ./target/debug/citizenchain --chain "$TMP" \
        --base-path "$NODE_TMP_DIR" --rpc-port "$RPC_PORT" \
        --no-mdns --no-prometheus --no-telemetry \
        >"$NODE_TMP_DIR/node.log" 2>&1
) &
NODE_PID=$!

GENESIS_HASH="null"
for _ in $(seq 1 120); do
    sleep 5
    if ! kill -0 "$NODE_PID" 2>/dev/null; then
        echo "错误: 临时节点提前退出,日志尾部:" >&2
        tail -20 "$NODE_TMP_DIR/node.log" >&2
        exit 1
    fi
    GENESIS_HASH=$(rpc chain_getBlockHash '[0]' 2>/dev/null || echo null)
    [[ "$GENESIS_HASH" != "null" && -n "$GENESIS_HASH" ]] && break
done
if [[ "$GENESIS_HASH" == "null" || -z "$GENESIS_HASH" ]]; then
    echo "错误: 10 分钟内未完成创世物化,日志尾部:" >&2
    tail -20 "$NODE_TMP_DIR/node.log" >&2
    exit 1
fi
GENESIS_SECS=$(( $(date +%s) - GENESIS_T0 ))
GENESIS_HASH_STR=$(echo "$GENESIS_HASH" | tr -d '"')
STATE_ROOT=$(rpc chain_getHeader "[$GENESIS_HASH]" | python3 -c 'import sys,json;print(json.loads(sys.stdin.read())["stateRoot"])')
echo "==> 创世物化完成: 耗时 ${GENESIS_SECS}s, genesis=$GENESIS_HASH_STR, stateRoot=$STATE_ROOT"

if [[ "$SKIP_CHECK" != "1" ]]; then
    echo "==> 检查宪法创世与冻结条件(RPC 模式)..."
    CHECK_ARGS=("$SCRIPT_DIR/check-constitution-genesis.py" --rpc "http://127.0.0.1:$RPC_PORT" --at "$GENESIS_HASH_STR")
    if [[ -n "$WASM_FILE_ARG" ]]; then
        CHECK_ARGS+=(--expect-code-file "$WASM_FILE")
    fi
    python3 "${CHECK_ARGS[@]}"
else
    echo "==> 已跳过宪法创世检查(--skip-check)"
fi

echo "==> 生成 CitizenApp 轻形态 chainspec(stateRootHash)..."
python3 - "$TMP" "$APP_OUT" "$STATE_ROOT" <<'PYEOF'
import json, sys
plain_path, app_path, state_root = sys.argv[1], sys.argv[2], sys.argv[3]
plain = json.load(open(plain_path))
# 轻形态:去掉 runtimeGenesis(完整 state 不进 App),只留 stateRootHash;
# smoldot 据此自建创世头,校验后续区块。
app = {k: plain[k] for k in
       ("name", "id", "chainType", "bootNodes", "telemetryEndpoints",
        "protocolId", "properties", "codeSubstitutes") if k in plain}
app["genesis"] = {"stateRootHash": state_root}
json.dump(app, open(app_path, "w"), ensure_ascii=False, indent=2)
print(f"    {app_path}")
PYEOF

echo "==> 生成 CitizenApp lightSyncState checkpoint..."
LIGHT_SYNC_STATE_JSON="$(rpc sync_state_genLightSyncState '[]')"
python3 - "$APP_LIGHT_SYNC_STATE_OUT" "$LIGHT_SYNC_STATE_JSON" <<'PYEOF'
import json
import re
import sys

out_path, raw = sys.argv[1], sys.argv[2]
lss = json.loads(raw)
required = ("finalizedBlockHeader", "grandpaAuthoritySet")
missing = [key for key in required if key not in lss]
if missing:
    raise SystemExit(f"lightSyncState 缺少字段:{','.join(missing)}")
for key in required:
    value = lss[key]
    if not isinstance(value, str) or not re.fullmatch(r"0x[0-9a-fA-F]+", value):
        raise SystemExit(f"lightSyncState.{key} 必须为 0x 十六进制字符串")
with open(out_path, "w", encoding="utf-8") as f:
    json.dump(lss, f, ensure_ascii=False, indent=2)
    f.write("\n")
print(f"    {out_path}")
PYEOF

kill "$NODE_PID" 2>/dev/null || true
wait "$NODE_PID" 2>/dev/null || true
NODE_PID=""

echo "==> 生成创世链状态包(供节点安装包首启直接复制链数据库)..."
rm -rf "$GENESIS_STATE_OUT"
mkdir -p "$GENESIS_STATE_OUT/chains/citizenchain"
if [[ ! -d "$NODE_TMP_DIR/chains/citizenchain/db" ]]; then
    echo "错误: 临时节点未生成 chains/citizenchain/db,无法制作创世链状态包。" >&2
    find "$NODE_TMP_DIR" -maxdepth 4 -type d | sort >&2
    exit 1
fi
cp -a "$NODE_TMP_DIR/chains/citizenchain/db" "$GENESIS_STATE_OUT/chains/citizenchain/db"
python3 - "$GENESIS_STATE_OUT/manifest.json" "$GENESIS_HASH_STR" "$STATE_ROOT" "$TMP" "${WASM_FILE:-}" "$APP_LIGHT_SYNC_STATE_OUT" "$PUBLIC_INSTITUTION_ROOT" "$GENESIS_SECS" "$WASM_CI_RUN_ID" "$WASM_CI_HEAD_SHA" <<'PYEOF'
import datetime
import hashlib
import json
import os
import sys

manifest_path, genesis_hash, state_root, chainspec_path, wasm_path, light_sync_state_path, public_institution_root, secs, wasm_ci_run_id, wasm_ci_head_sha = sys.argv[1:]

def sha256_file(path):
    if not path or not os.path.isfile(path):
        return ""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

manifest = {
    "package_format": "citizenchain-genesis-state-v1",
    "chain_id": "citizenchain",
    "snapshot_block_number": 0,
    "snapshot_block_hash": genesis_hash,
    "genesis_hash": genesis_hash,
    "state_root": state_root,
    "chainspec_hash": sha256_file(chainspec_path),
    "runtime_wasm_hash": sha256_file(wasm_path),
    "runtime_wasm_ci_run_id": wasm_ci_run_id,
    "runtime_wasm_ci_head_sha": wasm_ci_head_sha,
    "light_sync_state_hash": sha256_file(light_sync_state_path),
    "public_institution_root": public_institution_root,
    "genesis_materialization_secs": int(secs),
    "included_paths": ["chains/citizenchain/db"],
    "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
with open(manifest_path, "w", encoding="utf-8") as f:
    json.dump(manifest, f, ensure_ascii=False, indent=2)
    f.write("\n")
print(f"    {manifest_path}")
PYEOF

validate_genesis_state_package "$GENESIS_STATE_OUT"
echo "==> 创世链状态包白名单校验通过:仅包含 manifest.json 与 chains/citizenchain/db"

mv "$TMP" "$OUT"
trap - EXIT
rm -rf "$NODE_TMP_DIR"
echo "==> 已生成: $OUT"
echo "==> 首启物化耗时 ${GENESIS_SECS}s(验收记录);创世哈希 $GENESIS_HASH_STR"

if [[ "$FINALIZE" == "1" ]]; then
    NODE_SPEC="$CHAIN_ROOT/node/chainspecs/citizenchain.plain.json"
    APP_SPEC="$REPO_ROOT/citizenapp/assets/chainspec.json"
    APP_LIGHT_SYNC_STATE="$REPO_ROOT/citizenapp/assets/light_sync_state.json"
    install -m 0644 "$OUT" "$NODE_SPEC"
    install -m 0644 "$APP_OUT" "$APP_SPEC"
    install -m 0644 "$APP_LIGHT_SYNC_STATE_OUT" "$APP_LIGHT_SYNC_STATE"
    echo "==> 已同步冻结 SSOT:"
    echo "    $NODE_SPEC"
    echo "    $APP_SPEC (轻形态 stateRootHash)"
    echo "    $APP_LIGHT_SYNC_STATE (lightSyncState checkpoint)"
    echo "==> 创世链状态包已生成,打包安装包前需作为资源放入 genesis-state/:"
    echo "    $GENESIS_STATE_OUT"
else
    echo "==> 预览模式完成,未覆盖冻结 SSOT。正式创世请加 --finalize --wasm <CI_WASM>。"
fi
