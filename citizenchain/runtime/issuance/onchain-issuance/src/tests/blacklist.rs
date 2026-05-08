//! 字符串黑名单测试占位(命中 / 不命中 / 大小写 / 中英文混合)。
//!
//! 实装步骤(后续任务卡 A):
//! 1. mock runtime 注入 GenesisConfig 的 default_blacklist_words
//! 2. 构造命中字段(USD-Token / 锚定积分 / 数字人民币 等)发起 issue,断言 BlacklistedWord error
//! 3. 构造干净字段(SafeCoin / 校园积分 等)发起 issue,断言 ok
//! 4. 单独测试 RuntimeUpgrade 路径添词/删词后的行为变化
//!
//! 中文注释:基础校验逻辑已在 `validation::tests::blacklist_*` 单元测试覆盖,
//! 本文件主要测 GenesisConfig 接入 + extrinsic 入口反应。

#[test]
fn placeholder_blacklist_compiles() {
    assert!(true);
}
