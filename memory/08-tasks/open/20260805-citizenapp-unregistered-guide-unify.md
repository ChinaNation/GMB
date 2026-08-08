# 任务卡：CitizenApp 未注册身份统一引导(全 App 规范)

状态：实现完成(2026-08-05;全部 9 触点落地,analyze 零问题,受影响测试
25+34+296+204 全绿;真机验收待用户统一安排)

## 任务需求

「已创建/导入钱包但未注册 CID」是合法用户状态。当前各页面对它的呈现是散落的报错
(广场「广场内容加载失败」假故障、聊天 `WalletAuthException` 底层异常横幅、创作者页
错误态纯文本、若干 SnackBar),必须统一为**引导注册**,且 iOS/Android 两端一致
(Flutter 共享代码,天然一致)。

用户逐字要求:

- 只改中间的提示区,**别动其他 UI 界面**;
- 提示标题「尚未注册」;删掉「当前钱包还没有绑定 CID 号。」这句;按钮文案「注册」;
- 点「注册」**就地弹出**「我的 → 身份」的注册身份弹窗,不跳转身份页;
- 该注册身份弹窗全仓**统一只有一个**;
- 不止广场/聊天,创作者、通讯录等所有同逻辑页面一并统一。

## 三个单源

| 单源 | 位置 |
|---|---|
| 提示组件 `IdentityRegisterGuide` | `lib/ui/widgets/identity_register_guide.dart`(新建) |
| 注册弹窗 `showRegisterIdentitySheet` | `lib/my/myid/widgets/register_identity_sheet.dart`(已存在,内容零改动) |
| 注册流程 `startCidRegistrationFlow` | `lib/my/myid/register_identity_flow.dart`(新建,自 `myid_page.dart` 抽出:弹窗→余额闸→占号提交→身份缓存失效→成功提示) |

动作级助手 `ensureCidRegisteredOrPrompt(context)`:已注册放行;未注册就地弹统一
注册弹窗并中止动作(注册成功后用户重新触发动作,与身份页充值后不自动续跑同哲学)。
服务层 fail-closed 检查全部保留作兜底。

## 触点清单

页面级(主体区显示提示组件):

1. 广场 tab feed 区 —— 加载前身份缓存判定短路 + Worker `cid_not_bound` 映射;
   其它错误维持「广场内容加载失败」。
2. 聊天 tab 列表区 —— `_reload` 无 CID 短路(不读加密存储、不起轮询),
   异常横幅不再抢在提示前;现有 `_NoIdentity` 升级为统一组件。
3. 创作者页主体 —— `CreatorException` 未注册分支换提示组件,其它错误原样。

动作级(未注册 → 直接弹统一注册弹窗,替代现有 SnackBar/横幅):

4. 广场·发布(`square_home_page._openCompose`)
5. 聊天·私信/群聊/加好友(`chat_tab._requireChatIdentity`)
6. 通讯录·扫码加好友(`contact_book_page.scanAndAddContact`)
7. 我的·个人资料入口(`user.dart` 资料按钮)
8. 会员·订阅(`membership_page`,仅未注册分支)
9. 创作者·订阅按钮(`creator_subscribe_button`,仅未注册分支)

明确不动:无钱包提示、弹窗内部功能、服务层检查、Worker/链、其余一切 UI。

## 背景诊断(为何现在是报错)

- 广场:Worker feed 强制会话且用量按 CID 计量;未注册 → 登录挑战第一步
  `403 cid_not_bound`,客户端零处理,UI 折叠成「广场内容加载失败」。
- 聊天:渲染层三态本来就有提示(`_NoIdentity`),但 `_reload` 先读加密会话存储
  (`readConversationPreviews` 第一行解析密钥绑定),对未注册身份抛
  `WalletAuthException('当前 CID 钱包绑定尚未激活私有数据密钥')`,catch 成 `_error`
  横幅盖住提示。resolver 对无 CID 用户回退账户 0(非空),故读取器能拿到 accountId。
- 创作者:服务层 `isRegistered` 检查抛异常,页面错误态显示原文。

## 验收

- [x] 三个单源落地,身份页 `_onRegister` 改调共享流程,无逻辑复制
      (`register_identity_flow.dart` 含 `onSubmitting` 回调:转圈只跟提交阶段,
      面板打开期间不转 —— 提前置真会让 AppBar CircularProgressIndicator 永转,
      pumpAndSettle 超时即为此症)
- [x] 9 个触点全部统一;未注册零报错文案、零多余网络请求
      (广场 feed 与红点轮询 `_notifySession` 双短路 —— 红点是第二条会话路径,
      漏掉它未注册用户仍会周期性打 403)
- [x] 注册成功后:`IdentityAccountCache.instance.invalidate()` + 各页 onRegistered
      回刷（占号不改钱包列表，但 finalized 身份闭环成立后服务层统一失效缓存并递增
      walletsRevision；页面回调只作当前页即时兜底）
- [x] 测试:流程 4 例(放行/拦截/占号成功失效缓存/链读失败 fail-closed)+
      聊天短路铁证(readPreviewCount==0)+ 广场 3 例(缓存命中零请求/
      cid_not_bound 映射/通用错误保持原文案);受影响套件 25+34+296+204 全绿。
      两个既有测试文件(creator_plan/membership_page)补注已注册身份 fake
      (页面新增身份门,不注 fake 打真单例违反 hermetic)
- [x] 文档:CHAT_TECHNICAL §13 / PROFILE_TECHNICAL / USER_TECHNICAL §7;
      残留清零(`_NoIdentity` 删、旧文案 lib 内零命中、无未用 import)
