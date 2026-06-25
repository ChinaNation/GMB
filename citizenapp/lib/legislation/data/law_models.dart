// 立法法律 Dart 镜像类型(ADR-027 / ADR-028 P3)——与链端 legislation-yuan
// SCALE 布局逐字段对齐。解码见 [legislation_codec.dart](单一源)。
//
// 中文注释:法律层级 = 章 > 节 > 条 > 款 > 项。宪法(law_id=0,tier=宪法)双语
// (`*En` 全填),普通法律单语(`*En` 为 null)。

/// 法律层级(链端 Tier,1 字节枚举索引)。
enum LawTier {
  constitution, // 0 宪法
  national, // 1 国家
  provincial, // 2 省
  municipal; // 3 市

  static LawTier fromIndex(int i) => switch (i) {
        0 => LawTier.constitution,
        1 => LawTier.national,
        2 => LawTier.provincial,
        _ => LawTier.municipal,
      };

  int get index0 => index;

  String get label => switch (this) {
        LawTier.constitution => '宪法',
        LawTier.national => '国家',
        LawTier.provincial => '省',
        LawTier.municipal => '市',
      };
}

/// 法律状态(链端 LawStatus,1 字节)。
enum LawStatus {
  pending, // 0 通过待生效
  effective, // 1 生效
  repealed; // 2 废止

  static LawStatus fromIndex(int i) => switch (i) {
        0 => LawStatus.pending,
        1 => LawStatus.effective,
        _ => LawStatus.repealed,
      };

  String get label => switch (this) {
        LawStatus.pending => '待生效',
        LawStatus.effective => '生效中',
        LawStatus.repealed => '已废止',
      };
}

/// 款下的「项」。
class LawClause {
  const LawClause({required this.number, required this.text, this.textEn});
  final int number;
  final String text;
  final String? textEn;
}

/// 条(全法唯一连续编号,用于不可修改条款比对 + 锚点)。
class LawArticle {
  const LawArticle({
    required this.number,
    required this.title,
    required this.body,
    required this.clauses,
    this.titleEn,
    this.bodyEn,
  });
  final int number;
  final String title;
  final String? titleEn;
  final String body;
  final String? bodyEn;
  final List<LawClause> clauses;
}

/// 节。
class LawSection {
  const LawSection({
    required this.number,
    required this.title,
    required this.articles,
    this.titleEn,
  });
  final int number;
  final String title;
  final String? titleEn;
  final List<LawArticle> articles;
}

/// 章。
class LawChapter {
  const LawChapter({
    required this.number,
    required this.title,
    required this.sections,
    this.titleEn,
  });
  final int number;
  final String title;
  final String? titleEn;
  final List<LawSection> sections;
}

/// 立法机构院(机构码 + 主账户 hex);houses[0]=发起院。
class LawHouse {
  const LawHouse({required this.institutionCode, required this.accountHex});
  final String institutionCode;
  final String accountHex;
}

/// 法律主记录(链端 `law(law_id)` 返回)。
class Law {
  const Law({
    required this.lawId,
    required this.tier,
    required this.scopeCode,
    required this.houses,
    required this.currentVersion,
    required this.status,
  });
  final int lawId;
  final LawTier tier;

  /// 0=全国;否则 china.sqlite 行政区 code(ADR-021)。
  final int scopeCode;
  final List<LawHouse> houses;
  final int currentVersion;
  final LawStatus status;
}

/// 法律某版本正文(链端 `law_version(law_id, version)` 返回)。
class LawVersion {
  const LawVersion({
    required this.lawId,
    required this.version,
    required this.title,
    required this.chapters,
    required this.contentHash,
    required this.voteType,
    required this.proposalId,
    required this.publishedAt,
    required this.effectiveAt,
    this.titleEn,
  });
  final int lawId;
  final int version;
  final String title;
  final String? titleEn;
  final List<LawChapter> chapters;

  /// blake2_256(SCALE chapters) hex(公投/签名绑定)。
  final String contentHash;
  final int voteType;

  /// 创世宪法为 0。
  final int proposalId;

  /// 块号(经 target_block_time_ms 换算日期)。
  final int publishedAt;
  final int effectiveAt;
}

/// 宪法不可修改条款 manifest(链端 ConstitutionImmutableManifest)。
class ImmutableManifest {
  const ImmutableManifest({
    required this.articleNumbers,
    required this.articleHashes,
  });

  /// 不可修改的条号集合(链端固定 [1,2,3,17,19,23,33,41])。
  final List<int> articleNumbers;

  /// 与 [articleNumbers] 平行的条文 blake2_256 hex 数组。
  final List<String> articleHashes;

  bool isImmutable(int articleNumber) => articleNumbers.contains(articleNumber);
}
