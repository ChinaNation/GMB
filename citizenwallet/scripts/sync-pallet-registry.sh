#!/usr/bin/env bash
# 把 runtime 的 pallet_index / call_index 全量回写到冷钱包的 pallet_registry.dart。
#
# 名字→数字的映射永远以 citizenchain/runtime 为唯一真源。冷钱包离线签名，索引一旦
# 与链端脱节，签出来的交易会被链上按另一个 pallet 解码——这类事故没有任何编译期信号。
#
# 本脚本是该同步逻辑的唯一实现。此前存在两份副本（citizenwallet-run.sh 与
# citizenwallet-ci.yml），且覆盖范围不同：本地同步 20 个 pallet，CI 只同步 3 个。
# 加 iOS job 会变成三份，因此收敛到这里，三个调用点共用同一份、覆盖同一个全集。
#
# 用法：sync-pallet-registry.sh [仓库根目录]
#   省略参数时按脚本位置推断（citizenwallet/scripts/.. /..）。
set -euo pipefail

REPO_ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
RUNTIME_LIB="$REPO_ROOT/citizenchain/runtime/src/lib.rs"
REGISTRY="$REPO_ROOT/citizenwallet/lib/signer/pallet_registry.dart"
TRANSFER_PALLET="$REPO_ROOT/citizenchain/runtime/transaction/multisig/src/lib.rs"
JOINT_VOTE_PALLET="$REPO_ROOT/citizenchain/runtime/votingengine/joint-vote/src/lib.rs"

for required in "$RUNTIME_LIB" "$REGISTRY" "$TRANSFER_PALLET" "$JOINT_VOTE_PALLET"; do
  [[ -f "$required" ]] || { echo "缺少同步所需文件：$required" >&2; exit 1; }
done

# 全量按 pallet【名字】从 runtime construct_runtime! 抽取 pallet_index，逐一回写对应 Dart
# 常量。必须覆盖 registry 里全部 pallet 常量——「只同步 3 个、其余手改」造成的半同步漂移
# 正是改号事故的来源。
sync_pallet() {
  # $1 = runtime `pub type` 名称, $2 = Dart 常量名
  local idx
  idx=$(grep -B1 "pub type $1 =" "$RUNTIME_LIB" \
    | grep -o 'pallet_index([0-9]*)' | grep -o '[0-9]*')
  [[ -n "$idx" ]] || { echo "未找到 $1 pallet_index" >&2; exit 1; }
  sed -i '' -e "s/${2} = [0-9]*/${2} = $idx/" "$REGISTRY" 2>/dev/null \
    || sed -i -e "s/${2} = [0-9]*/${2} = $idx/" "$REGISTRY"
  echo "    $1 -> $2 = $idx"
}

# call_index 稳定(D2 保留语义分带),只同步 runtime 里会漂移的 3 个业务 call。
sync_call() {
  # $1 = pallet 源文件, $2 = fn 名, $3 = Dart 常量名
  local idx
  idx=$(grep -B2 "fn $2" "$1" | grep -o 'call_index([0-9]*)' | grep -o '[0-9]*')
  [[ -n "$idx" ]] || { echo "未找到 $2 call_index" >&2; exit 1; }
  sed -i '' -e "s/${3} = [0-9]*/${3} = $idx/" "$REGISTRY" 2>/dev/null \
    || sed -i -e "s/${3} = [0-9]*/${3} = $idx/" "$REGISTRY"
  echo "    $2 -> $3 = $idx"
}

echo "==> 同步 runtime pallet/call 索引..."

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

sync_call "$TRANSFER_PALLET"   propose_transfer  proposeTransferCall
# 联合投票内部投票阶段:JointVote::cast_admin
sync_call "$JOINT_VOTE_PALLET" cast_admin        jointVoteCall
# 联合公投阶段:JointVote::cast_referendum
sync_call "$JOINT_VOTE_PALLET" cast_referendum   castReferendumCall
