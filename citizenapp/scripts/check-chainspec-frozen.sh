#!/usr/bin/env bash
# chainspec 单一权威源(SSOT)守卫
#
# 规则:citizenapp 轻节点 chainspec 的「创世部分」必须 == 链端 SSOT 的「创世部分」。
#   SSOT = citizenchain/node/chainspecs/citizenchain.raw.json(:code 永远是 CI WASM)。
#   citizenapp/assets/chainspec.json 是从 SSOT 派生的副本,二者创世必须逐字节等价,
#   否则轻节点 genesis hash 与全网对不上,smoldot 握手直接 ProtocolNotAvailable。
#
# bootNodes / lightSyncState 不参与 genesis hash,剔除后比对(允许网络层/checkpoint 差异)。
# 重新创世只跑 citizenchain/scripts/bake-chainspec.sh(会同步 SSOT 与 citizenapp 副本)。
# runtime 升级请走链上 system.setCode,绝不重新 build-spec。详见 memory/07-ai/chainspec-frozen.md
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CITIZENAPP="$REPO_ROOT/citizenapp/assets/chainspec.json"
SSOT="$REPO_ROOT/citizenchain/node/chainspecs/citizenchain.raw.json"

for f in "$CITIZENAPP" "$SSOT"; do
  if [[ ! -s "$f" ]]; then
    echo "[chainspec-ssot] 错误:$f 不存在或为空"
    exit 1
  fi
done

# 剔除 bootNodes / lightSyncState 后计算创世内容 sha256。
genesis_sha() { jq -cS 'del(.bootNodes, .lightSyncState)' "$1" | shasum -a 256 | awk '{print $1}'; }
CITIZENAPP_SHA="$(genesis_sha "$CITIZENAPP")"
SSOT_SHA="$(genesis_sha "$SSOT")"

if [[ "$CITIZENAPP_SHA" != "$SSOT_SHA" ]]; then
  cat >&2 <<EOF
╔══════════════════════════════════════════════════════════════════╗
║  拒绝:citizenapp chainspec 创世部分 ≠ 链端 SSOT!                  ║
╚══════════════════════════════════════════════════════════════════╝
  citizenapp 创世 sha256(不含 bootNodes/lightSyncState): $CITIZENAPP_SHA
  SSOT     创世 sha256(不含 bootNodes/lightSyncState): $SSOT_SHA

二者创世必须一致,否则轻节点 genesis hash 与全网对不上(ProtocolNotAvailable)。

修复:
  - 重新创世:跑 citizenchain/scripts/bake-chainspec.sh(自动同步 SSOT 与 citizenapp)
  - 仅同步副本:cp "$SSOT" "$CITIZENAPP"

runtime 升级请走链上 system.setCode,不要重新 build-spec。
详见 memory/07-ai/chainspec-frozen.md
EOF
  exit 1
fi
echo "[chainspec-ssot] citizenapp chainspec 创世部分 == 链端 SSOT ✅"
