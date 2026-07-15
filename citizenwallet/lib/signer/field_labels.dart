/// 扫码确认页 reviewFields 字段名中文翻译单源。
///
/// decoder(payload_decoder.dart)的 `reviewFields` 保留英文机器 key 用于
/// 跨端验真,到 UI 层统一经本文件翻译。payload_decoder 新增 reviewFields key
/// 时必须同步登记本表,并在 test/signer/field_labels_test.dart 补断言;
/// 未登记字段用中文兜底,避免直接把英文 key 展示给用户。
library;

/// fields value 转换(如 approve: true → 赞成)。
String fieldValueText(String key, String value) {
  if (key == 'approve') return value == 'true' ? '赞成' : '反对';
  return value;
}

/// reviewFields key → 中文字段名。
String fieldLabelText(String key) {
  if (key.startsWith('amount_')) {
    final accountName = key.substring('amount_'.length);
    return accountName.isEmpty ? '账户金额' : '$accountName金额';
  }
  return switch (key) {
    'to' => '收款账户',
    'beneficiary' => '收款账户',
    'account' => '账户',
    'institution' => '机构账户',
    'institution_code' => '机构类型',
    'cid_number' => 'CID编号',
    'cid_full_name' => '机构全称',
    'account_name' => '账户名称',
    'amount_yuan' => '金额',
    'total_amount_yuan' => '总金额',
    'remark' => '备注',
    'reason' => '原因',
    'proposal_id' => '提案编号',
    'approve' => '投票意见',
    'admins' => '管理员',
    'admins_len' => '管理员人数',
    'threshold' => '阈值',
    'regular_threshold' => '普通阈值',
    'create_threshold' => '创建阈值',
    'new_threshold' => '新阈值',
    'issuer_cid_number' => '签发机构编号',
    'issuer_main_account' => '签发机构账户',
    'signer_pubkey' => '签发管理员',
    'scope_province_name' => '省级范围',
    'scope_city_name' => '市级范围',
    'allocation_count' => '分配项数',
    'action_type' => '操作类型',
    'actor_province_name' => '操作省份',
    'actor_pubkey' => '操作管理员',
    'target' => '目标账户',
    'before_hash' => '变更前哈希',
    'after_hash' => '变更后哈希',
    'admin_pubkey' => '管理员账户',
    'pubkey' => '公钥',
    'bank_main' => '清算行主账户',
    'new_bank' => '新清算行',
    'peer_id' => '节点标识',
    'rpc_domain' => '节点域名',
    'rpc_port' => '节点端口',
    'new_domain' => '新节点域名',
    'new_port' => '新节点端口',
    'new_key' => '新密钥',
    'expires_at' => '过期时间',
    'title' => '法律标题',
    'tier' => '法律层级',
    'vote_type' => '表决类型',
    'scope_code' => '行政区代码',
    'houses' => '表决院',
    'chapter_count' => '章数',
    'article_count' => '条数',
    'effective_at' => '生效区块',
    'law_id' => '法律编号',
    'eligible_total' => '合格选民数',
    // 公民链上身份(citizen_identity / register_voting_identity /
    // upgrade_to_candidate_identity)。
    'registrar_account' => '注册机构账户',
    'identity_level' => '身份类型',
    'wallet_account' => '公民钱包账户',
    'citizen_age_years' => '周岁年龄',
    'valid_range' => '护照有效期',
    'citizen_status' => '身份状态',
    'residence' => '居住地',
    'birth_place' => '出生地',
    'birth_date' => '出生日期',
    'citizen_full_name' => '公民姓名',
    'citizen_sex' => '公民性别',
    _ => '未知字段',
  };
}
