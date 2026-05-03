/// 交易 action 英文标识 → 中文显示名映射。
///
/// 冷钱包 UI 显示时查此表翻译，未命中则原样显示英文。
///
/// Phase 3（2026-04-22）：
/// - 所有管理员投票统一 action = `internal_vote`，业务 `vote_X` 标签全部删除。
/// - 联合投票、公民投票、任意人触发的终态执行单独保留。
///
/// Phase 4（2026-05-02）：
/// - 业务 pallet 的 `execute_xxx` / `cancel_failed_xxx` wrapper 全部删除，
///   统一到 `retry_passed_proposal` / `cancel_passed_proposal`，对应 7 个旧
///   action label 一并下线。
const Map<String, String> actionLabels = {
  'transfer': '转账',

  // 投票引擎（pallet=9）· 统一投票入口
  'internal_vote': '管理员投票',
  'joint_vote': '联合投票',
  'citizen_vote': '公民投票',
  'finalize_proposal': '触发提案执行',
  'retry_passed_proposal': '手动执行已通过提案',
  'cancel_passed_proposal': '取消已通过但不可执行的提案',

  // 业务提案创建（propose_X）
  'propose_transfer': '发起转账提案',
  'propose_safety_fund_transfer': '安全基金转账提案',
  'propose_sweep_to_main': '手续费划转提案',
  'propose_create': '创建多签账户',
  'propose_create_personal': '创建个人多签',
  'propose_create_institution': '创建机构多签账户',
  'propose_close': '关闭多签提案',
  'propose_destroy': '销毁决议提案',
  'propose_admin_replacement': '管理员替换提案',
  'propose_replace_grandpa_key': 'GRANDPA 密钥提案',
  'propose_resolution_issuance': '决议发行提案',

  // 业务提案幂等入口
  // Phase 4(2026-05-02): execute_xxx / cancel_failed_xxx 的 7 个旧 label
  // 已删除,所有手动重试/取消统一显示为 retry_passed_proposal /
  // cancel_passed_proposal(在 voting-engine 段已声明)。
  'cleanup_rejected_proposal': '清理被否决提案',
  'register_sfid_institution': '登记 SFID 机构信息',

  // Runtime 升级
  'propose_runtime_upgrade': 'Runtime 升级提案',
  'developer_direct_upgrade': '开发期直升 Runtime',

  // 其他
  'activate_admin': '管理员激活',
  'decrypt_admin': '解密清算行管理员',
  'offchain_pay': '链下支付',
  'bind_clearing_bank': '绑定清算行',
  'deposit_clearing_bank': '充值到清算行',
  'withdraw_clearing_bank': '从清算行提现',
  'switch_clearing_bank': '切换清算行',
  'offchain_clearing_pay': '清算行扫码付款',
  'register_clearing_bank': '声明清算行节点',
  'update_clearing_bank_endpoint': '更新清算行端点',
  'unregister_clearing_bank': '注销清算行节点',
};
