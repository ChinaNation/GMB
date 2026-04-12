#!/usr/bin/env bash
# chainspec.json 创世冻结守卫
#
# 校验 wuminapp/assets/chainspec.json 中 **影响 genesis hash 的字段** 是否被篡改。
# bootNodes 仅用于节点发现，不参与 genesis hash 计算，因此允许修改（如域名变更）。
# 校验方式：用 jq 剔除 bootNodes 后计算 sha256，与 .sha256 文件比对。
#
# runtime 升级请走链上 system.setCode 交易，不要重新 build-spec。
# 详见 memory/07-ai/chainspec-frozen.md
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHAINSPEC="$REPO_ROOT/wuminapp/assets/chainspec.json"
SHAFILE="$REPO_ROOT/wuminapp/assets/chainspec.json.sha256"

if [[ ! -s "$CHAINSPEC" ]]; then
  echo "[chainspec-frozen] 错误：$CHAINSPEC 不存在或为空"
  exit 1
fi
if [[ ! -s "$SHAFILE" ]]; then
  echo "[chainspec-frozen] 错误：$SHAFILE 不存在。请先运行 scripts/lock-chainspec.sh 记录初始哈希。"
  exit 1
fi

# bootNodes 不参与 genesis hash，剔除后再校验，允许域名等网络层变更。
EXPECTED="$(awk '{print $1}' "$SHAFILE")"
ACTUAL="$(jq -cS 'del(.bootNodes)' "$CHAINSPEC" | shasum -a 256 | awk '{print $1}')"

if [[ "$ACTUAL" != "$EXPECTED" ]]; then
  cat >&2 <<EOF
╔══════════════════════════════════════════════════════════════════╗
║  拒绝提交： wuminapp/assets/chainspec.json 是创世冻结文件！     ║
╚══════════════════════════════════════════════════════════════════╝
  期望 sha256（不含 bootNodes）: $EXPECTED
  实际 sha256（不含 bootNodes）: $ACTUAL

chainspec 决定 genesis hash， genesis hash 决定 libp2p 通知协议名。
一旦修改， 所有已部署的节点都会和新轻节点握手失败（ProtocolNotAvailable）。

runtime 升级请走链上 system.setCode 交易， 不要重新 build-spec。
详见： memory/07-ai/chainspec-frozen.md

如果你是在做硬分叉（ 极少数情况）， 请手动：
  git commit --no-verify
并同步更新 wuminapp/assets/chainspec.json.sha256。
EOF
  exit 1
fi
echo "[chainspec-frozen] chainspec.json 创世内容校验通过（bootNodes 变更不受限）"
