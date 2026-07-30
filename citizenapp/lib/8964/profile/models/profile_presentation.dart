/// 用户公开资料缺失时的唯一展示规则。
///
/// 默认昵称、头像和背景由永久 CID 稳定派生；无 CID 的访客才用当前账户兜底。
/// 这些展示值不持久化、不上传，也不参与身份或权限判断。
class ProfilePresentation {
  const ProfilePresentation._({
    required this.identityKey,
    required this.fallbackName,
    required this.avatarAsset,
    required this.bannerAsset,
  });

  /// 永久 CID；仅未注册访客允许传当前 `account_id` 作为稳定兜底。
  final String identityKey;
  final String fallbackName;
  final String avatarAsset;
  final String bannerAsset;

  static final RegExp _accountIdPattern = RegExp(r'^0x[0-9a-f]{64}$');
  static final RegExp _ss58Pattern = RegExp(r'^[1-9A-HJ-NP-Za-km-z]{40,64}$');

  static const List<String> _namePrefixes = <String>[
    '晨光',
    '青松',
    '星河',
    '云海',
    '远山',
    '清风',
    '春雨',
    '秋叶',
    '白露',
    '暖阳',
    '碧海',
    '长空',
    '新月',
    '流云',
    '萤火',
    '曙光',
    '南风',
    '北辰',
    '夏木',
    '冬雪',
  ];

  static const List<String> _nameSuffixes = <String>[
    '旅人',
    '行者',
    '朋友',
    '伙伴',
    '邻居',
    '来客',
    '信使',
    '守望者',
    '探索者',
    '漫游者',
    '远客',
    '听风者',
    '逐光者',
    '观星者',
    '寻路者',
    '拾光者',
    '筑梦者',
    '摆渡人',
    '追云者',
    '望山人',
  ];

  /// 用户指定的本地图片池；头像和背景共用资源，但按不同盐值选图。
  static const List<String> assets = <String>[
    'assets/profile_defaults/xiserge-silver-gull-7787328_1280.jpg',
    'assets/profile_defaults/paul_reuss-patagonia-10020972_1280.jpg',
    'assets/profile_defaults/anneef-meadow-10272037_1280.jpg',
    'assets/profile_defaults/arweltatty-lonely-tree-10293271_1280.jpg',
    'assets/profile_defaults/ahmetyuksek-prague-10204909_1280.jpg',
    'assets/profile_defaults/bluestone-canadian-rockies-9855618_1280.jpg',
    'assets/profile_defaults/wal_172619-ferris-wheel-10340490_1280.jpg',
    'assets/profile_defaults/nunziog666-lake-10349715_1280.jpg',
    'assets/profile_defaults/wj_y2017fufu-mountain-9784922_1280.jpg',
    'assets/profile_defaults/nunziog666-banff-10349707_1920.jpg',
    'assets/profile_defaults/couleur-tomatoes-10368988_1280.jpg',
  ];

  factory ProfilePresentation.forIdentityKey(String identityKey) {
    identityKey = identityKey.trim();
    // 空身份键只用于页面尚未完成加载时的稳定占位，不代表真实用户。
    final seed =
        identityKey.isEmpty ? 'citizenapp-default-profile' : identityKey;
    final namePrefix = _stableHash(seed, 0x4e414d45) % _namePrefixes.length;
    final nameSuffix = _stableHash(seed, 0x4e49434b) % _nameSuffixes.length;
    final avatarIndex = _stableHash(seed, 0x41564154) % assets.length;
    var bannerIndex = _stableHash(seed, 0x42414e4e) % assets.length;
    if (bannerIndex == avatarIndex) {
      bannerIndex = (bannerIndex + 1) % assets.length;
    }
    return ProfilePresentation._(
      identityKey: identityKey,
      fallbackName: '${_namePrefixes[namePrefix]}${_nameSuffixes[nameSuffix]}',
      avatarAsset: assets[avatarIndex],
      bannerAsset: assets[bannerIndex],
    );
  }

  /// 公开昵称唯一真源是 `display_name`；缺失时使用稳定默认昵称。
  ///
  /// 本机钱包名不得传入本方法。任何账户本身或截断账户也不会被接受为昵称。
  String resolveDisplayName({String? publicName}) {
    final normalized = publicName?.trim() ?? '';
    if (normalized.isNotEmpty && !_isIdentityOrAccountDerived(normalized)) {
      return normalized;
    }
    return fallbackName;
  }

  bool _isIdentityOrAccountDerived(String candidate) {
    if (_accountIdPattern.hasMatch(candidate) ||
        _ss58Pattern.hasMatch(candidate)) {
      return true;
    }
    if (identityKey.isEmpty) return false;
    if (candidate == identityKey) return true;
    if (identityKey.length <= 12) return false;
    final prefix = identityKey.substring(0, 6);
    final suffix = identityKey.substring(identityKey.length - 6);
    return candidate == '$prefix...$suffix' || candidate == '$prefix…$suffix';
  }

  /// FNV-1a 32 位哈希只用于稳定分桶，不承担密码学安全职责。
  static int _stableHash(String value, int salt) {
    var hash = (0x811c9dc5 ^ salt) & 0xffffffff;
    for (final unit in value.codeUnits) {
      hash ^= unit;
      hash = (hash * 0x01000193) & 0xffffffff;
    }
    return hash & 0x7fffffff;
  }
}
