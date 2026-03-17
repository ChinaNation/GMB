import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

class UserProfileState {
  const UserProfileState({
    required this.nickname,
    required this.nicknameCustomized,
    this.avatarPath,
    this.backgroundPath,
    this.communicationAddress,
  });

  final String nickname;
  final bool nicknameCustomized;
  final String? avatarPath;
  final String? backgroundPath;
  final String? communicationAddress;

  UserProfileState copyWith({
    String? nickname,
    bool? nicknameCustomized,
    Object? avatarPath = _sentinel,
    Object? backgroundPath = _sentinel,
    Object? communicationAddress = _sentinel,
  }) {
    return UserProfileState(
      nickname: nickname ?? this.nickname,
      nicknameCustomized: nicknameCustomized ?? this.nicknameCustomized,
      avatarPath: identical(avatarPath, _sentinel)
          ? this.avatarPath
          : avatarPath as String?,
      backgroundPath: identical(backgroundPath, _sentinel)
          ? this.backgroundPath
          : backgroundPath as String?,
      communicationAddress: identical(communicationAddress, _sentinel)
          ? this.communicationAddress
          : communicationAddress as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'nickname': nickname,
      'nickname_customized': nicknameCustomized,
      'avatar_path': avatarPath,
      'background_path': backgroundPath,
      'communication_address': communicationAddress,
    };
  }

  static UserProfileState fromJson(Map<String, dynamic> json) {
    final rawNickname = json['nickname']?.toString().trim() ?? '';
    return UserProfileState(
      nickname: rawNickname.isEmpty
          ? UserProfileService.defaultNickname
          : rawNickname,
      nicknameCustomized: json['nickname_customized'] as bool? ?? false,
      avatarPath: _normalizeOptionalString(json['avatar_path']),
      backgroundPath: _normalizeOptionalString(json['background_path']),
      communicationAddress:
          _normalizeOptionalString(json['communication_address']),
    );
  }
}

class UserQrPayload {
  const UserQrPayload({
    required this.nickname,
    required this.address,
  });

  static const String protocol = 'WUMINAPP_CONTACT_V1';
  static const String legacyProtocol = 'WUMINAPP_USER_CARD_V1';

  final String nickname;
  final String address;

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'proto': protocol,
      'address': address,
      'name': nickname,
    };
  }

  String toRawJson() => jsonEncode(toJson());

  static UserQrPayload parse(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('二维码数据格式错误');
    }

    final proto = (decoded['proto'] ?? decoded['type'] ?? '').toString();

    // 兼容旧版 WUMINAPP_USER_CARD_V1 格式。
    if (proto == legacyProtocol) {
      final nickname = decoded['nickname']?.toString().trim() ?? '';
      final accountPubkeyHex = UserContactService.normalizePubkeyHex(
          decoded['account_pubkey']?.toString() ?? '');
      if (nickname.isEmpty || accountPubkeyHex.isEmpty) {
        throw const FormatException('二维码缺少昵称或账号信息');
      }
      return UserQrPayload(nickname: nickname, address: accountPubkeyHex);
    }

    // 新版 WUMINAPP_CONTACT_V1 格式。
    if (proto != protocol) {
      throw const FormatException('不是用户通讯录二维码');
    }

    final name = decoded['name']?.toString().trim() ?? '';
    final address = decoded['address']?.toString().trim() ?? '';
    if (name.isEmpty || address.isEmpty) {
      throw const FormatException('二维码缺少昵称或地址信息');
    }

    return UserQrPayload(nickname: name, address: address);
  }
}

class UserContact {
  const UserContact({
    required this.accountPubkeyHex,
    required this.sourceNickname,
    required this.addedAtMillis,
    required this.updatedAtMillis,
    this.localNickname,
  });

  final String accountPubkeyHex;
  final String sourceNickname;
  final int addedAtMillis;
  final int updatedAtMillis;
  final String? localNickname;

  String get displayNickname {
    final local = localNickname?.trim() ?? '';
    if (local.isNotEmpty) {
      return local;
    }
    return sourceNickname;
  }

  UserContact copyWith({
    String? accountPubkeyHex,
    String? sourceNickname,
    int? addedAtMillis,
    int? updatedAtMillis,
    Object? localNickname = _sentinel,
  }) {
    return UserContact(
      accountPubkeyHex: accountPubkeyHex ?? this.accountPubkeyHex,
      sourceNickname: sourceNickname ?? this.sourceNickname,
      addedAtMillis: addedAtMillis ?? this.addedAtMillis,
      updatedAtMillis: updatedAtMillis ?? this.updatedAtMillis,
      localNickname: identical(localNickname, _sentinel)
          ? this.localNickname
          : localNickname as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'account_pubkey': accountPubkeyHex,
      'source_nickname': sourceNickname,
      'local_nickname': localNickname,
      'added_at': addedAtMillis,
      'updated_at': updatedAtMillis,
    };
  }

  static UserContact fromJson(Map<String, dynamic> json) {
    final account = UserContactService.normalizePubkeyHex(
      json['account_pubkey']?.toString() ?? '',
    );
    if (account.isEmpty) {
      throw const FormatException('通讯录账号为空');
    }

    final sourceNickname = json['source_nickname']?.toString().trim() ?? '';
    if (sourceNickname.isEmpty) {
      throw const FormatException('通讯录昵称为空');
    }

    return UserContact(
      accountPubkeyHex: account,
      sourceNickname: sourceNickname,
      localNickname: _normalizeOptionalString(json['local_nickname']),
      addedAtMillis: _normalizeInt(json['added_at']),
      updatedAtMillis: _normalizeInt(json['updated_at']),
    );
  }
}

class ContactImportResult {
  const ContactImportResult({
    required this.contact,
    required this.created,
  });

  final UserContact contact;
  final bool created;
}

class UserProfileService {
  static const String defaultNickname = '轻节点';
  static const String _kProfile = 'user.profile.state.v2';

  Future<UserProfileState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kProfile);
    if (raw == null || raw.trim().isEmpty) {
      return const UserProfileState(
        nickname: defaultNickname,
        nicknameCustomized: false,
      );
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        throw const FormatException('profile payload invalid');
      }
      return UserProfileState.fromJson(decoded);
    } catch (_) {
      return const UserProfileState(
        nickname: defaultNickname,
        nicknameCustomized: false,
      );
    }
  }

  Future<UserProfileState> saveState(UserProfileState state) async {
    final sanitized = UserProfileState(
      nickname: _normalizeNickname(state.nickname),
      nicknameCustomized: state.nicknameCustomized,
      avatarPath: _normalizeOptionalString(state.avatarPath),
      backgroundPath: _normalizeOptionalString(state.backgroundPath),
    );
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kProfile, jsonEncode(sanitized.toJson()));
    return sanitized;
  }

  Future<UserProfileState> updateNickname(String nickname) async {
    final current = await getState();
    return saveState(
      current.copyWith(
        nickname: _normalizeNickname(nickname),
        nicknameCustomized: true,
      ),
    );
  }

  Future<UserProfileState> updateAvatarPath(String? avatarPath) async {
    final current = await getState();
    return saveState(current.copyWith(avatarPath: avatarPath));
  }

  Future<UserProfileState> updateBackgroundPath(String? backgroundPath) async {
    final current = await getState();
    return saveState(current.copyWith(backgroundPath: backgroundPath));
  }

  Future<UserProfileState> updateCommunicationAddress(
      String? address) async {
    final current = await getState();
    return saveState(current.copyWith(communicationAddress: address));
  }

  bool isNicknameReady(UserProfileState state) {
    return state.nicknameCustomized && state.nickname.trim().isNotEmpty;
  }

  String _normalizeNickname(String nickname) {
    final normalized = nickname.trim();
    if (normalized.isEmpty) {
      return defaultNickname;
    }
    return normalized;
  }
}

class UserContactService {
  static const String _kContacts = 'user.contacts.items.v1';

  Future<List<UserContact>> getContacts() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kContacts);
    if (raw == null || raw.trim().isEmpty) {
      return const <UserContact>[];
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! List) {
        throw const FormatException('contacts payload invalid');
      }
      final contacts = decoded
          .whereType<Map<String, dynamic>>()
          .map(UserContact.fromJson)
          .toList(growable: true)
        ..sort((a, b) => b.updatedAtMillis.compareTo(a.updatedAtMillis));
      return contacts;
    } catch (_) {
      return const <UserContact>[];
    }
  }

  Future<ContactImportResult> addFromQrPayload(
    String rawPayload, {
    String? selfAccountPubkeyHex,
  }) async {
    final payload = UserQrPayload.parse(rawPayload);
    final incomingAccount = payload.address.trim();
    final selfAccount = normalizePubkeyHex(selfAccountPubkeyHex ?? '');
    if (selfAccount.isNotEmpty && incomingAccount == selfAccount) {
      throw const FormatException('不能把自己加入通讯录');
    }

    final contacts = (await getContacts()).toList(growable: true);
    final now = DateTime.now().millisecondsSinceEpoch;
    final index =
        contacts.indexWhere((item) => item.accountPubkeyHex == incomingAccount);
    if (index >= 0) {
      final updated = contacts[index].copyWith(
        sourceNickname: payload.nickname,
        updatedAtMillis: now,
      );
      contacts[index] = updated;
      await _saveContacts(contacts);
      return ContactImportResult(contact: updated, created: false);
    }

    final created = UserContact(
      accountPubkeyHex: incomingAccount,
      sourceNickname: payload.nickname,
      addedAtMillis: now,
      updatedAtMillis: now,
    );
    contacts.add(created);
    await _saveContacts(contacts);
    return ContactImportResult(contact: created, created: true);
  }

  Future<List<UserContact>> renameContact(
    String accountPubkeyHex,
    String localNickname,
  ) async {
    final normalizedAccount = normalizePubkeyHex(accountPubkeyHex);
    final contacts = (await getContacts()).toList(growable: true);
    final index = contacts
        .indexWhere((item) => item.accountPubkeyHex == normalizedAccount);
    if (index < 0) {
      throw Exception('未找到联系人');
    }

    final normalizedNickname = localNickname.trim();
    contacts[index] = contacts[index].copyWith(
      localNickname: normalizedNickname.isEmpty ? null : normalizedNickname,
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
    );
    await _saveContacts(contacts);
    return contacts
      ..sort((a, b) => b.updatedAtMillis.compareTo(a.updatedAtMillis));
  }

  Future<void> _saveContacts(List<UserContact> contacts) async {
    final prefs = await SharedPreferences.getInstance();
    final payload =
        contacts.map((item) => item.toJson()).toList(growable: false);
    await prefs.setString(_kContacts, jsonEncode(payload));
  }

  static String normalizePubkeyHex(String value) {
    var normalized = value.trim().toLowerCase();
    if (normalized.isEmpty) {
      return '';
    }
    if (!normalized.startsWith('0x')) {
      normalized = '0x$normalized';
    }
    final body = normalized.substring(2);
    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(body)) {
      throw const FormatException('公钥格式错误');
    }
    return normalized;
  }
}

const Object _sentinel = Object();

String? _normalizeOptionalString(Object? value) {
  final normalized = value?.toString().trim() ?? '';
  if (normalized.isEmpty) {
    return null;
  }
  return normalized;
}

int _normalizeInt(Object? value) {
  return switch (value) {
    int v => v,
    num v => v.toInt(),
    String v => int.tryParse(v) ?? 0,
    _ => 0,
  };
}
