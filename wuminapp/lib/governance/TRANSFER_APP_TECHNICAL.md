# 机构转账提案 App 侧技术方案

## 1. 功能概述

管理员在机构详情页点击"转账"提案类型后，进入转账表单页面，填写收款地址、金额、备注，提交后构造 `propose_transfer` extrinsic 签名上链。其他管理员可查看提案详情并投票（`vote_transfer`）。

## 2. 页面流程

```
机构详情页 → 提案类型页(转账) → 转账表单页 → 签名 → 提交
                                              ↓
                               投票事件列表 → 提案详情页 → 投票 → 签名 → 提交
```

## 3. 新增页面

### 3.1 转账表单页 `transfer_proposal_page.dart`

**入口**：从 `proposal_types_page.dart` 的"转账"卡片点击进入。

**页面元素**：

| 元素 | 类型 | 说明 |
|---|---|---|
| 机构名称 | 文本（只读） | 自动填充，不可编辑 |
| 机构类型 | 标签（只读） | NRC/PRC/PRB |
| 转出地址 | 文本（只读） | 机构 duoqian_address 的 SS58 地址 |
| 收款地址 | 输入框 | SS58 格式，支持扫码输入 |
| 转账金额 | 输入框 | 单位：元，最低 1.11 元（111分 = ED） |
| 预估手续费 | 文本（实时计算） | `max(金额 × 0.1%, 0.10元)` |
| 备注 | 输入框 | 可选，最长 256 字节 |
| 可用余额 | 文本（链上查询） | 机构 duoqian_address 的 free_balance |
| 提交按钮 | 按钮 | 校验通过后签名提交 |

**本地校验**：

1. 收款地址必须是合法 SS58 地址（format 2027）
2. 收款地址不能是本机构的 duoqian_address（自转账）
3. 金额 >= 1.11 元（111 分，ED）
4. 金额 + 手续费 + ED <= 可用余额
5. 备注 UTF-8 编码后 <= 256 字节

**提交流程**：

1. 构造 `propose_transfer` call data（见第 5 节）
2. 获取链上动态参数（nonce、runtime version、latest block）
3. 构造 SigningPayload
4. 调用签名（热钱包直接签名 / 冷钱包 QR 签名）
5. 构造 ExtrinsicPayload，编码并提交
6. 显示提交结果（成功/失败）

### 3.2 提案详情页 `transfer_proposal_detail_page.dart`

**入口**：从机构详情页的投票事件列表点击进入。

**页面元素**：

| 元素 | 说明 |
|---|---|
| 提案状态 | 投票中 / 已通过 / 已拒绝 / 执行失败 |
| 提案信息 | 收款地址、金额、备注、发起管理员、创建时间 |
| 投票进度 | 赞成 X / 阈值 Y，进度条 |
| 管理员投票明细 | 每位管理员：地址 + 赞成/反对/未投票 |
| 投票按钮 | 仅管理员且未投票时显示，赞成/反对两个按钮 |

**数据来源（链上查询）**：

| 数据 | Storage 路径 | 说明 |
|---|---|---|
| 提案动作 | `DuoqianTransferPow::ProposalActions(proposal_id)` | 机构、收款地址、金额、备注、发起人 |
| 提案状态 | `VotingEngineSystem::Proposals(proposal_id)` | status(0=投票中/1=通过/2=拒绝)、start、end |
| 投票计数 | `VotingEngineSystem::InternalTallies(proposal_id)` | yes_count、no_count |
| 投票记录 | `VotingEngineSystem::InternalVotesByAccount(proposal_id, admin_pubkey)` | bool(赞成/反对) |
| 活跃提案ID | `DuoqianTransferPow::ActiveProposalByInstitution(institution_id)` | u64 proposal_id |

## 4. 链上 Extrinsic 编码

### 4.1 propose_transfer

**pallet_index**: 19（DuoqianTransferPow）
**call_index**: 0

**SCALE 编码格式**：

```
[0x13]                              // pallet_index = 19
[0x00]                              // call_index = 0
[u8]                                // org: 0=NRC, 1=PRC, 2=PRB
[48 bytes]                          // institution: shenfen_id 右补零到 48 字节
[0x00 + 32 bytes]                   // beneficiary: MultiAddress::Id + AccountId32
[Compact<u128>]                     // amount: 金额（分）
[Compact<u32> + bytes]              // remark: SCALE Vec<u8> (Compact 长度 + 原始字节)
```

### 4.2 vote_transfer

**pallet_index**: 19
**call_index**: 1

**SCALE 编码格式**：

```
[0x13]                              // pallet_index = 19
[0x01]                              // call_index = 1
[8 bytes little-endian]             // proposal_id: u64
[0x01 或 0x00]                      // approve: true(0x01) / false(0x00)
```

## 5. 核心服务：TransferProposalService

新建 `lib/governance/transfer_proposal_service.dart`，封装链上交互：

```dart
class TransferProposalService {
  /// 查询机构 duoqian_address 的可用余额（元）
  Future<double> fetchInstitutionBalance(String shenfenId);

  /// 查询机构活跃的转账提案 ID（无活跃提案返回 null）
  Future<int?> fetchActiveTransferProposal(String shenfenId);

  /// 查询提案详情（TransferAction）
  Future<TransferProposalInfo?> fetchProposalInfo(int proposalId);

  /// 查询投票状态（赞成数、反对数）
  Future<({int yes, int no})> fetchVoteTally(int proposalId);

  /// 查询某管理员对某提案的投票记录（null=未投票）
  Future<bool?> fetchAdminVote(int proposalId, String pubkeyHex);

  /// 构造并提交 propose_transfer extrinsic
  Future<String> submitProposeTransfer({
    required InstitutionInfo institution,
    required String beneficiaryAddress,
    required double amountYuan,
    required String remark,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  });

  /// 构造并提交 vote_transfer extrinsic
  Future<String> submitVoteTransfer({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  });
}
```

## 6. Storage Key 构造

### 6.1 DuoqianTransferPow 存储

| 存储项 | Key 格式 |
|---|---|
| `ProposalActions` | `twox128("DuoqianTransferPow") + twox128("ProposalActions") + blake2_128_concat(u64_le)` |
| `ActiveProposalByInstitution` | `twox128("DuoqianTransferPow") + twox128("ActiveProposalByInstitution") + blake2_128_concat(institution_48bytes)` |

### 6.2 VotingEngineSystem 存储

| 存储项 | Key 格式 |
|---|---|
| `Proposals` | `twox128("VotingEngineSystem") + twox128("Proposals") + blake2_128_concat(u64_le)` |
| `InternalTallies` | `twox128("VotingEngineSystem") + twox128("InternalTallies") + blake2_128_concat(u64_le)` |
| `InternalVotesByAccount` | `twox128("VotingEngineSystem") + twox128("InternalVotesByAccount") + blake2_128_concat(u64_le) + blake2_128_concat(account_32bytes)` |

注：`blake2_128_concat` = `blake2_128(data) + data`。

## 7. duoqian_address 查询

机构的 duoqian_address 预置在 `primitives` 中，App 侧需要：

1. 在 `institution_data.dart` 中为每个 `InstitutionInfo` 增加 `duoqianAddress` 字段（32 字节公钥 hex）
2. 从 `primitives/china/china_cb.rs` 和 `primitives/china/china_ch.rs` 中提取 87 个机构的 `duoqian_address`
3. 通过 `Keyring().encodeAddress(bytes, 2027)` 转换为 SS58 地址展示

## 8. 手续费显示

复用 `OnchainRpc.estimateTransferFeeYuan(amountYuan)`，实时计算并显示：

- 费率：0.1%
- 最低：0.10 元
- 支付方：机构 duoqian_address（非管理员个人）

## 9. 签名适配

复用现有签名基础设施：

| 钱包类型 | 签名方式 |
|---|---|
| 热钱包（local） | `LocalSigner.sign(payload)` → 64 字节 sr25519 签名 |
| 冷钱包（external） | `QrSignSessionPage` → QR 码展示 payload → 扫码获取签名 |

签名流程与 `OnchainRpc.transferKeepAlive` 完全一致，仅 call data 不同。

## 10. 文件清单

| 文件 | 说明 | 状态 |
|---|---|---|
| `lib/governance/transfer_proposal_page.dart` | 转账表单页 | 新建 |
| `lib/governance/transfer_proposal_detail_page.dart` | 提案详情 + 投票页 | 新建 |
| `lib/governance/transfer_proposal_service.dart` | 链上交互服务 | 新建 |
| `lib/governance/institution_data.dart` | 增加 duoqianAddress 字段 | 修改 |
| `lib/governance/proposal_types_page.dart` | "转账"按钮接入真实页面 | 修改 |
| `lib/governance/institution_detail_page.dart` | 投票事件列表接入真实数据 | 修改 |

## 11. 实施顺序

1. `institution_data.dart` — 增加 87 个 duoqianAddress
2. `transfer_proposal_service.dart` — 链上交互（extrinsic 编码 + storage 查询）
3. `transfer_proposal_page.dart` — 转账表单页 UI + 提交
4. `proposal_types_page.dart` — 接入转账页面
5. `transfer_proposal_detail_page.dart` — 提案详情 + 投票
6. `institution_detail_page.dart` — 投票事件列表接入
