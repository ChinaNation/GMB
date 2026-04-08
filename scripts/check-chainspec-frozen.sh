#!/usr/bin/env bash
# chainspec.json 冻结守卫：验证 wuminapp/assets/chainspec.json 的 sha256
# 与 wuminapp/assets/chainspec.json.sha256 中记录的期望值一致。
#
# chainspec.json 是链的创世文件，决定了 genesis hash，进而决定 libp2p
# 通知协议名。一旦创世冻结，任何修改都会导致所有轻节点和全节点握手
# 失败。runtime 升级请走链上 system.setCode 交易，不要动 chainspec。
#
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

EXPECTED="$(awk '{print $1}' "$SHAFILE")"
ACTUAL="$(shasum -a 256 "$CHAINSPEC" | awk '{print $1}')"

if [[ "$ACTUAL" != "$EXPECTED" ]]; then
  cat >&2 <<EOF
╔══════════════════════════════════════════════════════════════════╗
║  chainspec.json 被修改了！这是创世冻结文件，严禁任何变动。       ║
╚══════════════════════════════════════════════════════════════════╝
  期望 sha256: $EXPECTED
  实际 sha256: $ACTUAL

为什么 chainspec 不能改：
  chainspec 决定 genesis hash，genesis hash 决定 libp2p 通知协议名
  (/<genesis_hash>/block-announces/1)。一改就和线上节点握手失败。

runtime 升级请走链上 system.setCode 交易，不要重新 build-spec。

恢复方法：
  git checkout -- wuminapp/assets/chainspec.json

详见：memory/07-ai/chainspec-frozen.md
EOF
  exit 1
fi
echo "[chainspec-frozen] chainspec.json 完整性校验通过"
