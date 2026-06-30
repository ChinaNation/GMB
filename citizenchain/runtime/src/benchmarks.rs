// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

frame_benchmarking::define_benchmarks!(
    [frame_benchmarking, BaselineBench::<Runtime>]
    [frame_system, SystemBench::<Runtime>]
    [frame_system_extensions, SystemExtensionsBench::<Runtime>]
    [pallet_balances, Balances]
    [pallet_timestamp, Timestamp]
    [provincialbank_interest, ProvincialBankInterest]
    [fullnode_issuance, FullnodeIssuance]
    [citizen_issuance, CitizenIssuance]
    [resolution_issuance, ResolutionIssuance]
    [cid_system, CidSystem]
    [pow_difficulty, PowDifficulty]
    [public_admins, PublicAdmins]
    [private_admins, PrivateAdmins]
    [resolution_destro, ResolutionDestro]
    [grandpakey_change, GrandpaKeyChange]
    [public_manage, PublicManage]
    [private_manage, PrivateManage]
    // personal_manage / personal_admins benchmark 用例待 follow-up;当前 benchmarks.rs 是空骨架,
    // 不挂载到 list_benchmarks 避免 Benchmarking trait 缺失编译错误。
    // [personal_admins, PersonalAdmins]
    [multisig_transfer, MultisigTransfer]
    // internal-vote / joint-vote 删除 migration benchmark 后暂无 benchmark fn,
    // cast / finalize 权重待补;votingengine 引擎核心 + election-vote 同样暂无。
    // 无 benchmark fn 的 pallet 不挂载,避免 Benchmarking trait 缺失编译错误。
    // [internal_vote, InternalVote]
    // [joint_vote, JointVote]
    // [votingengine, VotingEngine]
    // [election_vote, ElectionVote]
    [runtime_upgrade, RuntimeUpgrade]
);
