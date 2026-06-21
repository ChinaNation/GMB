// 公权机构账户行 —— 全部本地派生(ADR-018 §九 卡C)。
//
// 中文注释:主/费/自定义账户地址一律用卡0 account_derivation 本地派生(零网络),
// 余额另由 chainData 批量补。account 发现 100% 本地,不扫链(R1)。

import 'package:citizenapp/governance/shared/account_derivation.dart';
import 'package:citizenapp/governance/shared/reserved_account_names.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

class PublicAccountRow {
  const PublicAccountRow({
    required this.label,
    required this.addressHex,
    required this.addressSs58,
    this.balanceYuan,
  });

  final String label;
  final String addressHex;
  final String addressSs58;
  final double? balanceYuan;

  PublicAccountRow withBalance(double? yuan) => PublicAccountRow(
        label: label,
        addressHex: addressHex,
        addressSs58: addressSs58,
        balanceYuan: yuan,
      );
}

/// 由机构本地派生全部账户行:主账户 + 费用账户 + 自定义账户。
List<PublicAccountRow> deriveAccountRows(PublicInstitutionEntity inst) {
  final rows = <PublicAccountRow>[];

  final main = deriveInstitutionMainAccountId(inst.cidNumber);
  rows.add(PublicAccountRow(
    label: kReservedNameMain,
    addressHex: hexFromAccountId(main),
    addressSs58: ss58FromAccountId(main),
  ));

  final fee = deriveInstitutionFeeAccountId(inst.cidNumber);
  rows.add(PublicAccountRow(
    label: kReservedNameFee,
    addressHex: hexFromAccountId(fee),
    addressSs58: ss58FromAccountId(fee),
  ));

  for (final name in inst.customAccountNames) {
    if (name.isEmpty || isForbiddenAccountName(name)) continue;
    final id = deriveInstitutionCustomAccountId(inst.cidNumber, name);
    rows.add(PublicAccountRow(
      label: name,
      addressHex: hexFromAccountId(id),
      addressSs58: ss58FromAccountId(id),
    ));
  }

  return rows;
}
