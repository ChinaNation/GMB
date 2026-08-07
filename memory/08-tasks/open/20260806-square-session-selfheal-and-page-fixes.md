# 任务卡：广场会话自愈(invalid_signature)+ 四页呈现修复

状态：代码完成(2026-08-06;analyze 零问题,受影响测试全绿,双端已重建装机);
真机复测被手机 smoldot 同步超时阻塞,真因排查中。
**更正**:链高停在 #41 是**空块不出块的正常空闲态**(死规则,勿再误判"链停摆/
矿工未开");昨晚同一高度手机能秒同步,今早超时 → 真因在手机侧 peers/网络,与链高无关。

## 背景(真机验收 Pixel 撞出的一串问题)

用户报 5 症状(有 CID、曾订阅平台会员):我的-背景图停顿、创作者恒"同步中"、
通讯录云端恒"离线"、会员页无订阅无价格、广场发布提示无订阅。

诊断结论:

- **主根因**:生产 Worker 登录完成阶段 401 `invalid_signature`。重新创世 + model B
  重派生后钱包重建 → walletIndex 换新 → 硬件 P-256 子钥换新;D1 里该身份的设备行
  **存在但钥不对**。而懒登记只在 `device_not_registered`(库无行)触发 →
  **死锁:挑战能发、完成必败、登记永不触发**。会话挂 → 创作者/通讯录云端/会员页/
  发布全部连坐(各页第一步都是 `ensureSession`)。
- 该死锁**不是开发期特例**:正式用户换新手机/重装 App 同样命中(同身份多设备本是
  Worker 设计,`device_id = sha256(P-256 公钥)` 每设备一行)。
- **副根因**:链上订阅记录随重新创世归零 → 需重新订阅。平台三档价格为创世
  `genesis_build` 内置,**无需任何登记动作**(旧「无 genesis 配价」口径已过时,
  记忆索引已修正)。
- 背景图停顿独立:进主页前同步 `await MyIdService.getState()` 整闭环链读。

## 修复(全部共享 Dart,iOS/Android 天然一致;铁律已入 agent-rules.md)

1. **会话自愈**(核心,[square_api_client.dart] `_establishSessionWithRetry`):
   可自愈 401 从 `{device_not_registered}` 扩为 `{device_not_registered,
   invalid_signature}`,均触发一次**本机**子钥登记并重试一次。安全性由注册端点
   兜底:链上 finalized 绑定 + 钱包主钥 sr25519 绑定签名 + Turnstile,无种子者
   伪造不出;多设备并存不覆盖别机。**Worker 零改动,D1 一行不删**(旧行成无害
   幽灵行,登录按行遍历验签自然跳过)。
2. **背景图入口**([user.dart] `_openMyProfile`):改 `IdentityAccountCache.resolve()`
   (缓存命中零等待,未命中才落一次链读;链读失败 fail-closed 提示,不冒充未注册)。
3. **创作者页**:首载失败(`_data==null && _error!=null`)显示明确错误态
   (复用 `_inlineError` 含重试),不再落"同步中"骨架无限装加载。
4. **会员页**:会话失败且无可展示数据 → 顶部 `_LoadFailureBanner`(警示横幅+重试),
   三张静态卡按该页设计保留,不整页替换;有数据时维持原 SnackBar 语义。

## 测试

- `test/8964/square_session_selfheal_test.dart`(新):invalid_signature 自愈成功 /
  device_not_registered 不回归 / 其它错误码零登记原样上抛 / 自愈只重试一次不循环。
- 创作者:首载失败 → 错误态非"同步中",重试恢复。
- 会员:会话失败 → 失败横幅 + 三卡保留(key: membership-load-failure-banner)。
- 受影响目录 8964/my/user 全绿。

## 链侧事实(更正版)

- **空块不出块是设计**:交易池空则不产块,链高静止 = 正常空闲;矿工低 CPU 同理
  (有交易才挖)。验证链活性的正确方式 = 发交易看出块,不是看高度。
- 手机 smoldot 同步超时与链高无关(昨晚同高度秒同步),真因在手机侧
  peers/网络,单独排查。
- D1 幽灵设备行(旧钥):无害,留档;后续可补按 `updated_at` 的陈旧行修剪,
  不在本卡范围。

## 复测清单(手机同步恢复后)

- [ ] Pixel:进广场 → 触发自愈(生物识别一次)→ feed 加载;通讯录云端转"已同步";
      创作者页正常(未订阅则显示开通引导);会员页显示三档价格(创世价)。
- [ ] 重新订阅平台会员 → 广场发布放行;创作者页进入已开通态。
- [ ] iPhone 同项复测(同一份代码)。
