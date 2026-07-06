import 'dart:convert';

import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:shared_preferences/shared_preferences.dart';

class UserProfileState {
  const UserProfileState({
    this.avatarPath,
    this.backgroundPath,
  });

  final String? avatarPath;
  final String? backgroundPath;

  UserProfileState copyWith({
    Object? avatarPath = _sentinel,
    Object? backgroundPath = _sentinel,
  }) {
    return UserProfileState(
      avatarPath: identical(avatarPath, _sentinel)
          ? this.avatarPath
          : avatarPath as String?,
      backgroundPath: identical(backgroundPath, _sentinel)
          ? this.backgroundPath
          : backgroundPath as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'avatar_path': avatarPath,
      'background_path': backgroundPath,
    };
  }

  static UserProfileState fromJson(Map<String, dynamic> json) {
    return UserProfileState(
      avatarPath: _normalizeOptionalString(json['avatar_path']),
      backgroundPath: _normalizeOptionalString(json['background_path']),
    );
  }
}

class UserContact {
  const UserContact({
    required this.address,
    required this.sourceNickname,
    required this.addedAtMillis,
    required this.updatedAtMillis,
    this.localNickname,
  });

  /// 联系人链上地址。通讯录/二维码/转账输入边界统一使用 SS58。
  final String address;
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
    String? address,
    String? sourceNickname,
    int? addedAtMillis,
    int? updatedAtMillis,
    Object? localNickname = _sentinel,
  }) {
    return UserContact(
      address: address ?? this.address,
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
      'address': address,
      'source_nickname': sourceNickname,
      'local_nickname': localNickname,
      'added_at': addedAtMillis,
      'updated_at': updatedAtMillis,
    };
  }

  static UserContact fromJson(Map<String, dynamic> json) {
    final address = (json['address']?.toString().trim()) ?? '';
    if (address.isEmpty) {
      throw const FormatException('通讯录地址为空');
    }

    final sourceNickname = json['source_nickname']?.toString().trim() ?? '';
    if (sourceNickname.isEmpty) {
      throw const FormatException('通讯录昵称为空');
    }

    return UserContact(
      address: address,
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
}

class UserContactService {
  static const String _kContacts = 'user.contacts.items.v2';
  static const int _ss58Prefix = 2027;

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
    required String contactName,
    String? selfAddress,
  }) async {
    final incomingAddress = normalizeAddress(address);
    final normalizedContactName = contactName.trim();
    if (incomingAddress.isEmpty || normalizedContactName.isEmpty) {
      throw const FormatException('地址或名称为空');
    }
    final self = selfAddress?.trim() ?? '';
    if (self.isNotEmpty && incomingAddress == normalizeAddress(self)) {
      throw const FormatException('不能把自己加入通讯录');
    }

    final contacts = (await getContacts()).toList(growable: true);
    final now = DateTime.now().millisecondsSinceEpoch;
    final index =
        contacts.indexWhere((item) => item.address == incomingAddress);
    if (index >= 0) {
      final updated = contacts[index].copyWith(
        sourceNickname: normalizedContactName,
        updatedAtMillis: now,
      );
      contacts[index] = updated;
      await _saveContacts(contacts);
      return ContactImportResult(contact: updated, created: false);
    }

    final created = UserContact(
      address: incomingAddress,
      sourceNickname: normalizedContactName,
      addedAtMillis: now,
      updatedAtMillis: now,
    );
    contacts.add(created);
    await _saveContacts(contacts);
    return ContactImportResult(contact: created, created: true);
  }

  Future<List<UserContact>> renameContact(
    String address,
    String localNickname,
  ) async {
    final normalizedAddress = normalizeAddress(address);
    final contacts = (await getContacts()).toList(growable: true);
    final index =
        contacts.indexWhere((item) => item.address == normalizedAddress);
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

  static String normalizeAddress(String value) {
    final normalized = value.trim();
    if (normalized.isEmpty) {
      return '';
    }
    try {
      final decoded = Keyring().decodeAddress(normalized);
      final reEncoded = Keyring().encodeAddress(decoded, _ss58Prefix);
      if (reEncoded != normalized) {
        throw const FormatException('联系人地址不是本链 SS58 地址');
      }
      return normalized;
    } catch (_) {
      throw const FormatException('联系人地址不是本链 SS58 地址');
    }
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
