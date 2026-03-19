import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

class UserProfileState {
  const UserProfileState({
    this.avatarPath,
    this.backgroundPath,
    this.communicationWalletIndex,
    this.communicationAddress,
    this.communicationWalletName,
  });

  final String? avatarPath;
  final String? backgroundPath;
  /// 通信账户钱包 index（对应 WalletProfile.walletIndex）。
  final int? communicationWalletIndex;
  /// 通信账户钱包地址（SS58）。
  final String? communicationAddress;
  /// 通信账户钱包名称（= 用户昵称）。
  final String? communicationWalletName;

  /// 用户昵称 = 通信钱包名称，未设置时显示默认值。
  String get nickname =>
      communicationWalletName?.trim().isNotEmpty == true
          ? communicationWalletName!
          : UserProfileService.defaultNickname;

  UserProfileState copyWith({
    Object? avatarPath = _sentinel,
    Object? backgroundPath = _sentinel,
    Object? communicationWalletIndex = _sentinel,
    Object? communicationAddress = _sentinel,
    Object? communicationWalletName = _sentinel,
  }) {
    return UserProfileState(
      avatarPath: identical(avatarPath, _sentinel)
          ? this.avatarPath
          : avatarPath as String?,
      backgroundPath: identical(backgroundPath, _sentinel)
          ? this.backgroundPath
          : backgroundPath as String?,
      communicationWalletIndex: identical(communicationWalletIndex, _sentinel)
          ? this.communicationWalletIndex
          : communicationWalletIndex as int?,
      communicationAddress: identical(communicationAddress, _sentinel)
          ? this.communicationAddress
          : communicationAddress as String?,
      communicationWalletName: identical(communicationWalletName, _sentinel)
          ? this.communicationWalletName
          : communicationWalletName as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'avatar_path': avatarPath,
      'background_path': backgroundPath,
      'communication_wallet_index': communicationWalletIndex,
      'communication_address': communicationAddress,
      'communication_wallet_name': communicationWalletName,
    };
  }

  static UserProfileState fromJson(Map<String, dynamic> json) {
    return UserProfileState(
      avatarPath: _normalizeOptionalString(json['avatar_path']),
      backgroundPath: _normalizeOptionalString(json['background_path']),
      communicationWalletIndex: json['communication_wallet_index'] as int?,
      communicationAddress:
          _normalizeOptionalString(json['communication_address']),
      communicationWalletName:
          _normalizeOptionalString(json['communication_wallet_name']),
    );
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
    final account = (json['account_pubkey']?.toString().trim()) ?? '';
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
      return const UserProfileState();
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        throw const FormatException('profile payload invalid');
      }
      return UserProfileState.fromJson(decoded);
    } catch (_) {
      return const UserProfileState();
    }
  }

  Future<UserProfileState> saveState(UserProfileState state) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kProfile, jsonEncode(state.toJson()));
    return state;
  }

  Future<UserProfileState> updateAvatarPath(String? avatarPath) async {
    final current = await getState();
    return saveState(current.copyWith(avatarPath: avatarPath));
  }

  Future<UserProfileState> updateBackgroundPath(String? backgroundPath) async {
    final current = await getState();
    return saveState(current.copyWith(backgroundPath: backgroundPath));
  }

  /// 设置通信账户（钱包 index + 地址 + 名称）。
  Future<UserProfileState> setCommunicationWallet({
    required int walletIndex,
    required String address,
    required String walletName,
  }) async {
    final current = await getState();
    return saveState(current.copyWith(
      communicationWalletIndex: walletIndex,
      communicationAddress: address,
      communicationWalletName: walletName,
    ));
  }

  /// 更新通信账户钱包名称（双向同步用）。
  Future<UserProfileState> updateCommunicationWalletName(
      String walletName) async {
    final current = await getState();
    if (current.communicationWalletIndex == null) return current;
    return saveState(
        current.copyWith(communicationWalletName: walletName));
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

  /// 添加或更新通讯录联系人。
  Future<ContactImportResult> addContact({
    required String address,
    required String name,
    String? selfAddress,
  }) async {
    final incomingAccount = address.trim();
    if (incomingAccount.isEmpty || name.trim().isEmpty) {
      throw const FormatException('地址或名称为空');
    }
    final self = selfAddress?.trim() ?? '';
    if (self.isNotEmpty && incomingAccount == self) {
      throw const FormatException('不能把自己加入通讯录');
    }

    final contacts = (await getContacts()).toList(growable: true);
    final now = DateTime.now().millisecondsSinceEpoch;
    final index =
        contacts.indexWhere((item) => item.accountPubkeyHex == incomingAccount);
    if (index >= 0) {
      final updated = contacts[index].copyWith(
        sourceNickname: name.trim(),
        updatedAtMillis: now,
      );
      contacts[index] = updated;
      await _saveContacts(contacts);
      return ContactImportResult(contact: updated, created: false);
    }

    final created = UserContact(
      accountPubkeyHex: incomingAccount,
      sourceNickname: name.trim(),
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
