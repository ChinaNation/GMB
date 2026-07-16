import 'dart:convert';
import 'dart:typed_data';

/// 机构码(CID institution_code)热钱包端表示与治理分类 = institution_code_label.dart
///
/// (铁律):
/// 本文件逐字镜像冷钱包 `citizenwallet/lib/signer/institution_code.dart`(同一套
/// 104 码)。链上治理统一使用 4 字节 `institution_code`
/// ([u8;4] 原始码字节,3 字符码右补 `0`)。热钱包用本
/// 文件的纯函数从机构码派生治理分类(是不是固定治理档 / 个人多签 / 机构账户)，
/// 绝不另立第二套分类。
///
/// 字节表示：机构码是 3~4 个大写 ASCII 字符，统一用 4 字节，3 字符码右补 `0`：
///   NRC → [78,82,67,0]  CGOV → [67,71,79,86]  PMUL → [80,77,85,76]
class InstitutionCodeLabel {
  const InstitutionCodeLabel._();
  // 治理相关机构码常量(4 字节,3 字符码末位补 0)
  /// 国家储委会(固定治理档)。"NRC\0"。
  static const List<int> nrc = [78, 82, 67, 0];

  /// 省公民储备委员会(固定治理档)。"PRC\0"。
  static const List<int> prc = [80, 82, 67, 0];

  /// 省公民储备银行(固定治理档)。"PRB\0"。
  static const List<int> prb = [80, 82, 66, 0];

  /// 个人多签账户(不发号，仅链上/后端分类常量)。"PMUL"。
  static const List<int> pmul = [80, 77, 85, 76];
  // 字节 ↔ 字符串
  /// 取前 4 字节，去掉尾部 0 字节，UTF-8 → 大写字符串。
  static String codeToString(List<int> bytes) {
    var end = bytes.length < 4 ? bytes.length : 4;
    while (end > 0 && bytes[end - 1] == 0) {
      end--;
    }
    if (end == 0) return '';
    return utf8
        .decode(bytes.sublist(0, end), allowMalformed: true)
        .toUpperCase();
  }

  /// 字符串机构码 → 4 字节(右补 0，超 4 截断)。
  static List<int> codeBytes(String code) {
    final raw = utf8.encode(code.toUpperCase());
    final out = Uint8List(4);
    for (var i = 0; i < 4 && i < raw.length; i++) {
      out[i] = raw[i];
    }
    return out;
  }

  // 机构码分类清单(与链端 PUBLIC/PRIVATE/UNINCORPORATED 同源)
  /// 公权法人机构码(A 国家级 38 + B 省级 17 + C 市级 17 + D 镇级 14 + 公立大学/学校 2)= 88。
  static const Set<String> _publicLegalCodes = <String>{
    // A 国家级单体(38)
    'PRS', 'FSC', 'FIB', 'FSS', 'FPR', 'FRG', 'MFA', 'MDF',
    'ARM', 'NAV', 'AIR', 'SPF', 'JOS', 'ARC', 'NVC', 'AFC',
    'SFC', 'MHS', 'NGB', 'NGC', 'MCW', 'FDA', 'MHU', 'MAG',
    'MCM', 'MFT', 'MEN', 'MTR', 'NLG', 'NSN', 'NRP', 'NJD',
    'NSP', 'FAC', 'FAU', 'FIV', 'NED', 'NRC',
    // B 省级类型(17)
    'PGV', 'PLG', 'PSN', 'PRP', 'PJD', 'PSP', 'PRC', 'PRB',
    'PDF', 'PHS', 'PCW', 'PHU', 'PAG', 'PCM', 'PFT', 'PEN',
    'PTR',
    // C 市级类型(17)
    'CGOV', 'CLEG', 'CSUP', 'CJUD', 'CEDU', 'CSLF', 'CDEF', 'CHSC', 'CCWF',
    'CHUD', 'CAGR', 'CCOM', 'CFIN', 'CENR', 'CTRN', 'CREG', 'CPOL',
    // D 镇级类型(14)
    'TGOV', 'TCWF', 'THUD', 'TAGR', 'TFIN', 'TDEF', 'THSC', 'TCOM', 'TENR',
    'TTRN', 'TPOL', 'TSLF', 'TSUP', 'TJUD',
    // 公立大学 / 公立学校
    'GUN', 'GSCH',
  };

  /// 私权法人机构码(有限合伙/股权/股份/公益/注册协会 + 私立/教会大学/学校)= 9。
  static const Set<String> _privateLegalCodes = <String>{
    'SFLP',
    'SFGQ',
    'SFGF',
    'SFGY',
    'SFAS',
    'SUN',
    'JUN',
    'SFSC',
    'JSCH',
  };

  /// 非法人机构码(个体经营/无限合伙/非法人组织)= 3。
  static const Set<String> _unincorporatedCodes = <String>{
    'SFGT',
    'SFGP',
    'UNIN',
  };
  // 治理策略派生(纯函数，热钱包唯一分类来源)
  /// 是否为固定治理档机构码(国家储委会/省储委会/省储行/联邦注册局/国家司法院)。
  static bool isFixedGovernance(String code) {
    return code == 'NRC' ||
        code == 'PRC' ||
        code == 'PRB' ||
        code == 'FRG' ||
        code == 'NJD';
  }

  /// 固定治理档制度阈值，逐字镜像 runtime
  /// `fixed_governance_pass_threshold`；其它机构返回 null 并读取链上动态阈值。
  static int? fixedGovernanceThreshold(String code) => switch (code) {
        'NRC' => 13,
        'PRC' || 'PRB' => 6,
        'FRG' => 3,
        'NJD' => 8,
        _ => null,
      };

  /// 是否为个人多签账户机构码(PMUL)。
  static bool isPersonal(String code) {
    return code == 'PMUL';
  }

  /// 是否为公权法人机构码。
  static bool isPublicLegal(String code) => _publicLegalCodes.contains(code);

  /// 是否为私权法人机构码。
  static bool isPrivateLegal(String code) => _privateLegalCodes.contains(code);

  /// 是否为非法人机构码。
  static bool isUnincorporated(String code) =>
      _unincorporatedCodes.contains(code);

  /// 是否为机构账户机构码(公权/私权/非法人法人实体，经机构管理(public/private-manage)注册多签)。
  ///
  /// 个人/个人多签不算机构账户；
  /// 固定治理档(NRC/PRC/PRB/FRG/NJD)是 china 内建创世账户，走固定治理路径，也不算机构账户。
  static bool isInstitution(String code) {
    if (isFixedGovernance(code)) return false;
    return isPublicLegal(code) ||
        isPrivateLegal(code) ||
        _unincorporatedCodes.contains(code);
  }

  /// 是否为注册多签动态阈值账户机构码(个人多签 或 机构账户)。
  /// 固定治理档不在内。镜像链端 `is_registered_multisig_code`。
  static bool isRegisteredMultisig(String code) {
    return isPersonal(code) || isInstitution(code);
  }

  /// 是否归 PublicAdmins 管理。
  static bool isPublicAdminCode(String code) {
    return isPublicLegal(code) || isFixedGovernance(code);
  }

  /// 是否归 PrivateAdmins 管理。
  ///
  /// 非法人不是私权同义词。SFGT/SFGP/UNIN 只能说明是非法人机构码,
  /// 不能决定管理员模块;必须由 CID 注册关系按所属公法人/私法人显式路由。
  static bool isPrivateAdminCode(String code) {
    return isPrivateLegal(code);
  }

  /// 非法人管理员模块候选码。调用方必须再结合所属法人决定 public/private。
  static bool isUnincorporatedAdminCode(String code) {
    return isUnincorporated(code);
  }

  /// 是否可被 PublicAdmins 保存。仅作为显式路由后的 storage 能力判断。
  static bool canStorePublicAdminCode(String code) {
    return isPublicAdminCode(code) || isUnincorporatedAdminCode(code);
  }

  /// 是否可被 PrivateAdmins 保存。仅作为显式路由后的 storage 能力判断。
  static bool canStorePrivateAdminCode(String code) {
    return isPrivateAdminCode(code) || isUnincorporatedAdminCode(code);
  }

  /// AdminAccountKind 链上枚举值：0 公权 / 1 私权 / 2 个人。
  static int adminAccountKind(String code) {
    if (isPublicAdminCode(code)) return 0;
    if (isPrivateAdminCode(code)) return 1;
    if (isPersonal(code)) return 2;
    throw ArgumentError('无法按机构码选择管理员类型: $code');
  }

  /// `AdminAccounts` 所属 runtime pallet 名。
  static String adminAccountsPalletName(String code) {
    if (isPublicAdminCode(code)) return 'PublicAdmins';
    if (isPrivateAdminCode(code)) return 'PrivateAdmins';
    if (isPersonal(code)) return 'PersonalAdmins';
    throw ArgumentError('无法按机构码选择管理员模块: $code');
  }

  /// 按链上 AdminAccountKind 选择管理员模块。
  ///
  /// 非法人机构的机构码不能决定 public/private,但链上 AdminAccount.kind
  /// 已经携带了最终模块归属。涉及已注册账户的读取/提交应优先用 kind 路由。
  static String adminAccountsPalletNameForKind(int kind) {
    return switch (kind) {
      0 => 'PublicAdmins',
      1 => 'PrivateAdmins',
      2 => 'PersonalAdmins',
      _ => throw ArgumentError('无法按管理员类型选择管理员模块: $kind'),
    };
  }

  /// 机构码人机展示标签：固定治理档/个人多签特化为中文名，其余返回码字符串本身。
  static String codeLabel(String code) {
    switch (code) {
      case 'NRC':
        return '国家储委会';
      case 'PRC':
        return '省储委会';
      case 'PRB':
        return '省储行';
      case 'FRG':
        return '联邦注册局';
      case 'NJD':
        return '国家司法院';
      case 'PMUL':
        return '个人多签';
      default:
        return code;
    }
  }
}
