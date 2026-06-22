import 'dart:convert';
import 'dart:typed_data';

/// 机构码(CID institution_code)冷钱包端表示与治理分类 = institution_code.dart
///
/// 中文注释(铁律):
/// 本文件逐字镜像链端 `primitives::institution_code`(同一套 86 码)。链上治理不再使用
/// 任何独立的 ORG_xx 单字节数字标签;链上已全部替换为 4 字节 `institution_code`([u8;4]
/// 原始码字节,3 字符码右补 `0`)。冷钱包离线解码用本文件的纯函数从机构码派生治理分类
/// (是不是固定治理档 / 个人多签 / 机构账户),绝不另立第二套分类。
///
/// 字节表示:机构码是 3~4 个大写 ASCII 字符,统一用 4 字节,3 字符码右补 `0`:
///   NRC → [78,82,67,0]  CGOV → [67,71,79,86]  PMUL → [80,77,85,76]
class InstitutionCode {
  const InstitutionCode._();

  // ──────────────────────────────────────────────────────────────────
  // 治理相关机构码常量(4 字节,3 字符码末位补 0)
  // ──────────────────────────────────────────────────────────────────

  /// 国家公民储备委员会(固定治理档)。"NRC\0"。
  static const List<int> nrc = [78, 82, 67, 0];

  /// 省公民储备委员会(固定治理档)。"PRC\0"。
  static const List<int> prc = [80, 82, 67, 0];

  /// 省公民储备银行(固定治理档)。"PRB\0"。
  static const List<int> prb = [80, 82, 66, 0];

  /// 个人多签账户(不发号,仅链上/后端分类常量)。"PMUL"。
  static const List<int> pmul = [80, 77, 85, 76];

  // ──────────────────────────────────────────────────────────────────
  // 字节 ↔ 字符串
  // ──────────────────────────────────────────────────────────────────

  /// 取前 4 字节,去掉尾部 0 字节,UTF-8 → 大写字符串。
  static String codeToString(List<int> bytes) {
    var end = bytes.length < 4 ? bytes.length : 4;
    while (end > 0 && bytes[end - 1] == 0) {
      end--;
    }
    if (end == 0) return '';
    return utf8.decode(bytes.sublist(0, end), allowMalformed: true).toUpperCase();
  }

  /// 字符串机构码 → 4 字节(右补 0,超 4 截断)。
  static List<int> codeBytes(String code) {
    final raw = utf8.encode(code.toUpperCase());
    final out = Uint8List(4);
    for (var i = 0; i < 4 && i < raw.length; i++) {
      out[i] = raw[i];
    }
    return out;
  }

  // ──────────────────────────────────────────────────────────────────
  // 机构码分类清单(与链端 PUBLIC/PRIVATE/UNINCORPORATED 同源)
  // ──────────────────────────────────────────────────────────────────

  /// 公权法人机构码(A 国家级 26 + B 省级 17 + C 市级 17 + D 镇级 10 + 公立大学/学校 2)= 72。
  static const Set<String> _publicLegalCodes = <String>{
    // A 国家级单体(26)
    'PRS', 'FSC', 'FIB', 'FSS', 'FPR', 'FRG', 'MFA', 'MDF',
    'MHS', 'MCW', 'MHU', 'MAG', 'MCM', 'MFT', 'MEN', 'MTR',
    'NLG', 'NJD', 'NSP', 'FAC', 'FAU', 'FIV', 'NED', 'NRC',
    'NSN', 'NRP',
    // B 省级类型(17)
    'PGV', 'PLG', 'PJD', 'PSP', 'PRC', 'PRB', 'PDF', 'PHS',
    'PCW', 'PHU', 'PAG', 'PCM', 'PFT', 'PEN', 'PTR', 'PSN',
    'PRP',
    // C 市级类型(17)
    'CGOV', 'CLEG', 'CSUP', 'CJUD', 'CEDU', 'CSLF', 'CDEF', 'CHSC', 'CCWF',
    'CHUD', 'CAGR', 'CCOM', 'CFIN', 'CENR', 'CTRN', 'CREG', 'CPOL',
    // D 镇级类型(10)
    'TGOV', 'TCWF', 'THUD', 'TAGR', 'TFIN', 'TDEF', 'THSC', 'TCOM', 'TENR',
    'TTRN',
    // 公立大学 / 公立学校
    'GUN', 'GSCH',
  };

  /// 私权法人机构码(有限合伙/股权/股份/公益/注册协会 + 私立大学/学校)= 7。
  static const Set<String> _privateLegalCodes = <String>{
    'SFLP', 'SFGQ', 'SFGF', 'SFGY', 'SFAS', 'SUN', 'SFSC',
  };

  /// 非法人机构码(个体经营/无限合伙/非法人组织)= 3。
  static const Set<String> _unincorporatedCodes = <String>{
    'SFGT', 'SFGP', 'UNIN',
  };

  // ──────────────────────────────────────────────────────────────────
  // 治理策略派生(纯函数,冷钱包唯一分类来源)
  // ──────────────────────────────────────────────────────────────────

  /// 是否为固定治理档机构码(国储会/省储会/省储行)。
  static bool isFixedGovernance(String code) {
    return code == 'NRC' || code == 'PRC' || code == 'PRB';
  }

  /// 是否为个人多签账户机构码(PMUL)。
  static bool isPersonal(String code) {
    return code == 'PMUL';
  }

  /// 是否为公权法人机构码。
  static bool isPublicLegal(String code) => _publicLegalCodes.contains(code);

  /// 是否为私权法人机构码。
  static bool isPrivateLegal(String code) => _privateLegalCodes.contains(code);

  /// 是否为机构账户机构码(公权/私权/非法人法人实体,经 organization-manage 注册多签)。
  ///
  /// 取代旧 `org∈{4,5}`(ORG_PUP/ORG_OTH)判定:个人/个人多签不算机构账户;
  /// 固定治理档(NRC/PRC/PRB)是 china 内建创世账户,走固定治理路径,也不算机构账户。
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

  /// 机构码人机展示标签:固定治理档/个人多签特化为中文名,其余返回码字符串本身。
  static String codeLabel(String code) {
    switch (code) {
      case 'NRC':
        return '国储会';
      case 'PRC':
        return '省储会';
      case 'PRB':
        return '省储行';
      case 'PMUL':
        return '个人多签';
      default:
        return code;
    }
  }
}
