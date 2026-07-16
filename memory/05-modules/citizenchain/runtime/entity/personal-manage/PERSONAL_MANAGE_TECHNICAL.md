# personal-manage 技术说明

模块：`personal-manage`

职责：个人多签账户生命周期。只负责个人多签创建、关闭和投票终态回调中的业务
pending 处理。

## 边界

- 个人多签管理员真源归 `personal-admins`。
- 个人多签转账归 `multisig-transfer`。
- 个人多签机构码为 `PMUL`。
- 不承担公权机构、私权机构生命周期。
- 创建与关闭执行器只接受投票引擎 callback scope 内、owner/kind/stage/status 与本模块生命周期提案完全绑定的回调；关闭还必须匹配 `PendingCloseProposal[account]`，不能拿其它已通过内部提案复用。

## MODULE_TAG

- `b"per-mgmt"`

## 费用与 ED

- `propose_create` / `propose_close` 是个人操作，外层最低链上操作费由签名者支付；`InternalVote::cast` 才由实际投票签名者支付固定 1 元。
- 创建执行费按创建金额计算，由创建提案人支付；创建本金同样由提案人转入个人多签账户，两项均不使用机构费用账户。
- 关闭执行费按关闭前余额计算并从个人多签账户收取，剩余余额再以 `AllowDeath` 转给受益人；只有这条显式关闭路径允许个人多签账户死亡。
- 关闭不保存独立固定最低余额常量；提案时直接用统一链上费公式计算执行费，并要求扣费后的转出金额不低于链上 ED；执行时重新校验最新余额。
- 统一收费器必须完整扣款并保留 ED；收费失败不改扣其他管理员，创建/关闭业务状态保持不变。

## 钱包扫码

- pallet index：`7`
- 创建 call：`propose_create`
- 关闭 call：`propose_close`
- call index 2 永久留洞，不复用。否决、超时和执行失败后的 pending/预留款清理由
  votingengine 终态回调自动完成，不存在人工清理交易。
