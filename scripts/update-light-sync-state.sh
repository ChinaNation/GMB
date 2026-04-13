#!/usr/bin/env bash
# 从全节点获取 lightSyncState checkpoint，写入 wuminapp/assets/light_sync_state.json。
#
# smoldot 轻节点拿到 checkpoint 后直接从 finalized block 开始同步，
# 跳过 genesis 到 finalized 之间的全部区块头验证，冷启动从分钟级降到秒级。
#
# 用法：
#   ./scripts/update-light-sync-state.sh                  # 从本地 127.0.0.1:9944
#   ./scripts/update-light-sync-state.sh http://host:9944 # 从指定全节点
#
# 安全措施：脚本会校验来源节点的 genesis hash 与冻结的 chainspec 一致，
# 防止连错环境把别的链的 checkpoint 打进包里。
#
# light_sync_state.json 不参与 genesis hash，也不在 chainspec 冻结校验范围内。
# 每次发版前运行一次即可，checkpoint 落后不影响正确性，只影响追赶长度。
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT="$REPO_ROOT/wuminapp/assets/light_sync_state.json"
RPC_URL="${1:-http://127.0.0.1:9944}"

# citizenchain 主网 genesis hash（冻结值，与 chainspec.json 对应）
EXPECTED_GENESIS="0x9341a792cde9e1b298b740c15e08409501701a7162faf2accb804156278942af"

echo "[light-sync-state] 从 $RPC_URL 获取 checkpoint..."

# 1. 校验来源节点的 genesis hash
GENESIS="$(curl -sf --connect-timeout 10 -X POST "$RPC_URL/" \
  -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getBlockHash","params":[0]}')" \
  || { echo "错误：无法连接 $RPC_URL"; exit 1; }

ACTUAL_GENESIS="$(echo "$GENESIS" | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])")"

if [[ "$ACTUAL_GENESIS" != "$EXPECTED_GENESIS" ]]; then
  echo "错误：来源节点 genesis hash 不匹配！"
  echo "  期望（citizenchain 主网）: $EXPECTED_GENESIS"
  echo "  实际（$RPC_URL）:          $ACTUAL_GENESIS"
  echo "  请确认连接的是 citizenchain 主网全节点。"
  exit 1
fi
echo "[light-sync-state] genesis hash 校验通过"

# 2. 获取 lightSyncState
RESPONSE="$(curl -sf --connect-timeout 10 -X POST "$RPC_URL/" \
  -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"sync_state_genSyncSpec","params":[]}')" \
  || { echo "错误：sync_state_genSyncSpec 调用失败"; exit 1; }

# 3. 提取并写入
python3 -c "
import sys, json
d = json.loads(sys.stdin.read())
if 'result' not in d or 'lightSyncState' not in d['result']:
    print('错误：响应中无 lightSyncState', file=sys.stderr)
    sys.exit(1)
lss = d['result']['lightSyncState']
print(json.dumps(lss))
" <<< "$RESPONSE" > "$OUTPUT"

SIZE="$(wc -c < "$OUTPUT" | tr -d ' ')"
echo "[light-sync-state] 已写入 $OUTPUT ($SIZE bytes)"
