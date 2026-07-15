import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 通讯录唯一业务模型。公开昵称、头像和签名属于用户公开资料，不复制进通讯录；
/// 本模型只保存联系人账户与当前用户自己的私人联系人名称。
class UserContact {
  const UserContact({
    required this.address,
    required this.contactName,
    required this.createdAt,
    required this.updatedAt,
  });

  final String address;
  final String contactName;
  final int createdAt;
  final int updatedAt;

  UserContact copyWith({
    String? address,
    String? contactName,
    int? createdAt,
    int? updatedAt,
  }) {
    return UserContact(
      address: address ?? this.address,
      contactName: contactName ?? this.contactName,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
        'address': address,
        'contact_name': contactName,
        'created_at': createdAt,
        'updated_at': updatedAt,
      };

  factory UserContact.fromJson(Map<String, dynamic> json) {
    final address = json['address']?.toString().trim() ?? '';
    final contactName = json['contact_name']?.toString().trim() ?? '';
    if (address.isEmpty || contactName.isEmpty) {
      throw const FormatException('通讯录地址或联系人名称为空');
    }
    final createdAt = _asInt(json['created_at']);
    final updatedAt = _asInt(json['updated_at']);
    if (createdAt <= 0 || updatedAt <= 0) {
      throw const FormatException('通讯录时间戳不合法');
    }
    return UserContact(
      address: UserContactService.normalizeAddress(address),
      contactName: contactName,
      createdAt: createdAt,
      updatedAt: updatedAt,
    );
  }
}

class ContactImportResult {
  const ContactImportResult({required this.contact, required this.created});

  final UserContact contact;
  final bool created;
}

enum ContactSyncPhase { idle, syncing, synced, pending, offline, failed }

class ContactSyncState {
  const ContactSyncState({
    required this.phase,
    this.updatedAt = 0,
    this.message,
  });

  final ContactSyncPhase phase;
  final int updatedAt;
  final String? message;

  String get label => switch (phase) {
        ContactSyncPhase.syncing => '正在同步',
        ContactSyncPhase.synced => '云端已同步',
        ContactSyncPhase.pending => '待同步',
        ContactSyncPhase.offline => '离线，显示本地通讯录',
        ContactSyncPhase.failed => '同步失败，点击重试',
        ContactSyncPhase.idle => '本地通讯录',
      };
}

/// 联系人端到端加密器。AES-GCM 保护内容与完整性，HMAC 生成不透明 contact_id；
/// 两把钥匙均由 WalletManager 从 seed 域隔离派生，本类永远接触不到 seed。
class ContactCryptor {
  ContactCryptor({required this.ownerAccount, required this.keys});

  static const String schema = 'citizenapp.contacts.v1';
  final String ownerAccount;
  final ContactKeyMaterial keys;
  final AesGcm _aes = AesGcm.with256bits();
  final Hmac _hmac = Hmac.sha256();

  Future<String> contactId(String address) async {
    final mac = await _hmac.calculateMac(
      utf8.encode(UserContactService.normalizeAddress(address)),
      secretKey: SecretKey(keys.indexKey),
    );
    return _hex(mac.bytes);
  }

  Future<SquareEncryptedContact> encrypt(UserContact contact) async {
    final id = await contactId(contact.address);
    final clear = utf8.encode(jsonEncode(<String, Object?>{
      'schema': schema,
      'owner_account': ownerAccount,
      ...contact.toJson(),
    }));
    final nonce = _randomBytes(12);
    final box = await _aes.encrypt(
      clear,
      secretKey: SecretKey(keys.encryptionKey),
      nonce: nonce,
      aad: _aad(id),
    );
    return SquareEncryptedContact(
      contactId: id,
      ciphertext: _base64UrlEncode(box.cipherText),
      nonce: _base64UrlEncode(box.nonce),
      mac: _base64UrlEncode(box.mac.bytes),
      updatedAt: contact.updatedAt,
    );
  }

  Future<UserContact> decrypt(SquareEncryptedContact record) async {
    try {
      final clear = await _aes.decrypt(
        SecretBox(
          _base64UrlDecode(record.ciphertext),
          nonce: _base64UrlDecode(record.nonce),
          mac: Mac(_base64UrlDecode(record.mac)),
        ),
        secretKey: SecretKey(keys.encryptionKey),
        aad: _aad(record.contactId),
      );
      final decoded = jsonDecode(utf8.decode(clear));
      if (decoded is! Map<String, dynamic> ||
          decoded['schema'] != schema ||
          decoded['owner_account'] != ownerAccount) {
        throw const FormatException('通讯录密文归属或版本不匹配');
      }
      final contact = UserContact.fromJson(decoded);
      if (await contactId(contact.address) != record.contactId) {
        throw const FormatException('通讯录密文索引不匹配');
      }
      return contact;
    } on SecretBoxAuthenticationError {
      throw const FormatException('通讯录密文认证失败');
    }
  }

  List<int> _aad(String id) => utf8.encode('$schema|$ownerAccount|$id');
}

/// 本地优先的加密通讯录服务。Isar 保存按 owner 隔离的可用缓存与待同步操作；
/// Cloudflare 只接收 [SquareEncryptedContact]，网络失败不会阻塞本地增删改。
class UserContactService {
  UserContactService({
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    SquareApiClient? apiClient,
    bool autoSync = true,
  })  : _walletManager = walletManager ?? WalletManager(),
        _sessionProvider = sessionProvider ?? SquareSessionProvider.instance,
        _apiClient = apiClient ?? SquareApiClient(),
        _autoSync = autoSync;

  static const String legacyPreferencesKey = 'user.contacts.items.v2';
  static const int _ss58Prefix = 2027;
  static const String _contactsPrefix = 'contacts:';
  static const String _pendingPrefix = 'contact_pending_ops:';
  static const String _syncPrefix = 'contact_sync_state:';

  final WalletManager _walletManager;
  final SquareSessionProvider _sessionProvider;
  final SquareApiClient _apiClient;
  final bool _autoSync;

  final ValueNotifier<ContactSyncState> syncState =
      ValueNotifier<ContactSyncState>(
    const ContactSyncState(phase: ContactSyncPhase.idle),
  );

  /// 通讯录只属于“我的钱包”中的默认热钱包，调用方不得用交易付款钱包覆盖。
  Future<List<UserContact>> getContacts() async {
    final owner = (await _requireDefaultWallet()).address;
    return _getContacts(owner);
  }

  Future<List<UserContact>> _getContacts(String owner) async {
    await _migrateLegacyContacts(owner);
    return _readContacts(owner);
  }

  /// 返回通讯录当前所属的默认用户账户，供扫码页做“不能添加自己”校验。
  Future<String> getOwnerAccount() async =>
      (await _requireDefaultWallet()).address;

  Future<ContactImportResult> addContact({
    required String address,
    required String contactName,
  }) async {
    final wallet = await _requireDefaultWallet();
    final owner = wallet.address;
    await _migrateLegacyContacts(owner);
    final normalizedAddress = normalizeAddress(address);
    final normalizedName = contactName.trim();
    if (normalizedName.isEmpty) {
      throw const FormatException('联系人名称为空');
    }
    if (normalizedAddress == owner) {
      throw const FormatException('不能把自己加入通讯录');
    }

    final contacts = (await _readContacts(owner)).toList(growable: true);
    final index =
        contacts.indexWhere((item) => item.address == normalizedAddress);
    final created = index < 0;
    final now = _nextTimestamp(created ? 0 : contacts[index].updatedAt);
    final contact = created
        ? UserContact(
            address: normalizedAddress,
            contactName: normalizedName,
            createdAt: now,
            updatedAt: now,
          )
        : contacts[index].copyWith(contactName: normalizedName, updatedAt: now);
    if (created) {
      contacts.add(contact);
    } else {
      contacts[index] = contact;
    }
    await _writeContactsAndPending(
      owner,
      contacts,
      _PendingContactOp.upsert(contact.address, contact.updatedAt),
    );
    if (_autoSync) {
      unawaited(_syncWallet(wallet));
    }
    return ContactImportResult(contact: contact, created: created);
  }

  Future<List<UserContact>> renameContact(
    String address,
    String contactName,
  ) async {
    final wallet = await _requireDefaultWallet();
    final owner = wallet.address;
    final normalizedAddress = normalizeAddress(address);
    final normalizedName = contactName.trim();
    if (normalizedName.isEmpty) {
      throw const FormatException('联系人名称不能为空');
    }
    final contacts = (await _getContacts(owner)).toList(growable: true);
    final index =
        contacts.indexWhere((item) => item.address == normalizedAddress);
    if (index < 0) {
      throw Exception('未找到联系人');
    }
    contacts[index] = contacts[index].copyWith(
      contactName: normalizedName,
      updatedAt: _nextTimestamp(contacts[index].updatedAt),
    );
    await _writeContactsAndPending(
      owner,
      contacts,
      _PendingContactOp.upsert(
        contacts[index].address,
        contacts[index].updatedAt,
      ),
    );
    if (_autoSync) {
      unawaited(_syncWallet(wallet));
    }
    return _sorted(contacts);
  }

  Future<List<UserContact>> deleteContact(String address) async {
    final wallet = await _requireDefaultWallet();
    final owner = wallet.address;
    final normalizedAddress = normalizeAddress(address);
    final contacts = (await _getContacts(owner))
        .where((item) => item.address != normalizedAddress)
        .toList(growable: false);
    await _writeContactsAndPending(
      owner,
      contacts,
      _PendingContactOp.delete(
        normalizedAddress,
        _nextTimestamp(),
      ),
    );
    if (_autoSync) {
      unawaited(_syncWallet(wallet));
    }
    return _sorted(contacts);
  }

  /// 拉云端快照后重放本机待同步操作。损坏或属于其他钱包的密文只被忽略，
  /// 绝不覆盖本机有效缓存；下一次正常写入会修复对应云端记录。
  /// 同步入口同样只接受默认用户；付款钱包和调用方参数不能改变密文归属。
  Future<List<UserContact>> sync() async {
    final wallet = await _requireDefaultWallet();
    return _syncWallet(wallet);
  }

  Future<List<UserContact>> _syncWallet(WalletProfile wallet) async {
    final owner = wallet.address;
    await _migrateLegacyContacts(owner);
    await _setSyncState(owner, ContactSyncPhase.syncing);
    try {
      final keys = await _walletManager.ensureContactKeyMaterial(
        walletIndex: wallet.walletIndex,
        ownerAccount: owner,
      );
      final session = await _sessionProvider.ensureSession();
      if (session == null || session.ownerAccount != owner) {
        throw const SquareApiException('通讯录云同步需要默认热钱包会话');
      }
      final cryptor = ContactCryptor(ownerAccount: owner, keys: keys);
      final cloudRecords = <SquareEncryptedContact>[];
      String? cursor;
      do {
        final page = await _apiClient.fetchEncryptedContacts(
          session: session,
          cursor: cursor,
        );
        cloudRecords.addAll(page.items);
        cursor = page.nextCursor;
      } while (cursor != null);

      final pending = await _readPending(owner);
      final pendingAddresses = pending.map((item) => item.address).toSet();
      final local = await _readContacts(owner);
      final merged = <String, UserContact>{};
      final localByContactId = <String, UserContact>{};
      for (final contact in local) {
        localByContactId[await cryptor.contactId(contact.address)] = contact;
      }
      for (final record in cloudRecords) {
        try {
          final contact = await cryptor.decrypt(record);
          if (!pendingAddresses.contains(contact.address)) {
            merged[contact.address] = contact;
          }
        } on FormatException {
          // 单条损坏不应让整个通讯录不可用，也不能覆盖同 ID 的本地有效缓存。
          final cached = localByContactId[record.contactId];
          if (cached != null) merged[cached.address] = cached;
        }
      }
      for (final contact in local) {
        if (pendingAddresses.contains(contact.address)) {
          merged[contact.address] = contact;
        }
      }
      await _writeContacts(owner, merged.values.toList(growable: false));

      for (final op in List<_PendingContactOp>.from(pending)) {
        if (op.action == _PendingAction.delete) {
          await _apiClient.deleteEncryptedContact(
            session: session,
            contactId: await cryptor.contactId(op.address),
          );
        } else {
          final contact = merged[op.address];
          if (contact == null) continue;
          await _apiClient.putEncryptedContact(
            session: session,
            contact: await cryptor.encrypt(contact),
          );
        }
        await _removePending(owner, op);
      }
      final result = await _readContacts(owner);
      await _setSyncState(owner, ContactSyncPhase.synced);
      return result;
    } on Exception catch (error) {
      final pending = await _readPending(owner);
      final phase =
          pending.isEmpty ? ContactSyncPhase.offline : ContactSyncPhase.failed;
      await _setSyncState(owner, phase, message: error.toString());
      return _readContacts(owner);
    }
  }

  Future<ContactSyncState> readSyncState() async {
    final owner = (await _requireDefaultWallet()).address;
    final raw = await _readKv('$_syncPrefix$owner');
    if (raw == null) {
      return const ContactSyncState(phase: ContactSyncPhase.idle);
    }
    try {
      final json = jsonDecode(raw);
      if (json is! Map<String, dynamic>) throw const FormatException();
      final phaseName = json['phase']?.toString();
      final phase = ContactSyncPhase.values.firstWhere(
        (item) => item.name == phaseName,
        orElse: () => ContactSyncPhase.idle,
      );
      return ContactSyncState(
        phase: phase,
        updatedAt: _asInt(json['updated_at']),
        message: json['message']?.toString(),
      );
    } on FormatException {
      return const ContactSyncState(phase: ContactSyncPhase.idle);
    }
  }

  Future<WalletProfile> _requireDefaultWallet() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) throw const WalletAuthException('请先创建默认热钱包');
    return wallet;
  }

  Future<void> _migrateLegacyContacts(String owner) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(legacyPreferencesKey);
    if (raw == null) return;
    final migrated = <UserContact>[];
    try {
      final decoded = jsonDecode(raw);
      if (decoded is List) {
        for (final item in decoded.whereType<Map<String, dynamic>>()) {
          final address = item['address']?.toString().trim() ?? '';
          final localName = item['local_nickname']?.toString().trim() ?? '';
          final sourceName = item['source_nickname']?.toString().trim() ?? '';
          final name = localName.isNotEmpty ? localName : sourceName;
          if (address.isEmpty || name.isEmpty) continue;
          // 历史版本允许 0 时间戳；迁移时一次性收口为 Worker 可接受的正整数，
          // 不保留旧格式兼容分支。
          final rawCreatedAt = _asInt(item['added_at']);
          final rawUpdatedAt = _asInt(item['updated_at']);
          final createdAt = rawCreatedAt > 0 ? rawCreatedAt : _nextTimestamp();
          final updatedAt = rawUpdatedAt > 0
              ? rawUpdatedAt.clamp(createdAt, 0x7fffffffffffffff).toInt()
              : createdAt;
          migrated.add(UserContact(
            address: normalizeAddress(address),
            contactName: name,
            createdAt: createdAt,
            updatedAt: updatedAt,
          ));
        }
      }
      if (migrated.isNotEmpty) {
        final pending = migrated
            .map((item) =>
                _PendingContactOp.upsert(item.address, item.updatedAt))
            .toList(growable: false);
        await _writeSnapshot(owner, migrated, pending);
      }
    } finally {
      // 无论旧载荷是否合法都只处理一次；不保留双轨读取或损坏旧数据死循环。
      await prefs.remove(legacyPreferencesKey);
    }
  }

  Future<List<UserContact>> _readContacts(String owner) async {
    final raw = await _readKv('$_contactsPrefix$owner');
    if (raw == null || raw.isEmpty) return const <UserContact>[];
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! List) return const <UserContact>[];
      return _sorted(decoded
          .whereType<Map<String, dynamic>>()
          .map(UserContact.fromJson)
          .toList(growable: false));
    } on FormatException {
      return const <UserContact>[];
    }
  }

  Future<List<_PendingContactOp>> _readPending(String owner) async {
    final raw = await _readKv('$_pendingPrefix$owner');
    if (raw == null || raw.isEmpty) return const <_PendingContactOp>[];
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! List) return const <_PendingContactOp>[];
      return decoded
          .whereType<Map<String, dynamic>>()
          .map(_PendingContactOp.fromJson)
          .toList(growable: false);
    } on FormatException {
      return const <_PendingContactOp>[];
    }
  }

  Future<void> _writeContactsAndPending(
    String owner,
    List<UserContact> contacts,
    _PendingContactOp next,
  ) async {
    final pending = (await _readPending(owner)).toList(growable: true)
      ..removeWhere((item) => item.address == next.address)
      ..add(next);
    await _writeSnapshot(owner, contacts, pending);
    await _setSyncState(owner, ContactSyncPhase.pending);
  }

  Future<void> _removePending(String owner, _PendingContactOp completed) async {
    final pending = (await _readPending(owner))
        .where((item) =>
            item.address != completed.address ||
            item.updatedAt > completed.updatedAt)
        .toList(growable: false);
    await _writePending(owner, pending);
  }

  Future<void> _writeSnapshot(
    String owner,
    List<UserContact> contacts,
    List<_PendingContactOp> pending,
  ) {
    return WalletIsar.instance.writeTxn((isar) async {
      await _putKvInTxn(
        isar,
        '$_contactsPrefix$owner',
        jsonEncode(_sorted(contacts).map((item) => item.toJson()).toList()),
      );
      await _putKvInTxn(
        isar,
        '$_pendingPrefix$owner',
        jsonEncode(pending.map((item) => item.toJson()).toList()),
      );
    });
  }

  Future<void> _writeContacts(String owner, List<UserContact> contacts) =>
      _writeKv(
        '$_contactsPrefix$owner',
        jsonEncode(_sorted(contacts).map((item) => item.toJson()).toList()),
      );

  Future<void> _writePending(String owner, List<_PendingContactOp> pending) =>
      _writeKv(
        '$_pendingPrefix$owner',
        jsonEncode(pending.map((item) => item.toJson()).toList()),
      );

  Future<void> _setSyncState(
    String owner,
    ContactSyncPhase phase, {
    String? message,
  }) async {
    final state = ContactSyncState(
      phase: phase,
      updatedAt: DateTime.now().millisecondsSinceEpoch,
      message: message,
    );
    syncState.value = state;
    await _writeKv(
      '$_syncPrefix$owner',
      jsonEncode(<String, Object?>{
        'phase': phase.name,
        'updated_at': state.updatedAt,
        if (message != null) 'message': message,
      }),
    );
  }

  Future<String?> _readKv(String key) => WalletIsar.instance.read((isar) async {
        return (await isar.appKvEntitys.getByKey(key))?.stringValue;
      });

  Future<void> _writeKv(String key, String value) =>
      WalletIsar.instance.writeTxn((isar) => _putKvInTxn(isar, key, value));

  Future<void> _putKvInTxn(Isar isar, String key, String value) async {
    final row = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
    row
      ..key = key
      ..stringValue = value;
    await isar.appKvEntitys.put(row);
  }

  static String normalizeAddress(String input) {
    final trimmed = input.trim();
    if (trimmed.isEmpty) throw const FormatException('地址为空');
    try {
      final bytes = Keyring().decodeAddress(trimmed);
      final normalized = Keyring().encodeAddress(bytes, _ss58Prefix);
      if (normalized != trimmed) {
        throw const FormatException('联系人地址不是本链 SS58 地址');
      }
      return normalized;
    } on FormatException {
      rethrow;
    } catch (_) {
      throw const FormatException('联系人地址格式无效');
    }
  }
}

enum _PendingAction { upsert, delete }

class _PendingContactOp {
  const _PendingContactOp({
    required this.action,
    required this.address,
    required this.updatedAt,
  });

  factory _PendingContactOp.upsert(String address, int updatedAt) =>
      _PendingContactOp(
        action: _PendingAction.upsert,
        address: address,
        updatedAt: updatedAt,
      );

  factory _PendingContactOp.delete(String address, int updatedAt) =>
      _PendingContactOp(
        action: _PendingAction.delete,
        address: address,
        updatedAt: updatedAt,
      );

  final _PendingAction action;
  final String address;
  final int updatedAt;

  Map<String, Object?> toJson() => <String, Object?>{
        'action': action.name,
        'address': address,
        'updated_at': updatedAt,
      };

  factory _PendingContactOp.fromJson(Map<String, dynamic> json) {
    final action = json['action'] == 'delete'
        ? _PendingAction.delete
        : _PendingAction.upsert;
    return _PendingContactOp(
      action: action,
      address: UserContactService.normalizeAddress(
        json['address']?.toString() ?? '',
      ),
      updatedAt: _asInt(json['updated_at']),
    );
  }
}

List<UserContact> _sorted(Iterable<UserContact> contacts) =>
    contacts.toList()..sort((a, b) => b.updatedAt.compareTo(a.updatedAt));

int _asInt(Object? value) {
  if (value is int) return value;
  if (value is num) return value.toInt();
  return int.tryParse(value?.toString() ?? '') ?? 0;
}

/// 联系人冲突时间戳必须为正且单设备单调递增，避免同一毫秒内连续修改被旧值覆盖。
int _nextTimestamp([int previous = 0]) {
  final now = DateTime.now().millisecondsSinceEpoch;
  return now > previous ? now : previous + 1;
}

Uint8List _randomBytes(int length) {
  final random = Random.secure();
  return Uint8List.fromList(
    List<int>.generate(length, (_) => random.nextInt(256)),
  );
}

String _hex(List<int> bytes) =>
    bytes.map((value) => value.toRadixString(16).padLeft(2, '0')).join();

/// Worker 契约只接受 RFC 4648 Base64URL 字符集且不接受 `=` 填充。
String _base64UrlEncode(List<int> bytes) =>
    base64UrlEncode(bytes).replaceAll('=', '');

List<int> _base64UrlDecode(String value) {
  final padded = value.padRight(((value.length + 3) ~/ 4) * 4, '=');
  return base64Url.decode(padded);
}
