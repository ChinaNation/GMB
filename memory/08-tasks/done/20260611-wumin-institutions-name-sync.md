# 任务卡：wumin 机构注册表名称同步链端真源

## 背景

20260611-wuminapp-institution-registry-regen 的遗留待办:`wumin/lib/chain/institutions.dart` 机构名仍是 `c13e0f82`(身份ID协议重构)改名前的旧名:

- 国储会:国家储备委员会 →(链端现为)国家公民储备委员会
- 43 省储会:X省储备委员会 →(链端现为)X省公民储备委员会
- 43 省储行:已是新名(公民储备银行),核对即可

链端唯一真源 = `citizenchain/runtime/primitives/china/china_{cb,ch}.rs` 的 `sfid_full_name`;节点端 `node/src/governance/registry.rs` 直接读该常量,无第二份手抄。

已查实:此表当前 wumin lib/test 内**零引用**(机构标签走 org 字节 `_institutionAccountLabel`,创建凭证机构名直接解码自 payload),属数据残留,本卡只做字面同步,不做删除决策。

## 方案

1. 44 个旧名(国储会 1 + 省储会 43)按链端 `sfid_full_name` 同步;文件头"唯一事实源"注释指向已迁移的实际路径(primitives china_*.rs + node registry.rs)。
2. 脚本逐字段核对 87 机构(name + sfid_number)与 china_{cb,ch}.rs 0 mismatch。
3. wumin `flutter analyze` + `flutter test` 回归。

## 验收

- [ ] 87 机构 name/sfid_number 与链端真源逐字段一致
- [ ] analyze 0 issue + test 全过

## 完工记录(2026-06-11)

- `wumin/lib/chain/institutions.dart`:国储会 + 43 省储会名称同步为「公民储备委员会」;文件头唯一事实源注释修正为 primitives china_{cb,ch}.rs + node registry.rs(原指向的 node/src/ui/governance/mod.rs 已不存在);enum 文档注释同步。
- 脚本核对 87 机构 name + sfid_number 与链端真源:0 mismatch。
- wumin `flutter analyze` 0 issue,`flutter test` 116/116 全过。
- 验收两项全勾。
