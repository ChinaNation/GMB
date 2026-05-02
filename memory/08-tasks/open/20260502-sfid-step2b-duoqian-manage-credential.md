# SFID Step 2b:duoqian-manage 凭证字段 + verifier 按 (province, admin_pubkey) 双层匹配

- 状态:open
- 创建日期:2026-05-02
- 模块:`citizenchain/runtime/transaction/duoqian-manage/`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`(Step 2 章节)
- 前置依赖:step2a(sfid-system pallet 重写完成,helper `sheng_signing_pubkey_for_admin` 可用)
- 阻塞下游:step2d(wumin / wuminapp decoder 必须按本卡新字段同步)

## 任务需求

`duoqian-manage::register_sfid_institution` 当前 verifier 走两条路径:
- `signing_province=None` → `current_sfid_verify_pubkey()`(SfidMainAccount,Step 2a 已删)
- `signing_province=Some(p)` → `sheng_signing_pubkey(p)`(单参,Step 2a 已删)

本卡:
- 凭证 payload **加字段** `signer_admin_pubkey: [u8;32]`
- verifier 总是走 `sheng_signing_pubkey_for_admin(signing_province, signer_admin_pubkey)` 双层匹配
- 删除 `signing_province=None` 分支(链上 0 prior knowledge of SFID,凭证必须显式带 admin_pubkey)

## 影响范围

### 凭证 payload(SCALE 编码顺序固定)

```rust
struct InstitutionRegistrationCredential {
    sfid_id: BoundedVec<u8, ConstU32<64>>,
    institution_name: BoundedVec<u8, ConstU32<128>>,
    a3: BoundedVec<u8, ConstU32<8>>,
    sub_type: Option<BoundedVec<u8, ConstU32<32>>>,
    parent_sfid_id: Option<BoundedVec<u8, ConstU32<64>>>,
    province: BoundedVec<u8, ConstU32<64>>,
    register_nonce: BoundedVec<u8, ConstU32<128>>,
    signer_admin_pubkey: [u8; 32],   // ★ 新字段
    signature: BoundedVec<u8, ConstU32<128>>,  // 64 字节 sr25519
}
```

新字段位置:`register_nonce` 后、`signature` 前(避免破坏 signature 之外字段顺序差异性)。

### verifier 改造

```rust
let pubkey = SfidSystem::<T>::sheng_signing_pubkey_for_admin(
    &credential.province,
    &credential.signer_admin_pubkey,
)
.ok_or(Error::<T>::SfidProvinceAdminSigningNotActivated)?;
sr25519_verify(&credential.signature, &payload_hash, &pubkey)?;
```

删除:
- `current_sfid_verify_pubkey()` 调用
- `signing_province=None` 分支
- `Optional<signing_province>` 改为必填(BoundedVec)

新 Error:`SfidProvinceAdminSigningNotActivated`

### 文件级

- `citizenchain/runtime/transaction/duoqian-manage/src/lib.rs`:凭证 struct + verifier + Error
- 该 pallet 现有测试:配套更新(测试构造凭证时加 `signer_admin_pubkey` 字段)
- 节点桌面 / SFID 后端凭证签发函数:对齐(本卡只动 runtime;前后端联动留 step2d)

## 主要风险点

- **SCALE 编码兼容性**:凭证 struct 加字段会破坏旧 wumin/wuminapp 扫码签名 decoder;step2d 必须**同步上线**(`memory/07-ai/chat-protocol.md` 第 5 条铁律)
- **签名 payload 哈希**:加 `signer_admin_pubkey` 后 payload bytes 改变;wumin / wuminapp / SFID 三处签发流程必须用同一序列化规则
- **`signing_province=None` 分支删除**:duoqian-manage 现有测试如果有依赖 None 分支的,必须重写

## 是否需要先沟通

- 否(ADR-008 已 accepted)

## 验收清单

- `cargo check -p duoqian-manage` + `cargo test -p duoqian-manage` + `cargo clippy` 全绿
- 现有 register_sfid_institution 测试(若 N 条)全部更新为新凭证字段
- 新增 ≥ 3 测试:
  - `register_with_main_admin_signature_succeeds`
  - `register_with_backup_admin_signature_succeeds`
  - `register_with_admin_not_in_roster_rejected`
  - `register_signing_not_activated_for_admin_rejected`
- 任务卡 progress 章节回写

## 不要做的事

- 不要碰 sfid-system(step2a 范围)
- 不要碰 genesis / spec_version(step2c 范围)
- 不要碰 wumin / wuminapp(step2d 范围)
- 不要 commit

## 工作量

~200 行 + 4 测试,~1 agent round。
