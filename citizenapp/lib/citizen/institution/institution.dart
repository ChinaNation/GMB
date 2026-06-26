// 统一机构实体(ADR-028 决策 2)——合并公权 `PublicInstitutionEntity` 与治理
// `InstitutionInfo` 两套并行模型为一套。
//
// 中文注释:
// - 所有机构本质都是按 CID `institution_code` 分类的公权多签账户,差异只在权责。
//   本实体是五子 tab(广场/立法/选举/治理/公权)与统一详情页的唯一机构模型。
// - 身份字段来自目录(CID-BFF + Isar);治理三档(NRC/PRC/PRB)的固定账户 hex
//   由 [builtinAccounts] 承载(china 创世常量,不可派生),其余机构主/费/自定义账户
//   一律本地派生(account_derivation,零网络)。
// - 机构分类(orgType / 是否固定治理 / 是否机构账户)统一从机构码派生,
//   单一源 = `governance/shared/institution_code_label.dart`,绝不另立第二套。

import 'dart:typed_data';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

/// 单个机构的统一信息载体(不可变)。
class Institution {
  const Institution({
    required this.cidNumber,
    required this.cidFullName,
    required this.institutionCode,
    this.cidShortName,
    this.status = 'ACTIVE',
    this.provinceCode = '',
    this.cityCode = '',
    this.townCode = '',
    this.parentCidNumber,
    this.legalRepName,
    this.accountCount = 0,
    this.customAccountNames = const [],
    this.builtinAccounts,
  });

  /// 链上身份标识(CID 号,机构码内含于第二段)。
  final String cidNumber;

  /// 机构全称(与链端 `cid_full_name` 对齐)。
  final String cidFullName;

  /// 机构简称(与链端 `cid_short_name` 对齐;可空,展示回退全称)。
  final String? cidShortName;

  /// 机构码(CID 号第二段,如 NRC/PRC/PRB/CGOV/NLG…)。机构分类唯一依据。
  final String institutionCode;

  final String status;

  /// 所属省/市/镇 code(行政区唯一真源键;名字由字典 join,见 ADR-021)。
  final String provinceCode;
  final String cityCode;
  final String townCode;

  /// 所属上级法人 CID 号(仅非法人 UNIN 机构有值;法人为 null)。
  final String? parentCidNumber;

  /// 法定代表人姓名(公开目录字段,无则 null → 详情页留空)。
  final String? legalRepName;

  final int accountCount;
  final List<String> customAccountNames;

  /// 固定治理档(NRC/PRC/PRB)的链上固定账户集合(china 创世常量,不可派生)。
  /// 普通机构为 null —— 账户走本地派生。由仓库在加载固定治理档机构时附加。
  final InstitutionAccounts? builtinAccounts;

  /// 机构类型(单一源,由机构码派生):NRC/PRC/PRB → 对应固定治理档;其余 → 机构账户。
  int get orgType {
    switch (institutionCode) {
      case 'NRC':
        return OrgType.nrc;
      case 'PRC':
        return OrgType.prc;
      case 'PRB':
        return OrgType.prb;
      default:
        return OrgType.account;
    }
  }

  /// 是否固定治理档(国储会/省储会/省储行)。
  bool get isFixedGovernance =>
      InstitutionCodeLabel.isFixedGovernance(institutionCode);

  /// 是否非法人机构(挂上级法人;详情页加显「所属上级法人全称」)。
  bool get isUnincorporated =>
      parentCidNumber != null && parentCidNumber!.isNotEmpty;

  /// 详情页顶部标题用:简称优先,回退全称(ADR-028 决策 6)。
  String get displayName =>
      (cidShortName != null && cidShortName!.isNotEmpty)
          ? cidShortName!
          : cidFullName;

  /// 主账户 AccountId:固定治理档用 china 固定 hex,其余本地派生(行为保持)。
  Uint8List mainAccountId() {
    final baked = builtinAccounts?.mainAccount;
    if (baked != null && baked.isNotEmpty) {
      return _hexToBytes(baked);
    }
    return deriveInstitutionMainAccountId(cidNumber);
  }

  /// 主账户 hex(32 字节,不含 0x)。
  String get mainAccountHex => hexFromAccountId(mainAccountId());

  /// 附加固定治理档账户集合(仓库加载 NRC/PRC/PRB 时调用)。
  Institution withBuiltinAccounts(InstitutionAccounts accounts) => Institution(
        cidNumber: cidNumber,
        cidFullName: cidFullName,
        institutionCode: institutionCode,
        cidShortName: cidShortName,
        status: status,
        provinceCode: provinceCode,
        cityCode: cityCode,
        townCode: townCode,
        parentCidNumber: parentCidNumber,
        legalRepName: legalRepName,
        accountCount: accountCount,
        customAccountNames: customAccountNames,
        builtinAccounts: accounts,
      );

  /// 由治理静态注册表项构造(回退路径:目录未同步到时,治理机构仍可展示)。
  /// 地域 code 注册表项不带,留空 → 所属地按目录就绪后回填;账户用 baked 集合。
  factory Institution.fromGovernanceInfo(InstitutionInfo info) {
    final code = switch (info.orgType) {
      OrgType.nrc => 'NRC',
      OrgType.prc => 'PRC',
      OrgType.prb => 'PRB',
      _ => '',
    };
    return Institution(
      cidNumber: info.cidNumber,
      cidFullName: info.cidFullName,
      cidShortName: info.cidShortName,
      institutionCode: code,
      builtinAccounts: info.accounts,
    );
  }

  /// 由公权目录 Isar 实体构造(统一路径:全部机构身份来自目录)。
  factory Institution.fromPublicEntity(PublicInstitutionEntity e) {
    return Institution(
      cidNumber: e.cidNumber,
      cidFullName: e.cidFullName,
      cidShortName: e.cidShortName,
      institutionCode: e.institutionCode,
      status: e.status,
      provinceCode: e.provinceCode,
      cityCode: e.cityCode,
      townCode: e.townCode,
      parentCidNumber: e.parentCidNumber,
      legalRepName: e.legalRepName,
      accountCount: e.accountCount,
      customAccountNames: e.customAccountNames,
    );
  }

  static Uint8List _hexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return Uint8List.fromList(
      List<int>.generate(
        clean.length ~/ 2,
        (i) => int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16),
        growable: false,
      ),
    );
  }
}
