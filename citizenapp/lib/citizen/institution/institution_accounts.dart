// 统一机构账户行(ADR-028 决策 2)——合并公权「派生账户」与治理「固定账户」两套。
//
// 中文注释:
// - 固定治理档(NRC/PRC/PRB):账户是 china 创世固定 hex(主/费用/安全基金/两和基金/
//   永久质押),由 [Institution.builtinAccounts] 承载,不可派生。
// - 普通机构:主/费用/自定义账户一律本地派生(account_derivation 卡0,零网络)。
// - 余额另由链态服务批量补(ADR-018 R2 精确整键批量,不逐条)。

import 'dart:typed_data';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/shared/reserved_account_names.dart';

/// 单个机构账户行(标签 + hex + SS58 + 可选余额)。
class InstitutionAccountRow {
  const InstitutionAccountRow({
    required this.label,
    required this.accountHex,
    required this.addressSs58,
    this.balanceYuan,
  });

  final String label;
  final String accountHex;
  final String addressSs58;
  final double? balanceYuan;

  InstitutionAccountRow withBalance(double? yuan) => InstitutionAccountRow(
        label: label,
        accountHex: accountHex,
        addressSs58: addressSs58,
        balanceYuan: yuan,
      );
}

/// 由机构构造全部账户行:固定治理档用 china 固定账户,普通机构本地派生。
List<InstitutionAccountRow> institutionAccountRows(Institution inst) {
  final baked = inst.builtinAccounts;
  if (baked != null) {
    final rows = <InstitutionAccountRow>[_rowFromHex('主账户', baked.mainAccount)];
    final fee = baked.feeAccount;
    if (fee != null) rows.add(_rowFromHex('费用账户', fee));
    final safety = baked.safetyFundAccount;
    if (safety != null) rows.add(_rowFromHex('安全基金账户', safety));
    final he = baked.heAccount;
    if (he != null) rows.add(_rowFromHex('两和基金账户', he));
    final stake = baked.stakeAccount;
    if (stake != null) rows.add(_rowFromHex('永久质押', stake));
    return rows;
  }

  // 普通机构:主 + 费用 + 自定义(本地派生)。
  final rows = <InstitutionAccountRow>[];
  final main = deriveInstitutionMainAccountId(inst.cidNumber);
  rows.add(InstitutionAccountRow(
    label: kReservedNameMain,
    accountHex: hexFromAccountId(main),
    addressSs58: ss58FromAccountId(main),
  ));
  final feeId = deriveInstitutionFeeAccountId(inst.cidNumber);
  rows.add(InstitutionAccountRow(
    label: kReservedNameFee,
    accountHex: hexFromAccountId(feeId),
    addressSs58: ss58FromAccountId(feeId),
  ));
  for (final name in inst.customAccountNames) {
    if (!isRegistrableCustomName(name)) continue;
    final id = deriveInstitutionCustomAccountId(inst.cidNumber, name);
    rows.add(InstitutionAccountRow(
      label: name,
      accountHex: hexFromAccountId(id),
      addressSs58: ss58FromAccountId(id),
    ));
  }
  return rows;
}

InstitutionAccountRow _rowFromHex(String label, String hex) {
  final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
  final bytes = Uint8List.fromList(
    List<int>.generate(
      clean.length ~/ 2,
      (i) => int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    ),
  );
  return InstitutionAccountRow(
    label: label,
    accountHex: clean.toLowerCase(),
    addressSs58: ss58FromAccountId(bytes),
  );
}
