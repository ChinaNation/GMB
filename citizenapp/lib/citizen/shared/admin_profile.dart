import 'dart:convert';
import 'dart:typed_data';

/// 管理员资料来源(序号与链端 `admin-primitives::AdminSource` 严格一致)。
enum AdminProfileSource {
  genesis, // 0
  registry, // 1
  internalVote, // 2
  mutualElection, // 3
  popularElection, // 4
  unknown,
}

extension AdminProfileSourceLabel on AdminProfileSource {
  String get label => switch (this) {
        AdminProfileSource.genesis => '创世',
        AdminProfileSource.registry => '注册局',
        AdminProfileSource.internalVote => '内部投票',
        AdminProfileSource.mutualElection => '互选',
        AdminProfileSource.popularElection => '普选',
        AdminProfileSource.unknown => '未知',
      };
}

/// 机构管理员链上公开资料,镜像链端 `admin-primitives::AdminProfile`。
///
/// 中文注释:`account` 是密码学账户;`cidNumber` 是注册局签发、与真人一一绑定的实名锚;
/// 姓名/职务/任期供 CitizenApp 跨机构展示。**个人多签(PersonalAdmins,kind=3)只有 account**,
/// 其余字段空(链端 personal-admins 存裸 `Vec<AccountId>`,无 AdminProfile)。
class AdminProfile {
  const AdminProfile({
    required this.account,
    this.cidNumber = '',
    this.name = '',
    this.adminRole = '',
    this.termStartDay = 0,
    this.termEndDay = 0,
    this.source = AdminProfileSource.unknown,
  });

  /// 从持久化缓存 JSON 还原(键缩写见 [toJson])。
  factory AdminProfile.fromJson(Map<String, dynamic> j) => AdminProfile(
        account: (j['account'] ?? '').toString(),
        cidNumber: (j['cid'] ?? '').toString(),
        name: (j['name'] ?? '').toString(),
        adminRole: (j['admin_role'] ?? '').toString(),
        termStartDay: (j['ts'] as num?)?.toInt() ?? 0,
        termEndDay: (j['te'] as num?)?.toInt() ?? 0,
        source: _sourceFromByte((j['src'] as num?)?.toInt() ?? 5),
      );

  /// 管理员账户(小写 hex,不含 0x)。
  final String account;

  /// 实名锚:注册局签发的 CID 号(UTF-8)。
  final String cidNumber;

  /// 姓名快照(来自注册局-公民列表)。
  final String name;

  /// 对外法定职务。
  final String adminRole;

  /// 任期开始(天数自纪元;0=无任期)。
  final int termStartDay;

  /// 任期结束(天数自纪元;0=无任期)。
  final int termEndDay;

  /// 职务/任期来源。
  final AdminProfileSource source;

  /// 是否带实名资料(用于 UI 区分"实名管理员" vs 仅账户的个人多签/创世空 meta)。
  bool get hasIdentity =>
      cidNumber.isNotEmpty || name.isNotEmpty || adminRole.isNotEmpty;

  /// 持久化缓存序列化(键缩写省空间;source 存枚举序号)。
  Map<String, Object?> toJson() => {
        'account': account,
        'cid': cidNumber,
        'name': name,
        'admin_role': adminRole,
        'ts': termStartDay,
        'te': termEndDay,
        'src': source.index,
      };

  /// 任期展示文案(无任期→空串)。天数自纪元 → `yyyy-MM-dd`。
  String get termLabel {
    if (termStartDay == 0 && termEndDay == 0) return '';
    final start = termStartDay == 0 ? '—' : _formatDay(termStartDay);
    final end = termEndDay == 0 ? '—' : _formatDay(termEndDay);
    return '$start ~ $end';
  }

  static String _formatDay(int day) {
    final d = DateTime.fromMillisecondsSinceEpoch(
      day * 86400 * 1000,
      isUtc: true,
    );
    final mm = d.month.toString().padLeft(2, '0');
    final dd = d.day.toString().padLeft(2, '0');
    return '${d.year}-$mm-$dd';
  }

  static AdminProfileSource _sourceFromByte(int b) => switch (b) {
        0 => AdminProfileSource.genesis,
        1 => AdminProfileSource.registry,
        2 => AdminProfileSource.internalVote,
        3 => AdminProfileSource.mutualElection,
        4 => AdminProfileSource.popularElection,
        _ => AdminProfileSource.unknown,
      };

  /// 解码 `AdminAccount.admins` 向量(调用前已读完 `Compact<count>`,`offset` 指向首元素)。
  ///
  /// 逐字节对齐链端 `admin-primitives`:
  /// - `isPersonal`(kind==PersonalMultisig=3):每项裸 `AccountId[32]`(无资料);
  /// - 否则每项 `AdminProfile` = `account[32]` + `admin_cid_number`(Compact<len>+UTF8)
  ///   + `name`(Compact) + `admin_role`(Compact) + `term_start`(u32 LE) + `term_end`(u32 LE)
  ///   + `source`(u8)。
  ///
  /// 字节越界返回 null(让调用方按"解码失败"处理)。
  static (List<AdminProfile>, int)? decodeAdminsVec(
    Uint8List data,
    int offset,
    int count, {
    required bool isPersonal,
  }) {
    final out = <AdminProfile>[];
    for (var i = 0; i < count; i++) {
      if (offset + 32 > data.length) return null;
      final account = _hex(data.sublist(offset, offset + 32));
      offset += 32;
      if (isPersonal) {
        out.add(AdminProfile(account: account));
        continue;
      }
      final cid = _readCompactBytes(data, offset);
      if (cid == null) return null;
      offset = cid.$2;
      final name = _readCompactBytes(data, offset);
      if (name == null) return null;
      offset = name.$2;
      final adminRole = _readCompactBytes(data, offset);
      if (adminRole == null) return null;
      offset = adminRole.$2;
      if (offset + 4 + 4 + 1 > data.length) return null;
      final termStart = _readU32(data, offset);
      offset += 4;
      final termEnd = _readU32(data, offset);
      offset += 4;
      final source = _sourceFromByte(data[offset]);
      offset += 1;
      out.add(AdminProfile(
        account: account,
        cidNumber: utf8.decode(cid.$1, allowMalformed: true),
        name: utf8.decode(name.$1, allowMalformed: true),
        adminRole: utf8.decode(adminRole.$1, allowMalformed: true),
        termStartDay: termStart,
        termEndDay: termEnd,
        source: source,
      ));
    }
    return (out, offset);
  }

  // ──── 内部解码工具 ────

  static String _hex(Uint8List bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static int _readU32(Uint8List data, int offset) =>
      ByteData.sublistView(data).getUint32(offset, Endian.little);

  /// 读 `Compact<u32>` 长度前缀 + 该长度的字节;返回 (bytes, nextOffset),越界返回 null。
  static (Uint8List, int)? _readCompactBytes(Uint8List data, int offset) {
    final lenRead = _readCompactU32(data, offset);
    if (lenRead == null) return null;
    final (len, lenSize) = lenRead;
    final start = offset + lenSize;
    final end = start + len;
    if (end > data.length) return null;
    return (Uint8List.sublistView(data, start, end), end);
  }

  static (int, int)? _readCompactU32(Uint8List data, int offset) {
    if (offset >= data.length) return null;
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1) {
      if (offset + 2 > data.length) return null;
      return (((data[offset + 1] << 8) | first) >> 2, 2);
    }
    if (mode == 2) {
      if (offset + 4 > data.length) return null;
      final raw = data[offset] |
          (data[offset + 1] << 8) |
          (data[offset + 2] << 16) |
          (data[offset + 3] << 24);
      return (raw >> 2, 4);
    }
    final len = (first >> 2) + 4;
    if (offset + 1 + len > data.length) return null;
    var value = 0;
    for (var i = 0; i < len && i < 8; i++) {
      value |= data[offset + 1 + i] << (8 * i);
    }
    return (value, len + 1);
  }
}
