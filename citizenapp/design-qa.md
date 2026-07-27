# 钱包详情复制图标最终下移与二维码去底验收

## 对照真源

- 最终确认 UI：
  `/Users/rhett/.codex/generated_images/019f9fee-7cc8-7913-afa3-a151d4e31ad5/call_6qvxVNllUaqnfJ1beUR7pB2N.png`
- Pixel 8a 实现截图：
  `/tmp/wallet-copy-final.XC7LCf/05-wallet-detail-final.png`
- 全屏同图对照：
  `/tmp/wallet-copy-final.XC7LCf/06-comparison.png`
- 地址区局部同图对照：
  `/tmp/wallet-copy-final.XC7LCf/07-focused-comparison.png`
- 复制交互证据：
  `/tmp/wallet-copy-final.XC7LCf/08-copy-feedback.png`
- 源稿为 842×1869，实现为 1080×2400；两者宽高比一致。对照时将源稿等比归一为
  1080×2400 后与实现并排，不比较不同系统时间和真实余额数据。
- 真实设备：Pixel 8a、Android 16、物理分辨率 1080×2400、约 411×914 Flutter
  逻辑像素；Web CSS 尺寸不适用。
- 状态：CitizenApp 真实本地数据、钱包详情页、链上余额 `1,000,000.00 元`。

## Findings

- 无 P0、P1、P2 视觉问题。
- 字体与排版：钱包名称、完整地址和余额继续使用现有 Flutter 字体与字号；第一行地址
  使用复制按钮左侧最大可用宽度，第二行只承接剩余字符并左对齐，没有省略号或第三行。
- 间距与布局：复制图标保持横向位置和 28×28 点击热区，仅可见原子向下平移 4
  逻辑像素；竖线、二维码和交易记录宽度没有被带动。
- 颜色与视觉 token：二维码入口表面为完全透明，主视觉继续使用
  `AppTheme.primary (#007A74)`；复制、竖线和二维码保持原白色层级。
- 图片与图标：页面没有新增图片资产；复制继续使用 Material
  `Icons.copy_outlined`，二维码继续使用 `Icons.qr_code_rounded`，没有重绘、替换、
  拉伸或改变原子形状。
- 文案与内容：钱包名称、完整地址、余额、交易状态、来源、时间和金额均来自真实运行
  数据，未改动业务文案。
- 复制图标继续使用 Material `Icons.copy_outlined` 原双矩形图形，尺寸 12、颜色与横向
  位置不变，仅在原 28×28 点击热区内向下平移 4 逻辑像素。
- 二维码按钮继续保留 48×48 点击热区和 `Icons.qr_code_rounded` 原子图标，白色背景
  已删除，按钮表面为完全透明。

## 真实交互与自动检查

- [x] 点击复制图标触发完整地址复制和 Android 剪贴板反馈。
- [x] 钱包身份组件 4 项测试全部通过。
- [x] 目标实现与测试文件 `flutter analyze` 通过，无问题。
- [x] Pixel 8a 真机安装、真实页面与地址复制交互复验通过。
- [x] Flutter 运行日志没有 RenderFlex、溢出或本次修改引起的异常。

## Comparison History

- 首轮真机截图仍把地址提前换行，形成第一行短、第二行长的 P2 排版问题。
- 修正后拆分点取第一行可容纳的最长前缀；复验截图中第一行排满，第二行只显示溢出
  地址，P2 已关闭。

## Follow-up Polish

- 无。

final result: passed

---

# 公民币标志交易页视觉验收

## 对照真源

- 设计真源：
  `/Users/rhett/.codex/generated_images/019f9fee-7cc8-7913-afa3-a151d4e31ad5/call_yp5DVjHRiTqSzcboTtxsCVWT.png`
- 实现截图：`/tmp/gmb-logo-qa.uPNO4V/transaction.png`
- 全屏同图对照：`/tmp/gmb-logo-qa.uPNO4V/full-comparison.png`
- 币种栏局部同图对照：`/tmp/gmb-logo-qa.uPNO4V/focused-comparison.png`
- 设计真源像素：1536×1024。
- 实现截图像素：1080×2400。
- 真实设备：Pixel 8a，Android 16，物理密度 420 dpi（约 2.625 dpr），对应约
  411.4×914.3 Flutter 逻辑像素；Web CSS 尺寸不适用。
- 状态：CitizenApp 真机启动后进入“交易”，轻节点完成同步并显示“公民链 已更新”；
  币种栏显示“公民币标志 + GMB”。
- 密度归一化：全屏对照将两个截图等比缩放到 900px 高并置于同一画布；局部对照分别
  裁取品牌标志和币种栏后等比缩放到 520px 高并置于同一画布。

## Findings

- 无 P0、P1、P2 视觉问题。
- 字体与排版：`GMB` 继续使用交易表单现有 14sp、`FontWeight.w500`，没有换行、截断
  或层级漂移；图标不替代币种文本。
- 间距与布局：标志为 18×18，和 `GMB` 间距为 6，二者在原 112 宽币种框内整体居中；
  金额栏、币种栏、输入边框和页面纵向节奏未改变。
- 颜色与视觉 token：标志使用 `AppTheme.primary (#007A74)` 的单一纯色，文本使用
  `AppTheme.textPrimary (#1A2B3C)`；确认稿中 ImageGen 的轻微渲染光感未进入执行态。
- 图片质量与资产一致性：透明 PNG 保留定稿外环和右侧等长双横线，真机 18px 下边缘
  清楚，没有可见底色、透明毛边、变形或裁切；没有使用文本字符、emoji、Material
  通用币种图标或代码绘图替代。
- 文案与语义：币种仍显示 `GMB`，没有恢复 `CNY` 或改写金额单位；标志由
  `ExcludeSemantics` 排除重复朗读，`GMB` 继续承担无障碍和业务语义。

## Open Questions

- 无。

## Implementation Checklist

- [x] 公民币透明定稿资产已加入并注册。
- [x] `GMB` 左侧已显示 18×18 主题色标志。
- [x] widget test 已覆盖资产路径、尺寸、颜色和混合模式。
- [x] 定向 `flutter test` 与 `flutter analyze` 已通过。
- [x] Pixel 8a 真机安装、交易页导航、链同步状态和视觉显示已验证。
- [x] Flutter 运行日志未出现与本次图标接入相关的异常或渲染错误。

## Follow-up Polish

- 无阻塞或 P3 跟进项。

## Comparison History

- 首轮对照未发现 P0、P1、P2 问题，因此没有修复后复验循环。

final result: passed

---

# 钱包地址复制图标与交易记录宽度最终执行复验

## 对照真源

- 最终确认 UI：
  `/tmp/wallet-ui-exact-final-v2.png`
- Pixel 8a 实现截图：
  `/tmp/wallet-final-implementation-v2.png`
- 全屏同图对照：
  `/tmp/wallet-final-reference-comparison.png`
- 地址与交易记录局部同图对照：
  `/tmp/wallet-final-focused-comparison.png`
- 源稿与实现均为 1080×2400；真实设备为 Pixel 8a、Android 16、约 2.625 dpr，
  对应约 411×914 Flutter 逻辑像素，无密度缩放差异。

## Findings

- 无 P0、P1、P2 视觉问题。
- 字体与排版：完整 SS58 地址稳定显示为两行，第二行左对齐；真机根据系统等宽字体
  实际字宽在更早字符处换行，但完整内容、两行结构和层级与确认稿一致，没有省略或
  第三行。
- 图标：复制按钮继续使用 Material `Icons.copy_outlined` 原双矩形图形，没有替换、
  重绘或改变图形语义；尺寸为 12，位于地址第一行右侧并与该行垂直居中。
- 身份区：最左侧 56×56 钱包装饰图标及其 16 间距已删除，钱包名称和完整地址直接
  与主视觉左侧内容起点对齐。
- 间距：保留复制按钮与竖线之间 20 的布局间距，仅将原复制图标整体向左校准 1.25；
  Pixel 8a 最终 1080×2400 截图按同一亮度阈值量取，两侧可见图形到竖线均为
  81 物理像素。
- 交易记录：页面水平外边距由 16 收紧为 4，Pixel 8a 截图中白色板块接近左右边缘，
  与最终稿一致；内部圆角、标题、分隔线、记录内容和入口未改变。
- 色彩与图片：继续使用现行主题 token 和 Material 图标，没有新增图片、替代资产或
  颜色漂移。
- 文案与内容：钱包名称、完整地址、余额、交易状态、来源、时间和金额均保持真实
  运行数据。

## 真实交互验收

- [x] 点击复制图标显示“钱包地址已复制”。
- [x] 点击地址区域仍可复制完整地址。
- [x] 二维码入口、名称编辑、余额和交易详情入口未改变。
- [x] Flutter 运行日志无 RenderFlex、溢出或图片资源错误。
- [x] 最左侧钱包装饰图标未显示，身份区固定高度未改变。

## 自动检查

- 钱包身份、余额、操作和交易记录相关 17 项 Flutter widget 测试通过。
- `flutter analyze` 目标实现文件通过。
- `git diff --check` 通过。

## Comparison History

- 首轮实现截图中完整地址自动换成三行，属于 P2 排版偏差。
- 修正后按可用宽度分别计算第一行和第二行：第一行预留复制按钮，第二行使用完整
  宽度；复验截图稳定为两行，P2 已关闭。

## Follow-up Polish

- 无。

final result: passed

---

# 钱包详情方案 2 最终修正复验（最终真源）

## 修正原因

- 首轮实现把身份区的触控高度直接叠加进可见布局，导致主视觉总高度约 264 逻辑像素，
  比方案 2 约 237 逻辑像素高约 27。
- 首轮遗漏了账户信息与二维码入口之间的竖分隔线。
- 首轮只在“会员｜订阅”的三张会员卡片加入公民币标志，遗漏“会员详情”价格。

## 对照与证据

- 方案 2 视觉真源：
  `/Users/rhett/.codex/generated_images/019f9fee-7cc8-7913-afa3-a151d4e31ad5/call_eeu7Ypx31EW7Z7WkBu1UzAtw.png`
- 修正前钱包详情：
  `/tmp/wallet-membership-fix-audit.6OV8qR/01-wallet-detail-before.png`
- 修正后钱包详情：
  `/tmp/wallet-membership-fix-audit.6OV8qR/03-wallet-detail-after.png`
- 方案 2 与修正后同屏对照：
  `/tmp/wallet-membership-fix-audit.6OV8qR/04-wallet-reference-after-comparison.png`
- 修正前后同屏对照：
  `/tmp/wallet-membership-fix-audit.6OV8qR/05-wallet-before-after.png`
- 修正后会员详情：
  `/tmp/wallet-membership-fix-audit.6OV8qR/06-membership-detail-after.png`
- 完整地址与复制按钮最终截图：
  `/tmp/wallet-address-after.png`
- 真实设备：Pixel 8a，Android 16，物理分辨率 1080×2400，约 2.625 dpr。

## 修正结果

- 钱包身份区固定为 112 逻辑像素，余额区固定为 123 逻辑像素；加上中间 1 像素分隔线，
  主视觉总高度为 236 逻辑像素，与方案 2 约 237 逻辑像素一致。
- 账户信息与二维码入口之间已恢复 1×48 的半透明白色竖分隔线。
- 会员详情价格左侧已显示 18×18 公民币定稿标志，间距为 6；自由金底使用主文字色，
  深色会员档位使用白色。
- 公民币标志不重复承担无障碍朗读，价格文本和原有业务逻辑保持不变。
- 钱包地址已删除 `前 8...后 6` 的界面缩写，完整 SS58 地址在空间不足时分为两行，
  第二行与第一行左对齐。
- 复制按钮位于竖分隔线左侧；按钮边界到分隔线、分隔线到二维码按钮边界均为 16
  逻辑像素，两个可见图标到分隔线的视觉间距一致。

## 复验清单

- [x] Pixel 8a 真机确认钱包主视觉高度与方案 2 一致。
- [x] Pixel 8a 真机确认二维码竖分隔线清晰可见。
- [x] Pixel 8a 真机确认会员详情价格左侧公民币标志实际显示。
- [x] 组件测试锁定身份区、余额区和竖分隔线尺寸。
- [x] 会员测试覆盖详情页标志资产、尺寸、颜色和混合模式。
- [x] 钱包与会员相关 38 项 Flutter widget 测试全部通过。
- [x] Pixel 8a 真机确认完整地址两行显示、复制按钮等距布局和复制提示。
- [x] Flutter 运行日志未出现 RenderFlex、溢出或图片资源加载错误。

final result: passed

---

# 钱包详情方案 2 首轮验收记录（已由上方最终复验取代）

## 对照真源

- 钱包详情方案 2：
  `/Users/rhett/.codex/generated_images/019f9fee-7cc8-7913-afa3-a151d4e31ad5/call_eeu7Ypx31EW7Z7WkBu1UzAtw.png`
- 钱包详情实现截图：`/tmp/wallet-membership-qa.QiujLv/wallet-detail.png`
- 钱包详情同屏对照：`/tmp/wallet-membership-qa.QiujLv/wallet-comparison.png`
- 会员自由档截图：`/tmp/wallet-membership-qa.QiujLv/membership.png`
- 会员民主档截图：`/tmp/wallet-membership-qa.QiujLv/membership-democracy.png`
- 会员薪火档截图：`/tmp/wallet-membership-qa.QiujLv/membership-spark.png`
- 真实设备：Pixel 8a，Android 16，物理分辨率 1080×2400，约 411×914 Flutter
  逻辑像素。
- 钱包详情对照将方案 2 和真机 App 内容区分别等比归一到 1800px 高后并排检查；
  系统状态栏和 Home Indicator 不属于 App 设计范围。

## Findings

- 无 P0、P1、P2 视觉问题。
- 页面层级：钱包身份和 finalized 链上余额已合并为全宽纯青绿色主视觉，三项操作紧随
  其后，交易记录使用单一白色分组卡片，与方案 2 一致。
- 色彩与表面：主视觉使用 `AppTheme.primary (#007A74)` 纯色，没有恢复渐变、玻璃
  效果或阴影堆叠；操作区和交易区继续使用现行白色、边框和分隔线 token。
- 身份区：钱包名称、编辑图标、短地址、复制图标和二维码入口均清晰；长名称与地址未
  溢出，点击目标满足移动端触控尺寸。
- 余额区：真机显示 `1,000,000.00 元`，单位只出现一次；大额通过自适应缩放在 Pixel
  8a 宽度内完整展示。
- 操作区：充值、提现、零钱包三列等宽，线性图标和竖分隔线清晰；零钱包“未绑定”
  状态没有被视觉改造覆盖。
- 交易区：标题入口、已确认状态、来源、时间、金额和详情箭头均保留，真实记录没有
  截断或重叠。
- 会员价格：自由、民主、薪火三张卡均在月价左侧显示 18×18 公民币定稿标志；自由金
  使用主文字色，民主蓝与薪火红使用纯白，三个彩色表面上均有清晰对比。
- 无障碍语义：会员价格文本继续承担金额朗读，公民币标志通过 `ExcludeSemantics`
  排除重复朗读。

## 真实交互验收

- [x] 钱包详情更多菜单仍显示扫一扫、清算行、查看私钥和查看助记词。
- [x] 点击短地址复制后显示“钱包地址已复制”。
- [x] 二维码入口进入现有用户二维码页。
- [x] 充值、提现和零钱包原有点击结构及未绑定门槛测试通过。
- [x] 最近交易进入真实交易详情页并显示 finalized“已确认”状态。
- [x] 下拉刷新后链上余额和最近交易仍正常展示。
- [x] 会员卡左右滑动后，自由、民主、薪火三档价格标志均正常显示。
- [x] Flutter 运行日志无本次改造引起的 RenderFlex、溢出或资源加载错误。

## 自动检查

- 相关 35 项 Flutter widget 测试全部通过。
- 六个目标实现文件 `flutter analyze` 通过，无问题。
- `git diff --check` 通过。

## Open Questions

- 无。

## Follow-up Polish

- 无阻塞或 P3 跟进项。

final result: passed
