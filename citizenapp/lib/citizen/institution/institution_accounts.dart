// 统一机构账户行(ADR-028 决策 2)——合并公权「派生账户」与治理「固定账户」两套。
//
//
// - 创世治理机构:账户是 china 创世固定 hex,由 [Institution.builtinAccounts] 承载,不可派生。
// - 普通机构:主/费用/附加账户一律按名称路由本地派生(account_derivation 卡0,零网络)；
//   附加账户既可能是制度协议账户，也可能是普通自定义命名账户。
// - 余额另由链态服务批量补(ADR-018 R2 精确整键批量,不逐条)。

import 'dart:typed_data';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/shared/reserved_account_names.dart';

/// 单个机构账户行（标签 + AccountId + SS58 + 可选余额）。
class InstitutionAccountRow {
  const InstitutionAccountRow({
    required this.label,
    required this.accountId,
    required this.ss58Address,
    this.balanceYuan,
  });

  final String label;
  final String accountId;
  final String ss58Address;
  final double? balanceYuan;

  InstitutionAccountRow withBalance(double? yuan) => InstitutionAccountRow(
        label: label,
        accountId: accountId,
        ss58Address: ss58Address,
        balanceYuan: yuan,
      );
}

/// 由机构构造全部账户行:固定治理档用 china 固定账户,普通机构本地派生。
List<InstitutionAccountRow> institutionAccountIdRows(Institution inst) {
  final baked = inst.builtinAccounts;
  if (baked != null) {
    final rows = <InstitutionAccountRow>[
      _rowFromAccountId(kReservedNameMain, baked.mainAccountId),
      _rowFromAccountId(kReservedNameFee, baked.feeAccountId),
    ];
    final safety = baked.safetyFundAccountId;
    if (safety != null) {
      rows.add(_rowFromAccountId(kReservedNameSafetyFund, safety));
    }
    final he = baked.heAccountId;
    if (he != null) rows.add(_rowFromAccountId(kReservedNameHe, he));
    final stake = baked.stakeAccountId;
    if (stake != null) rows.add(_rowFromAccountId(kReservedNameStake, stake));
    return rows;
  }

  // 普通机构:主 + 费用 + 链快照列出的附加账户(本地派生)。
  // custom_account_names 是快照字段名，不代表其中每项都允许用户自定义注册：
  // 制度账户必须先按保留名路由到专用 op_tag，普通名称才回落 OP_NAME。
  final rows = <InstitutionAccountRow>[];
  final main = deriveInstitutionMainAccountId(inst.cidNumber);
  rows.add(InstitutionAccountRow(
    label: kReservedNameMain,
    accountId: accountIdText(main),
    ss58Address: ss58FromAccountId(main),
  ));
  final feeId = deriveInstitutionFeeAccountId(inst.cidNumber);
  rows.add(InstitutionAccountRow(
    label: kReservedNameFee,
    accountId: accountIdText(feeId),
    ss58Address: ss58FromAccountId(feeId),
  ));
  for (final name in inst.customAccountNames) {
    if (name.isEmpty || name == kReservedNameMain || name == kReservedNameFee) {
      continue;
    }
    final id = deriveInstitutionAccountIdByName(inst.cidNumber, name);
    rows.add(InstitutionAccountRow(
      label: name,
      accountId: accountIdText(id),
      ss58Address: ss58FromAccountId(id),
    ));
  }
  return rows;
}

InstitutionAccountRow _rowFromAccountId(String label, String accountId) {
  if (!isAccountIdText(accountId)) {
    throw const FormatException('机构 account_id 必须为小写 0x + 64 位十六进制');
  }
  final clean = accountId.substring(2);
  final bytes = Uint8List.fromList(
    List<int>.generate(
      clean.length ~/ 2,
      (i) => int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    ),
  );
  return InstitutionAccountRow(
    label: label,
    accountId: accountId,
    ss58Address: ss58FromAccountId(bytes),
  );
}
