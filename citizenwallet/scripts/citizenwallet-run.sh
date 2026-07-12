#!/usr/bin/env bash
# 清空缓存 + 生成代码 + 重新编译 + 启动 CitizenWallet 签名设备
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CITIZENWALLET_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$SCRIPT_DIR/../.."
TARGET_DIR="$CITIZENWALLET_DIR/target"
TARGET_APK="$TARGET_DIR/公民钱包.apk"
cd "$CITIZENWALLET_DIR"

echo "==> 同步 runtime pallet/call 索引..."
RUNTIME_LIB="$REPO_ROOT/citizenchain/runtime/src/lib.rs"
REGISTRY="lib/signer/pallet_registry.dart"

# 全量按 pallet【名字】从 runtime construct_runtime! 抽取 pallet_index，逐一回写
# 对应 Dart 常量。覆盖 registry 里全部 pallet 常量，杜绝“只同步 3 个、其余手改”
# 造成的半同步漂移(改号事故正是因此)。名字→数字映射永远以 runtime 为唯一真源。
sync_pallet() {
  # $1 = runtime `pub type` 名称, $2 = Dart 常量名
  local idx
  idx=$(grep -B1 "pub type $1 =" "$RUNTIME_LIB" \
    | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')
  test -n "$idx" || { echo "未找到 $1 pallet_index"; exit 1; }
  sed -i '' "s/${2} = [0-9]*/${2} = $idx/" "$REGISTRY"
  echo "    $1 -> $2 = $idx"
}

# 顺序无所谓，逐个按名同步(与 construct_runtime! 一一对应)。
sync_pallet OnchainTransaction  onchainTransactionPallet
sync_pallet VotingEngine        votingEnginePallet
sync_pallet CitizenIdentity     citizenIdentityPallet
sync_pallet InternalVote        internalVotePallet
sync_pallet JointVote           jointVotePallet
sync_pallet MultisigTransfer    multisigTransferPallet
sync_pallet RuntimeUpgrade      runtimeUpgradePallet
sync_pallet ResolutionDestroy   resolutionDestroPallet
sync_pallet GrandpaKeyChange    grandpaKeyChangePallet
sync_pallet ResolutionIssuance  resolutionIssuancePallet
sync_pallet OnchainIssuance     onchainIssuancePallet
sync_pallet LegislationYuan     legislationYuanPallet
sync_pallet LegislationVote     legislationVotePallet
sync_pallet OffchainTransaction offchainTransactionPallet
sync_pallet PersonalManage      personalManagePallet
sync_pallet PersonalAdmins      personalAdminsPallet
sync_pallet PublicAdmins        publicAdminsPallet
sync_pallet PrivateAdmins       privateAdminsPallet
sync_pallet PublicManage        publicManagePallet
sync_pallet PrivateManage       privateManagePallet

# call_index 稳定(D2 保留语义分带),仅同步 runtime 里会漂移的 3 个业务 call。
TRANSFER_PALLET="$REPO_ROOT/citizenchain/runtime/transaction/multisig/src/lib.rs"
JOINT_VOTE_PALLET="$REPO_ROOT/citizenchain/runtime/votingengine/joint-vote/src/lib.rs"

PROPOSE_CALL=$(grep -B2 'fn propose_transfer' "$TRANSFER_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
# 联合投票内部投票阶段:JointVote::cast_admin
JOINT_CALL=$(grep -B2 'fn cast_admin' "$JOINT_VOTE_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
# 联合公投阶段:JointVote::cast_referendum
REFERENDUM_CALL=$(grep -B2 'fn cast_referendum' "$JOINT_VOTE_PALLET" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')

test -n "$PROPOSE_CALL" || { echo "未找到 MultisigTransfer::propose_transfer call_index"; exit 1; }
test -n "$JOINT_CALL" || { echo "未找到 JointVote::cast_admin call_index"; exit 1; }
test -n "$REFERENDUM_CALL" || { echo "未找到 JointVote::cast_referendum call_index"; exit 1; }

sed -i '' "s/proposeTransferCall = [0-9]*/proposeTransferCall = $PROPOSE_CALL/" "$REGISTRY"
sed -i '' "s/jointVoteCall = [0-9]*/jointVoteCall = $JOINT_CALL/" "$REGISTRY"
sed -i '' "s/castReferendumCall = [0-9]*/castReferendumCall = $REFERENDUM_CALL/" "$REGISTRY"

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

# 启动脚本固定把本地 APK 产物沉淀到项目根 target/，方便冷钱包设备离线安装和回滚。
echo "==> 生成 Android 产物..."
flutter build apk --debug
sync_android_artifact

echo "==> 编译并启动 App..."
flutter run
sync_android_artifact
