/// 交易 action 英文标识 → 中文显示名映射。
///
/// 钱包 UI 显示时查此表翻译，未命中则原样显示英文。
///
/// - 所有管理员投票统一 action = `internal_vote`；联合投票、公民投票、
///   任意人触发的终态执行单独保留。
/// - 手动重试/取消统一走 `retry_passed_proposal` / `cancel_passed_proposal`。
const Map<String, String> actionLabels = {
  'transfer': '转账',

  // 投票引擎（pallet=9）· 统一投票入口
  'internal_vote': '管理员投票',
  'joint_vote': '联合投票',
  'cast_referendum': '联合公投',
  'finalize_proposal': '触发提案执行',
  'retry_passed_proposal': '手动执行已通过提案',
  'cancel_passed_proposal': '取消已通过但不可执行的提案',

  // 业务提案创建（propose_X）
  'propose_transfer': '发起转账提案',
  'propose_safety_fund_transfer': '安全基金转账提案',
  'propose_sweep_to_main': '手续费划转提案',
  // 个人多签为独立 pallet PersonalAdmins(7),
  // 'propose_create_personal' 是 decoder 输出 action 字符串,显式区分个人/机构提示文案。
  'propose_create_personal': '创建个人多签',
  'propose_create_institution': '创建机构多签账户',
  'propose_close_institution': '注销机构多签提案',
  'propose_close_personal': '关闭个人多签提案',
  'cleanup_rejected_personal_proposal': '清理被拒个人多签提案',
  'propose_destroy': '销毁决议提案',
  'propose_personal_admin_set_change': '管理员集合变更提案',
  'propose_genesis_admin_set_change': '管理员集合变更提案',
  'propose_public_admin_set_change': '管理员集合变更提案',
  'propose_private_admin_set_change': '管理员集合变更提案',
  'propose_replace_grandpa_key': 'GRANDPA 密钥提案',
  'propose_resolution_issuance': '决议发行提案',

  // 业务提案幂等入口
  // 手动重试/取消统一显示为 retry_passed_proposal /
  // cancel_passed_proposal(在 votingengine 段已声明)。
  'cleanup_rejected_proposal': '清理被否决提案',
  'register_cid_institution': '登记 CID 机构信息',

  // 立法院（pallet=27）· 立法/修法/废法发起
  'propose_enact_law': '发起立法',
  'propose_amend_law': '发起修法',
  'propose_repeal_law': '发起废法',

  // 立法投票（pallet=28）· 立法专属投票引擎
  'prepare_legislation_snapshot': '准备人口快照',
  'cast_house_vote': '院内表决',
  'cast_referendum_vote': '特别案公投',
  'executive_sign': '行政签署',
  'override_sign': '三人会签',
  'guard_vote': '护宪终审',

  // 协议升级
  'propose_runtime_upgrade': '协议升级提案',
  'developer_direct_upgrade': '开发期协议直升',

  // 其他
  'activate_admin_account': '管理员激活',
  'decrypt_admin': '解密清算行管理员',
  'bind_clearing_bank': '绑定清算行',
  'deposit_clearing_bank': '充值到清算行',
  'withdraw_clearing_bank': '从清算行提现',
  'switch_clearing_bank': '切换清算行',
  'register_clearing_bank': '声明清算行节点',
  'update_clearing_bank_endpoint': '更新清算行端点',
  'unregister_clearing_bank': '注销清算行节点',
  'cid_admin_action': 'CID 管理员治理',
};
