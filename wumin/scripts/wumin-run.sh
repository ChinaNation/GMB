#!/usr/bin/env bash
# 清空缓存 + 生成代码 + 重新编译 + 启动 Wumin 签名设备
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WUMIN_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$SCRIPT_DIR/../.."
TARGET_DIR="$WUMIN_DIR/target"
TARGET_APK="$TARGET_DIR/公民钱包.apk"
cd "$WUMIN_DIR"

echo "==> 同步 runtime pallet/call 索引..."
RUNTIME_LIB="$REPO_ROOT/citizenchain/runtime/src/lib.rs"
REGISTRY="lib/signer/pallet_registry.dart"

# 从 runtime pallet_index 宏提取 pallet 索引
extract_pallet_index() {
  grep -A1 "pallet_index($1)" "$RUNTIME_LIB" | grep "pub type $2" > /dev/null && echo "$1"
}
BALANCES_IDX=$(grep -B1 'pub type Balances' "$RUNTIME_LIB" | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')
DUOQIAN_IDX=$(grep -B1 'pub type DuoqianTransfer' "$RUNTIME_LIB" | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')
VOTING_IDX=$(grep -B1 'pub type VotingEngine' "$RUNTIME_LIB" | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')

sed -i '' "s/balancesPallet = [0-9]*/balancesPallet = $BALANCES_IDX/" "$REGISTRY"
sed -i '' "s/duoqianTransferPallet = [0-9]*/duoqianTransferPallet = $DUOQIAN_IDX/" "$REGISTRY"
sed -i '' "s/votingEnginePallet = [0-9]*/votingEnginePallet = $VOTING_IDX/" "$REGISTRY"

# 从各 pallet crate 提取 call_index
TRANSFER_PALLET="$REPO_ROOT/citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs"
JOINT_VOTE_PALLET="$REPO_ROOT/citizenchain/runtime/votingengine/joint-vote/src/lib.rs"

PROPOSE_CALL=$(grep -B2 'fn propose_transfer' "$TRANSFER_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
# 联合投票内部投票阶段:JointVote::cast_admin
JOINT_CALL=$(grep -B2 'fn cast_admin' "$JOINT_VOTE_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
# 联合公投阶段:JointVote::cast_referendum
REFERENDUM_CALL=$(grep -B2 'fn cast_referendum' "$JOINT_VOTE_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')

sed -i '' "s/proposeTransferCall = [0-9]*/proposeTransferCall = $PROPOSE_CALL/" "$REGISTRY"
sed -i '' "s/jointVoteCall = [0-9]*/jointVoteCall = $JOINT_CALL/" "$REGISTRY"
sed -i '' "s/castReferendumCall = [0-9]*/castReferendumCall = $REFERENDUM_CALL/" "$REGISTRY"

echo "    Balances=$BALANCES_IDX DuoqianTransfer=$DUOQIAN_IDX VotingEngine=$VOTING_IDX"
echo "    propose=$PROPOSE_CALL joint=$JOINT_CALL referendum=$REFERENDUM_CALL"

echo "==> 清空构建缓存..."
flutter clean
echo "==> 获取依赖..."
flutter pub get
echo "==> 生成 Isar 代码..."
flutter pub run build_runner build --delete-conflicting-outputs

sync_android_artifact() {
  local source_apk="build/app/outputs/flutter-apk/app-debug.apk"
  if [[ -f "$source_apk" ]]; then
    mkdir -p "$TARGET_DIR"
    cp "$source_apk" "$TARGET_APK"
    echo "==> Android 产物已保存: $TARGET_APK"
  fi
}

# 中文注释：启动脚本固定把本地 APK 产物沉淀到项目根 target/，方便冷钱包设备离线安装和回滚。
echo "==> 生成 Android 产物..."
flutter build apk --debug
sync_android_artifact

echo "==> 编译并启动 App..."
flutter run
sync_android_artifact
