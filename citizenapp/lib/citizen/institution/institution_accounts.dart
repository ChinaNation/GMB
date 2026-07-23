// 统一机构账户行(ADR-028 决策 2)——合并公权「派生账户」与治理「固定账户」两套。
//
//
// - 创世治理机构:账户是 china 创世固定 hex,由 [Institution.builtinAccounts] 承载,不可派生。
// - 普通机构:主/费用/自定义账户一律本地派生(account_derivation 卡0,零网络)。
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
      _rowFromAccountId('主账户', baked.mainAccountId),
      _rowFromAccountId('费用账户', baked.feeAccountId),
    ];
    final safety = baked.safetyFundAccountId;
    if (safety != null) rows.add(_rowFromAccountId('安全基金账户', safety));
    final he = baked.heAccountId;
    if (he != null) rows.add(_rowFromAccountId('两和基金账户', he));
    final stake = baked.stakeAccountId;
    if (stake != null) rows.add(_rowFromAccountId('永久质押', stake));
    return rows;
  }

  // 普通机构:主 + 费用 + 自定义(本地派生)。
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
    if (!isRegistrableCustomName(name)) continue;
    final id = deriveInstitutionCustomAccountId(inst.cidNumber, name);
    rows.add(InstitutionAccountRow(
      label: name,
      accountId: accountIdText(id),
      ss58Address: ss58FromAccountId(id),
    ));
  }
  return rows;
}

InstitutionAccountRow _rowFromAccountId(String label, String accountId) {
  if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId)) {
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
