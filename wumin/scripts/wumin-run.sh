#!/usr/bin/env bash
# 清空缓存 + 生成代码 + 重新编译 + 启动 Wumin 签名设备
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WUMIN_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$SCRIPT_DIR/../.."
cd "$WUMIN_DIR"

echo "==> 同步 runtime spec_version 和 pallet 索引..."
RUNTIME_LIB="$REPO_ROOT/citizenchain/runtime/src/lib.rs"
REGISTRY="lib/signer/pallet_registry.dart"

# 从链上读取当前 spec_version（必须能连上节点）
CHAIN_RPC="${CHAIN_RPC_URL:-http://localhost:9944}"
SPEC=$(curl -sS --max-time 5 -H 'Content-Type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"state_getRuntimeVersion","params":[]}' \
  "$CHAIN_RPC" 2>/dev/null \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['result']['specVersion'])" 2>/dev/null) || true
if [[ -z "$SPEC" ]]; then
  echo "错误：无法从 $CHAIN_RPC 读取链上 spec_version。请确保区块链节点正在运行。"
  exit 1
fi
sed -i '' "s/supportedSpecVersions = {[^}]*}/supportedSpecVersions = {$SPEC}/" "$REGISTRY"

# 从 runtime pallet_index 宏提取 pallet 索引
extract_pallet_index() {
  grep -A1 "pallet_index($1)" "$RUNTIME_LIB" | grep "pub type $2" > /dev/null && echo "$1"
}
BALANCES_IDX=$(grep -B1 'pub type Balances' "$RUNTIME_LIB" | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')
DUOQIAN_IDX=$(grep -B1 'pub type DuoqianTransferPow' "$RUNTIME_LIB" | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')
VOTING_IDX=$(grep -B1 'pub type VotingEngineSystem' "$RUNTIME_LIB" | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')

sed -i '' "s/balancesPallet = [0-9]*/balancesPallet = $BALANCES_IDX/" "$REGISTRY"
sed -i '' "s/duoqianTransferPowPallet = [0-9]*/duoqianTransferPowPallet = $DUOQIAN_IDX/" "$REGISTRY"
sed -i '' "s/votingEngineSystemPallet = [0-9]*/votingEngineSystemPallet = $VOTING_IDX/" "$REGISTRY"

# 从各 pallet crate 提取 call_index
TRANSFER_PALLET="$REPO_ROOT/citizenchain/runtime/transaction/duoqian-transfer-pow/src/lib.rs"
VOTING_PALLET="$REPO_ROOT/citizenchain/runtime/governance/voting-engine-system/src/lib.rs"

PROPOSE_CALL=$(grep -B2 'fn propose_transfer' "$TRANSFER_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
VOTE_CALL=$(grep -B2 'fn vote_transfer' "$TRANSFER_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
JOINT_CALL=$(grep -B2 'fn joint_vote' "$VOTING_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
CITIZEN_CALL=$(grep -B2 'fn citizen_vote' "$VOTING_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')

sed -i '' "s/proposeTransferCall = [0-9]*/proposeTransferCall = $PROPOSE_CALL/" "$REGISTRY"
sed -i '' "s/voteTransferCall = [0-9]*/voteTransferCall = $VOTE_CALL/" "$REGISTRY"
sed -i '' "s/jointVoteCall = [0-9]*/jointVoteCall = $JOINT_CALL/" "$REGISTRY"
sed -i '' "s/citizenVoteCall = [0-9]*/citizenVoteCall = $CITIZEN_CALL/" "$REGISTRY"

echo "    spec_version={$SPEC} (链上) Balances=$BALANCES_IDX DuoqianTransfer=$DUOQIAN_IDX VotingEngine=$VOTING_IDX"
echo "    propose=$PROPOSE_CALL vote=$VOTE_CALL joint=$JOINT_CALL citizen=$CITIZEN_CALL"

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 生成 Isar 代码..."
flutter pub run build_runner build --delete-conflicting-outputs
echo "==> 编译并启动 App..."
flutter run
