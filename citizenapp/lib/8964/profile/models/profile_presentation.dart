/// 用户公开资料缺失时的唯一展示规则。
///
/// 默认昵称、头像和背景只由钱包账户稳定派生，不持久化、不上传，也不参与
/// 身份或权限判断。同一账户在不同页面、设备和重启后得到相同结果。
class ProfilePresentation {
  const ProfilePresentation._({
    required this.ownerAccount,
    required this.fallbackName,
    required this.avatarAsset,
    required this.bannerAsset,
  });

  final String ownerAccount;
  final String fallbackName;
  final String avatarAsset;
  final String bannerAsset;

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

  factory ProfilePresentation.forAccount(String ownerAccount) {
    final account = ownerAccount.trim();
    // 空账户只用于页面尚未加载钱包时的稳定占位，不代表真实用户。
    final seed = account.isEmpty ? 'citizenapp-default-profile' : account;
    final namePrefix = _stableHash(seed, 0x4e414d45) % _namePrefixes.length;
    final nameSuffix = _stableHash(seed, 0x4e49434b) % _nameSuffixes.length;
    final avatarIndex = _stableHash(seed, 0x41564154) % assets.length;
    var bannerIndex = _stableHash(seed, 0x42414e4e) % assets.length;
    if (bannerIndex == avatarIndex) {
      bannerIndex = (bannerIndex + 1) % assets.length;
    }
    return ProfilePresentation._(
      ownerAccount: account,
      fallbackName: '${_namePrefixes[namePrefix]}${_nameSuffixes[nameSuffix]}',
      avatarAsset: assets[avatarIndex],
      bannerAsset: assets[bannerIndex],
    );
  }

  /// 钱包名称是本人昵称真源，Cloudflare `display_name` 是公开镜像；两者都
  /// 缺失时使用稳定默认昵称。任何账户本身或截断账户都不会被接受为昵称。
  String resolveDisplayName({String? walletName, String? publicName}) {
    for (final candidate in <String?>[walletName, publicName]) {
      final normalized = candidate?.trim() ?? '';
      if (normalized.isNotEmpty && !_isAccountDerived(normalized)) {
        return normalized;
      }
    }
    return fallbackName;
  }

  bool _isAccountDerived(String candidate) {
    if (ownerAccount.isEmpty) return false;
    if (candidate == ownerAccount) return true;
    if (ownerAccount.length <= 12) return false;
    final prefix = ownerAccount.substring(0, 6);
    final suffix = ownerAccount.substring(ownerAccount.length - 6);
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
