#!/usr/bin/env bash
# CitizenApp 轻节点 chainspec 守卫。
#
# 新创世形态:
#   - 链端 SSOT = citizenchain/node/chainspecs/citizenchain.plain.json
#     (runtime WASM + genesis patch + bootnodes,不含 GB 级 raw state)
#   - CitizenApp = assets/chainspec.json 轻形态,genesis 只允许携带 stateRootHash
#   - stateRootHash 来自 bake-chainspec.sh 临时节点物化块 0 后读取的 state_root
#
# 代码 CI 阶段允许 App 资产尚未 finalize,但正式发包前必须设置
# CITIZENAPP_REQUIRE_STATE_ROOT=1,强制拒绝旧 raw 资产。
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CITIZENAPP="$REPO_ROOT/citizenapp/assets/chainspec.json"
SSOT="$REPO_ROOT/citizenchain/node/chainspecs/citizenchain.plain.json"
GENESIS_MANIFEST="${CITIZENCHAIN_GENESIS_STATE_MANIFEST:-$REPO_ROOT/citizenchain/target/chainspec/genesis-state/manifest.json}"
REQUIRE_STATE_ROOT="${CITIZENAPP_REQUIRE_STATE_ROOT:-0}"

python3 - "$CITIZENAPP" "$SSOT" "$GENESIS_MANIFEST" "$REQUIRE_STATE_ROOT" <<'PY'
import json
import os
import re
import sys

app_path, ssot_path, manifest_path, require_state_root = sys.argv[1:]
errors = []
warnings = []

def load_json(path, label):
    if not os.path.isfile(path) or os.path.getsize(path) == 0:
        errors.append(f"{label} 不存在或为空:{path}")
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception as exc:
        errors.append(f"{label} JSON 解析失败:{exc}")
        return {}

app = load_json(app_path, "CitizenApp chainspec")
ssot = load_json(ssot_path, "节点 plain SSOT")

if ssot:
    if ssot.get("id") != "citizenchain":
        errors.append("节点 plain SSOT id 必须为 citizenchain")
    ssot_genesis = ssot.get("genesis") or {}
    if "runtimeGenesis" not in ssot_genesis:
        errors.append("节点 plain SSOT 必须是 runtimeGenesis plain 形态")
    if "raw" in ssot_genesis:
        errors.append("节点 plain SSOT 不得包含 raw genesis")

if app:
    if app.get("id") != "citizenchain":
        errors.append("CitizenApp chainspec id 必须为 citizenchain")
    if ssot and app.get("protocolId") != ssot.get("protocolId"):
        errors.append("CitizenApp protocolId 必须与节点 plain SSOT 一致")
    app_genesis = app.get("genesis") or {}
    state_root = app_genesis.get("stateRootHash")
    if state_root:
        if not re.fullmatch(r"0x[0-9a-fA-F]{64}", state_root):
            errors.append("CitizenApp genesis.stateRootHash 必须为 0x + 64 位十六进制")
        if "raw" in app_genesis or "runtimeGenesis" in app_genesis:
            errors.append("CitizenApp 轻形态不得同时携带 raw/runtimeGenesis")
        if os.path.isfile(manifest_path):
            manifest = load_json(manifest_path, "创世状态包 manifest")
            manifest_state_root = manifest.get("state_root")
            if manifest_state_root and manifest_state_root.lower() != state_root.lower():
                errors.append(
                    "CitizenApp stateRootHash 与创世状态包 manifest.state_root 不一致:"
                    f" app={state_root} manifest={manifest_state_root}"
                )
            if manifest.get("chain_id") and manifest.get("chain_id") != "citizenchain":
                errors.append("创世状态包 manifest.chain_id 必须为 citizenchain")
        else:
            warnings.append(f"未找到创世状态包 manifest,跳过 stateRootHash 交叉校验:{manifest_path}")
    elif "raw" in app_genesis:
        msg = (
            "CitizenApp chainspec 仍为旧 raw 形态。代码 CI 可暂时通过,"
            "但正式创世后必须由 bake-chainspec.sh 生成 stateRootHash 轻形态。"
        )
        if require_state_root == "1":
            errors.append(msg)
        else:
            warnings.append(msg)
    else:
        errors.append("CitizenApp chainspec 必须携带 genesis.stateRootHash")

if errors:
    print("[chainspec-ssot] 拒绝:", file=sys.stderr)
    for item in errors:
        print(f"  - {item}", file=sys.stderr)
    print("修复:运行 citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>。", file=sys.stderr)
    sys.exit(1)

for item in warnings:
    print(f"[chainspec-ssot][warn] {item}")
print("[chainspec-ssot] plain SSOT / CitizenApp chainspec 阶段校验通过")
PY
