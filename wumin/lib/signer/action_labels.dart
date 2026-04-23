/// 交易 action 英文标识 → 中文显示名映射。
///
/// 冷钱包 UI 显示时查此表翻译，未命中则原样显示英文。
///
/// Phase 3（2026-04-22）：
/// - 所有管理员投票统一 action = `internal_vote`，业务 `vote_X` 标签全部删除。
/// - 联合投票、公民投票、任意人触发的终态执行单独保留。
const Map<String, String> actionLabels = {
  'transfer': '转账',

  // 投票引擎（pallet=9）· 统一投票入口
  'internal_vote': '管理员投票',
  'joint_vote': '联合投票',
  'citizen_vote': '公民投票',
  'finalize_proposal': '触发提案执行',

  // 业务提案创建（propose_X）
  'propose_transfer': '发起转账提案',
  'propose_safety_fund_transfer': '安全基金转账提案',
  'propose_sweep_to_main': '手续费划转提案',
  'propose_create': '创建多签账户',
  'propose_create_personal': '创建个人多签',
  'propose_close': '关闭多签提案',
  'propose_destroy': '销毁决议提案',
  'propose_admin_replacement': '管理员替换提案',
  'propose_replace_grandpa_key': 'GRANDPA 密钥提案',
  'propose_resolution_issuance': '决议发行提案',

  // 业务提案执行重试（execute_X）
  'execute_transfer': '执行机构转账',
  'execute_safety_fund_transfer': '执行安全基金转账',
  'execute_sweep_to_main': '执行手续费划转',
  'execute_destroy': '执行决议销毁',
  'execute_admin_replacement': '执行管理员替换',
  'execute_replace_grandpa_key': '执行 GRANDPA 密钥替换',
  'cancel_failed_replace_grandpa_key': '取消失败的 GRANDPA 替换',
  'cleanup_rejected_proposal': '清理被否决提案',
  'register_sfid_institution': '登记 SFID 机构信息',

  // Runtime 升级
  'propose_runtime_upgrade': 'Runtime 升级提案',
  'developer_direct_upgrade': '开发期直升 Runtime',

  // 其他
  'activate_admin': '管理员激活',
  'offchain_pay': '链下支付',
  'bind_clearing_bank': '绑定清算行',
  'offchain_clearing_pay': '清算行扫码付款',
};
