# citizenchain/crates 技术说明

`citizenchain/crates/` 放不属于 runtime、node、onchina 三大件的独立工具 crate。

| crate | 用途 |
|---|---|
| `blockchain-harness` | 链行为夹具与篡改用例（导出块、构造异常 state root 供守卫测试） |
| `citizen-signer` | 公民钱包原生签名库真源（sr25519 派生与签名，四端共用） |
| `qr-protocol` | QR 协议真源与金标夹具（`golden_fixtures.rs`、`repo_guard.rs`） |

## 格式与静态检查

三个 crate 与 runtime/node/onchina 同属一个 workspace，CI 的「公民链全工程验证」对
**整个 workspace** 执行 `cargo fmt --all -- --check` 与 `cargo clippy`，任一文件格式偏差
都会让该 job 直接失败。

- 提交前必须 `cargo fmt --all`，不能只格式化本次改动的文件：
  局部格式化会把其它文件的既有偏差留到下一次提交，届时 CI 报的是「与本次改动无关的文件」，
  排查方向容易被带偏。
- `cargo clippy --workspace --all-targets` 需零 error 后再推。

## 金标夹具

`qr-protocol/tests/golden_fixtures.rs` 与 `repo_guard.rs` 是 QR 协议的跨端锁步夹具；
改动 QR 协议字段序时，它们与四端（onchina、citizenapp、citizenwallet、node）必须同改，
另有 `.github/scripts/check-golden-vectors-sync.mjs` 在 CI 侧校验真源与各端镜像一致。
