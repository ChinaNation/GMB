# citizenwallet 添加账户:混合式(下一个 + 指定序号)

状态:done(2026-07-27)
所属模块:Mobile(citizenwallet 冷钱包)

## 需求(用户拍板)
- 混合式:默认一键"添加下一个"(max+1)+ 高级"指定序号"(用于恢复非连续账户 / 加别处已注资的特定 `//N`)。
- 序号上界 **1989**(`//index` index 最大 1989;账户0 是创建时主账户,指定序号仅 1–1989)。
- 新账户默认名 `账户N`,用户后续自行改名(renameAccount 已有)。

## 落地
- `wallet_manager.dart`:`static const int maxAccountIndex = 1989`;`addAccount(masterId, {int? index})`——index 空=max+1(封顶 1989 抛)、非空=指定序号(范围 1–1989 校验 + 去重"账户N已存在",校验先于读种子);序号可非连续/回填。派生/金标/存储不动。
- `wallet_detail_page.dart`:「添加账户」按钮 → 底部菜单(复用 home_page 视觉):①添加下一个(副标题"将派生 //N")②指定序号(弹数字框 1–1989)→ `_doAdd({index})`;提示文案加"可指定序号精确还原"。
- `wallet_manager_test.dart` 新增:指定 //3(仅[0]时,accountId 对拍 `fromUri('//3')`)、回填([0,3]→加//1→[0,1,3])、已存在/序号0/越界 拒、上界1989 可加;保留"add next=max+1"与"删中间账户仍max+1"。

## 验收
- `dart analyze` **0** + `flutter test` **222 passed**;残留(`_addAccount`)复扫 0;改动只在主检出 citizenwallet(wallet_manager/wallet_detail_page/wallet_manager_test 三文件)。
