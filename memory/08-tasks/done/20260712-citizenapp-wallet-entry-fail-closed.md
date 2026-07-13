# 任务卡：门禁0 入口 fail-closed(创建/导入 = 注册强绑定,二元成败)

状态：已完成(2026-07-12)

任务需求：
进 app 身份门禁改二元 fail-closed——钱包创建 + Cloudflare 设备子钥注册必须都成功才进 app;
任一失败即回退,不允许"建了钱包却没注册"的中间态。现状注册 best-effort 吞错、钱包建成即放行,
进去后 session 因 device_not_registered 签发失败→聊天/广场全不可用。

所属模块：citizenapp / wallet

关键前提(已核实)：
- P-256 设备子钥=设备本地硬件密钥(Keystore/SE),不从助记词派生;square_device_subkeys 以
  owner_account 为主键 UPSERT→单账户单子钥,换设备导入必然重注册并顶掉旧设备。
- 用户数据(链上资产/身份 + Cloudflare 会员/帖子/资料/关注/额度)全按 owner_account 存,换设备
  不受影响;仅聊天记录端到端本机存、不跟随设备(隐私设计)。
- 门禁1(ED session assertOnchainWallet)/门禁2(会员 Stripe 四档 requireActiveMembership)/
  门禁3(用量 resourceLimits+usageLimits+square_browse_days+assertManifestQuota)均已建成生产,本次零改。

改动(纯客户端,worker 零改)：
- wallet_manager.dart：_tryRegisterDeviceSubkey→_registerDeviceSubkey 去掉吞错(失败上抛);
  createWallet(267)/importWallet(294) 把注册折进 try/catch,失败→_rollbackWalletCreation+rethrow,
  _bumpWalletsRevision 仅成功后触发。导入一律幂等注册(同设备已存在钱包先被 _checkDuplicatePubkey 拦)。
- create_wallet_onboarding_page.dart：_create catch 补"创建钱包失败"弹窗,停创建页可重试。
- wallet_page.dart：导入 _import catch 补"导入失败"弹窗;助记词仅成功路径 clear(失败保留)。
- 顺带:wallet_manager.dart:643 过时注释随注册方法重写清除。

验收结果：
- dart analyze 改动文件(含测试)：No issues found;dart format：0 changed。
- 新增测试(test/wallet/wallet_manager_test.dart 新组"门禁0 fail-closed")：createWallet 注册失败整笔
  回滚无残留、importWallet 注册失败整笔回滚无残留、createWallet 注册成功落库并主钥签绑定证明。
- flutter test test/wallet/ 全量 +97 全过,无回归。

未做(留档,非本次行为改动范围)：
- SquareSessionProvider "公开只读"残留注释未清(pre-existing 概念债,门禁0 落地后该 null 路径实际不可达)。
- ~~首启 onboarding 仍无导入入口~~ → **已补齐**(2026-07-12,见 20260712-citizenapp-onboarding-import-wallet):首启提供创建+导入两条二元 fail-closed 入口。
- 导入"已注册免注册"优化未做(改为一律幂等注册,二元最简)。
