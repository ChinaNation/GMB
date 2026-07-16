//! NRC 监管 5 动作测试占位(freeze/unfreeze/confiscate/forceTransfer/forceClose)。
//!
//! 实装步骤(后续任务卡 B):
//! 1. mock runtime 中预置 NRC CID、监管管理员、用户代币与持币账户
//! 2. 走 JointVote 通过路径 → callback → execute_monitor_*
//! 3. 断言 storage / event / 持仓变化 / 30 天封禁倒计时
//! 4. 失败分支:非 NRC 主体调用 reject、reason_hash 缺失 reject

#[test]
fn placeholder_monitor_compiles() {
    // 框架阶段占位,业务实装时本测试整体替换为真实监管场景。
    assert!(true);
}
