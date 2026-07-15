# CitizenApp 用户资料默认展示统一

## 任务需求

- 用户昵称、头像、背景使用同一套公开资料展示规则。
- 真实资料存在时展示真实资料；缺失或图片加载失败时，根据钱包账户稳定选择本地内置默认资料。
- 昵称不得回退为完整或截断的钱包账户；钱包账户只允许出现在明确的账户行。
- 通讯录、广场、聊天、关注列表和唯一用户主页必须对同一账户显示一致资料。
- 使用用户指定的本机 Downloads 图片作为默认头像和背景资源，确认精确源文件与目标路径后移入 CitizenApp。

## 实现边界

- 新增唯一前端资料展示解析器，稳定派生默认昵称、头像和背景。
- 默认资料只用于本地展示，不上传 Cloudflare，不成为身份或授权真源。
- 联系人的私人名称继续只属于通讯录，不得进入公开用户主页。
- 不修改 Cloudflare 数据契约，不修改 `citizenchain/runtime/`。
- 不保留账户充当昵称的旧回退、旧注释或重复展示规则。

## 预计修改目录

- `citizenapp/lib/8964/profile/`：新增统一展示解析器，收敛主页、头像、背景和标题展示代码。
- `citizenapp/lib/8964/models/`、`citizenapp/lib/8964/widgets/`：收敛广场作者展示规则。
- `citizenapp/lib/my/user/`：收敛通讯录公开昵称、头像和主页入口。
- `citizenapp/lib/chat/`：收敛聊天用户标题和头像展示。
- `citizenapp/assets/`：在精确文件清单确认后接收 Downloads 中的默认头像和背景资源。
- `citizenapp/test/`：补充稳定性、跨入口一致性及禁止账户昵称测试。
- `memory/05-modules/citizenapp/`：更新用户主页、通讯录和聊天技术文档，清理旧口径。

## 验收标准

- 同一账户跨页面、重启和设备得到相同默认昵称、头像、背景。
- 公开昵称缺失时使用本地默认昵称，任何昵称位置不显示钱包账户。
- 真实昵称、头像、背景优先；图片加载失败稳定回落本地资源。
- 自动测试、静态检查通过，并在连接的 Android 真机完成多入口真实页面验收。
- 文档、中文注释与残留同步清理。

## 当前状态

- **状态：已完成（2026-07-15）**。
- 已新增唯一展示解析器 `ProfilePresentation`：按钱包账户稳定派生默认昵称、头像和背景；真实钱包名/公开昵称优先，完整或截断账户不得进入昵称位置。
- 已把用户确认的 11 张 Downloads 照片移入 `citizenapp/assets/profile_defaults/`，统一供默认头像和背景使用；头像与背景采用不同稳定盐值并避免同图。
- 已收敛唯一用户主页、资料编辑、广场作者、关注列表、通讯录、聊天页和“我的”页的昵称、头像及背景展示规则；删除旧 SVG 默认头像及重复占位实现。
- 已更新用户资料、通讯录和聊天技术文档，补充中文边界注释，并清理账户充当昵称、昵称首字头像和旧资源路径等残留口径。

## 验收记录

- `flutter test test/8964/profile test/user/contact_book_page_test.dart test/chat/chat_tab_test.dart`：64 项通过。
- 目标文件静态检查：通过，无问题。
- `git diff --check`：通过。
- Android 真机 `SM A156U`（Android 16）验收：同一账户正常显示稳定默认昵称“碧海筑梦者”、本地照片头像和本地照片背景，钱包账户仅显示在独立账户行；Logcat 未发现资源加载失败、Flutter Widget 异常或 ErrorWidget。
- 临时真机验收入口已删除，`citizenapp/lib/main.dart` 恢复零差异；正常版本重新构建、安装并回到原始钱包入口。
- 全包 `flutter analyze` 仅报告两项本任务范围外的既有 info：`runtime_upgrade_detail_page.dart` 缺花括号、`onchain_payment_service.dart` 可使用 const；本任务修改范围无新增告警。

## 边界确认

- 未修改 `citizenchain/runtime/`。
- 未修改 Cloudflare 数据契约或执行部署。
- 未执行 Git 推送、远端 workflow 或 PR 操作。
