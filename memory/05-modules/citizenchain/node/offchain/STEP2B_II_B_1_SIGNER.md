# citizenchain/node/src/offchain/keystore_signer.rs · Step 2b-ii-β-1 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付 Step 2b-ii-β-1(真实批次签名器:接 offchain_keystore)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_II_A_PACKER.md`(`BatchSigner` trait + Noop 占位)
- **后续**:`STEP2B_II_B_2_INTEGRATION.md`(`PoolBatchSubmitter` + service.rs + cli.rs + rpc.rs 接入)

---

## 1. 本步范围

Step 2b-ii-β 再拆成 β-1 / β-2,本次交付 **β-1 · 真实 signer 实现**:
- 新建 `offchain/keystore_signer.rs`:`KeystoreBatchSigner` 实现 `packer::BatchSigner`,从 `offchain_keystore::SigningKey` 派生 sr25519 签名
- `offchain/mod.rs` 挂载新子模块并 `pub use` 供 β-2 service.rs 使用
- 5 个单测覆盖验签通过 / 未加载 Err / 不同消息 / 错误密钥不验过 / 热切换

**明确不做**(β-2):
- `PoolBatchSubmitter`(拼 RuntimeCall + UncheckedExtrinsic + TransactionPool.submit_one)
- `cli.rs` 加 `--clearing-bank` flag
- `service.rs` 启动清算行 worker
- `rpc.rs` 合并 `OffchainClearingRpcServer::into_rpc`

---

## 2. 新增代码

### 2.1 `KeystoreBatchSigner`

```rust
pub struct KeystoreBatchSigner {
    signing_key: Arc<RwLock<Option<SigningKey>>>,
}

impl KeystoreBatchSigner {
    pub fn new(signing_key: Arc<RwLock<Option<SigningKey>>>) -> Self { ... }
}

impl BatchSigner for KeystoreBatchSigner {
    fn sign_batch(&self, message: &[u8]) -> Result<[u8; 64], String> {
        let guard = self.signing_key.read()...;
        let key = guard.as_ref().ok_or("未加载")?;
        let signature = <sr25519::Pair as Pair>::sign(&key.pair, message);
        Ok(signature.0)
    }
}
```

### 2.2 依赖关系

- 依赖 `crate::offchain_keystore::SigningKey`(**旧** Step 1 保留模块,Step 2b-iv 清理时再评估是否迁移;本步**复用**以最小化改动)
- 依赖 `super::packer::BatchSigner`(Step 2b-ii-α 定义)
- **不依赖** substrate client / TransactionPool / runtime,完全本地可测

### 2.3 `mod.rs` 变化

```
 pub mod event_listener;
+pub mod keystore_signer;
 pub mod ledger;
 pub mod packer;
 pub mod rpc;
...
+#[allow(unused_imports)] // β-2 接入后去掉
+pub use self::keystore_signer::KeystoreBatchSigner;
```

## 3. 热切换语义

`Arc<RwLock<Option<SigningKey>>>` 设计允许节点运行中替换密钥:

- 启动时 `None`,待用户输入密码解锁后 `*slot.write() = Some(key)`
- 运行时如果需要换密钥(例如密钥轮换),直接 `*slot.write() = Some(new_key)`,所有持有 `signer: Arc<dyn BatchSigner>` 的消费者立即用新密钥

单测 `hot_swap_key_takes_effect` 验证:对同一消息,替换密钥后新签名由新公钥验签通过,旧公钥不通过。

## 4. 单元测试

| 测试 | 覆盖 |
|---|---|
| `sign_produces_verifiable_signature` | 用 `sr25519::Pair::verify` 对返回签名 + 公钥验签 |
| `sign_without_key_loaded_errs` | `slot = None` → Err 且消息含"未加载" |
| `different_messages_produce_different_signatures` | 两条不同消息签名必不同(sr25519 本身带随机,一定不同) |
| `signature_does_not_verify_against_wrong_key` | A 私钥签的消息不能被 B 公钥验过 |
| `hot_swap_key_takes_effect` | `*slot.write() = Some(new_key)` 后新签名只被新公钥验过 |

## 5. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node
  offchain/ 子树零错误零警告
  节点链接阶段在 ui/mod.rs:91 tauri proc macro 受 frontend/dist 门禁(项目固有)
```

## 6. Step 2b-ii-β-2 对接清单

下一步(本 signer 将被 β-2 这样使用):

```rust
// service.rs(β-2 新增)
let keystore = OffchainKeystore::new(&base_path);
let signing_key_slot: Arc<RwLock<Option<SigningKey>>> = Arc::new(RwLock::new(None));
if keystore.has_signing_key() {
    let password = cli_args.clearing_bank_password.as_deref().unwrap_or("");
    let key = keystore.load_signing_key(password)?;
    *signing_key_slot.write().unwrap() = Some(key);
}

let signer: Arc<dyn BatchSigner> =
    Arc::new(KeystoreBatchSigner::new(signing_key_slot));
let submitter: Arc<dyn BatchSubmitter> =
    Arc::new(PoolBatchSubmitter::new(client.clone(), pool.clone()));  // β-2 实现

let components = offchain::start_clearing_bank_components(
    &base_path,
    bank_main_from_cli,
    password,
    signer,
    submitter,
)?;

// 注册 RPC + 后台 packer worker
```

## 7. 变更记录

- 2026-04-19:Step 2b-ii-β-1 `KeystoreBatchSigner` + 5 个单测,offchain/ 子树零编译错误零警告。
