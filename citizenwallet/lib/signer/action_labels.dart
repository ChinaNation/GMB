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
  'citizen_identity': '公民身份上链确认',
  'citizen_candidate_identity': '公民参选身份上链确认',
  'register_voting_identity': '注册公民链上身份',
  'upgrade_to_candidate_identity': '注册公民参选身份',
  'update_voting_identity': '更新公民链上身份',
  'update_candidate_identity': '更新公民参选身份',
  'revoke_identity': '吊销公民链上身份',
  'occupy_cid': '注册局占用CID',
  'occupy_cids_batch': '注册局批量占用CID',
  'revoke_cid': '注册局吊销CID',

  // 业务提案创建（propose_X）
  'propose_transfer': '发起转账提案',
  'propose_safety_fund_transfer': '安全基金转账提案',
  'propose_sweep_to_main': '手续费划转提案',
  // 个人多签为独立 pallet PersonalManage(7),
  // 'propose_create_personal' 是 decoder 输出 action 字符串,显式区分个人/机构提示文案。
  'propose_create_personal': '创建个人多签',
  'propose_create_public_institution': '创建公权机构',
  'propose_close_public_institution': '关闭公权机构账户提案',
  'update_public_institution_info': '更新公权机构信息',
  'add_public_institution_account': '新增公权机构账户',
  'propose_create_private_institution': '创建私权机构',
  'propose_close_private_institution': '关闭私权机构账户提案',
  'update_private_institution_info': '更新私权机构信息',
  'add_private_institution_account': '新增私权机构账户',
  'propose_close_personal': '关闭个人多签提案',
  'propose_destroy': '销毁决议提案',
  'propose_personal_admin_set_change': '管理员集合变更提案',
  'propose_replace_grandpa_key': 'GRANDPA 密钥提案',
  'propose_issuance': '决议发行提案',
  'propose_asset_issue': '创建链上资产提案',
  'propose_asset_mint': '链上资产增发提案',
  'propose_asset_burn': '链上资产销毁提案',
  'propose_asset_close': '关闭链上资产提案',
  'propose_asset_transfer': '链上资产划转提案',
  'propose_monitor_freeze': '监管冻结资产持仓提案',
  'propose_monitor_unfreeze': '监管解冻资产持仓提案',
  'propose_monitor_confiscate': '监管扣押资产提案',
  'propose_monitor_force_transfer': '监管强制划转资产提案',
  'propose_monitor_force_close': '监管封禁资产提案',

  // 业务提案幂等入口
  // 手动重试/取消统一显示为 retry_passed_proposal /
  // cancel_passed_proposal(在 votingengine 段已声明)。
  // 立法院（pallet=25）· 立法/修法/废法发起
  'propose_enact_law': '发起立法',
  'propose_amend_law': '发起修法',
  'propose_repeal_law': '发起废法',

  // 立法投票（pallet=26）· 立法专属投票引擎
  'cast_representative_vote': '代表机构表决',
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
  'propose_l2_fee_rate': '提案调整链下费率',
  'set_address_catalog_version': '设置地址库版本',
  'set_address_name': '设置地址名称',
  'remove_address_name': '删除地址名称',
  'set_address': '设置完整地址',
  'remove_address': '删除完整地址',
  'onchina_admin_action': '链上中国平台管理员治理',
};
