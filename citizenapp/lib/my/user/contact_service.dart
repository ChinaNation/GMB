import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 通讯录唯一业务模型。公开昵称、头像和签名属于用户公开资料，不复制进通讯录；
/// 本模型只保存联系人账户与当前用户自己的私人联系人名称。
class UserContact {
  const UserContact({
    required this.accountId,
    required this.ss58Address,
    required this.contactName,
    required this.createdAt,
    required this.updatedAt,
  });

  final String accountId;
  final String ss58Address;
  final String contactName;
  final int createdAt;
  final int updatedAt;

  UserContact copyWith({
    String? accountId,
    String? ss58Address,
    String? contactName,
    int? createdAt,
    int? updatedAt,
  }) {
    return UserContact(
      accountId: accountId ?? this.accountId,
      ss58Address: ss58Address ?? this.ss58Address,
      contactName: contactName ?? this.contactName,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
        'account_id': accountId,
        'ss58_address': ss58Address,
        'contact_name': contactName,
        'created_at': createdAt,
        'updated_at': updatedAt,
      };

  factory UserContact.fromJson(Map<String, dynamic> json) {
    final accountId = json['account_id']?.toString() ?? '';
    final ss58Address = json['ss58_address']?.toString().trim() ?? '';
    final contactName = json['contact_name']?.toString().trim() ?? '';
    if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId) ||
        ss58Address.isEmpty ||
        contactName.isEmpty) {
      throw const FormatException('通讯录地址或联系人名称为空');
    }
    if (UserContactService.accountIdFromSs58(ss58Address) != accountId) {
      throw const FormatException('通讯录 account_id 与 ss58_address 不匹配');
    }
    final createdAt = _asInt(json['created_at']);
    final updatedAt = _asInt(json['updated_at']);
    if (createdAt <= 0 || updatedAt <= 0) {
      throw const FormatException('通讯录时间戳不合法');
    }
    return UserContact(
      accountId: accountId,
      ss58Address: UserContactService.normalizeSs58Address(ss58Address),
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
  ContactCryptor({required this.accountId, required this.keys});

  static const String schema = 'citizenapp.contacts.v1';
  final String accountId;
  final ContactKeyMaterial keys;
  final AesGcm _aes = AesGcm.with256bits();
  final Hmac _hmac = Hmac.sha256();

  Future<String> contactId(String accountId) async {
    final mac = await _hmac.calculateMac(
      utf8.encode(UserContactService.requireAccountId(accountId)),
      secretKey: SecretKey(keys.indexKey),
    );
    return _hex(mac.bytes);
  }

  Future<SquareEncryptedContact> encrypt(UserContact contact) async {
    final id = await contactId(contact.accountId);
    final clear = utf8.encode(jsonEncode(<String, Object?>{
      'schema': schema,
      'account_id': accountId,
      'contact_account_id': contact.accountId,
      'ss58_address': contact.ss58Address,
      'contact_name': contact.contactName,
      'created_at': contact.createdAt,
      'updated_at': contact.updatedAt,
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
          decoded['account_id'] != accountId) {
        throw const FormatException('通讯录密文归属或版本不匹配');
      }
      final contact = UserContact.fromJson(<String, dynamic>{
        ...decoded,
        'account_id': decoded['contact_account_id'],
      });
      if (await contactId(contact.accountId) != record.contactId) {
        throw const FormatException('通讯录密文索引不匹配');
      }
      return contact;
    } on SecretBoxAuthenticationError {
      throw const FormatException('通讯录密文认证失败');
    }
  }

  List<int> _aad(String id) => utf8.encode('$schema|$accountId|$id');
}

/// 本地优先的加密通讯录服务。Isar 保存按 accountId 隔离的可用缓存与待同步操作；
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
    final accountId = (await _requireDefaultWallet()).accountId;
    return _getContacts(accountId);
  }

  Future<List<UserContact>> _getContacts(String accountId) async {
    return _readContacts(accountId);
  }

  /// 返回通讯录当前所属的默认用户账户，供扫码页做“不能添加自己”校验。
  Future<String> getAccountId() async =>
      (await _requireDefaultWallet()).accountId;

  Future<ContactImportResult> addContact({
    required String ss58Address,
    required String contactName,
  }) async {
    final wallet = await _requireDefaultWallet();
    final currentAccountId = wallet.accountId;
    final normalizedSs58Address = normalizeSs58Address(ss58Address);
    final accountId = accountIdFromSs58(normalizedSs58Address);
    final normalizedName = contactName.trim();
    if (normalizedName.isEmpty) {
      throw const FormatException('联系人名称为空');
    }
    if (accountId == wallet.accountId) {
      throw const FormatException('不能把自己加入通讯录');
    }

    final contacts =
        (await _readContacts(currentAccountId)).toList(growable: true);
    final index = contacts.indexWhere((item) => item.accountId == accountId);
    final created = index < 0;
    final now = _nextTimestamp(created ? 0 : contacts[index].updatedAt);
    final contact = created
        ? UserContact(
            accountId: accountId,
            ss58Address: normalizedSs58Address,
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
      currentAccountId,
      contacts,
      _PendingContactOp.upsert(contact.accountId, contact.updatedAt),
    );
    if (_autoSync) {
      unawaited(_syncWallet(wallet));
    }
    return ContactImportResult(contact: contact, created: created);
  }

  Future<List<UserContact>> renameContact(
    String contactAccountId,
    String contactName,
  ) async {
    final wallet = await _requireDefaultWallet();
    final accountId = wallet.accountId;
    final normalizedContactAccountId = requireAccountId(contactAccountId);
    final normalizedName = contactName.trim();
    if (normalizedName.isEmpty) {
      throw const FormatException('联系人名称不能为空');
    }
    final contacts = (await _getContacts(accountId)).toList(growable: true);
    final index = contacts
        .indexWhere((item) => item.accountId == normalizedContactAccountId);
    if (index < 0) {
      throw Exception('未找到联系人');
    }
    contacts[index] = contacts[index].copyWith(
      contactName: normalizedName,
      updatedAt: _nextTimestamp(contacts[index].updatedAt),
    );
    await _writeContactsAndPending(
      accountId,
      contacts,
      _PendingContactOp.upsert(
        contacts[index].accountId,
        contacts[index].updatedAt,
      ),
    );
    if (_autoSync) {
      unawaited(_syncWallet(wallet));
    }
    return _sorted(contacts);
  }

  Future<List<UserContact>> deleteContact(String contactAccountId) async {
    final wallet = await _requireDefaultWallet();
    final accountId = wallet.accountId;
    final normalizedContactAccountId = requireAccountId(contactAccountId);
    final contacts = (await _getContacts(accountId))
        .where((item) => item.accountId != normalizedContactAccountId)
        .toList(growable: false);
    await _writeContactsAndPending(
      accountId,
      contacts,
      _PendingContactOp.delete(
        normalizedContactAccountId,
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
    final accountId = wallet.accountId;
    await _setSyncState(accountId, ContactSyncPhase.syncing);
    try {
      final keys = await _walletManager.ensureContactKeyMaterial(
        walletIndex: wallet.walletIndex,
        accountId: accountId,
      );
      final session = await _sessionProvider.ensureSession();
      if (session == null || session.accountId != accountId) {
        throw const SquareApiException('通讯录云同步需要默认热钱包会话');
      }
      final cryptor = ContactCryptor(accountId: accountId, keys: keys);
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

      final pending = await _readPending(accountId);
      final pendingAccountIds = pending.map((item) => item.accountId).toSet();
      final local = await _readContacts(accountId);
      final merged = <String, UserContact>{};
      final localByContactId = <String, UserContact>{};
      for (final contact in local) {
        localByContactId[await cryptor.contactId(contact.accountId)] = contact;
      }
      for (final record in cloudRecords) {
        try {
          final contact = await cryptor.decrypt(record);
          if (!pendingAccountIds.contains(contact.accountId)) {
            merged[contact.accountId] = contact;
          }
        } on FormatException {
          // 单条损坏不应让整个通讯录不可用，也不能覆盖同 ID 的本地有效缓存。
          final cached = localByContactId[record.contactId];
          if (cached != null) merged[cached.accountId] = cached;
        }
      }
      for (final contact in local) {
        if (pendingAccountIds.contains(contact.accountId)) {
          merged[contact.accountId] = contact;
        }
      }
      await _writeContacts(accountId, merged.values.toList(growable: false));

      for (final op in List<_PendingContactOp>.from(pending)) {
        if (op.action == _PendingAction.delete) {
          await _apiClient.deleteEncryptedContact(
            session: session,
            contactId: await cryptor.contactId(op.accountId),
          );
        } else {
          final contact = merged[op.accountId];
          if (contact == null) continue;
          await _apiClient.putEncryptedContact(
            session: session,
            contact: await cryptor.encrypt(contact),
          );
        }
        await _removePending(accountId, op);
      }
      final result = await _readContacts(accountId);
      await _setSyncState(accountId, ContactSyncPhase.synced);
      return result;
    } on Exception catch (error) {
      final pending = await _readPending(accountId);
      final phase =
          pending.isEmpty ? ContactSyncPhase.offline : ContactSyncPhase.failed;
      await _setSyncState(accountId, phase, message: error.toString());
      return _readContacts(accountId);
    }
  }

  Future<ContactSyncState> readSyncState() async {
    final accountId = (await _requireDefaultWallet()).accountId;
    final raw = await _readKv('$_syncPrefix$accountId');
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

  Future<List<UserContact>> _readContacts(String accountId) async {
    final raw = await _readKv('$_contactsPrefix$accountId');
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

  Future<List<_PendingContactOp>> _readPending(String accountId) async {
    final raw = await _readKv('$_pendingPrefix$accountId');
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
    String accountId,
    List<UserContact> contacts,
    _PendingContactOp next,
  ) async {
    final pending = (await _readPending(accountId)).toList(growable: true)
      ..removeWhere((item) => item.accountId == next.accountId)
      ..add(next);
    await _writeSnapshot(accountId, contacts, pending);
    await _setSyncState(accountId, ContactSyncPhase.pending);
  }

  Future<void> _removePending(
      String accountId, _PendingContactOp completed) async {
    final pending = (await _readPending(accountId))
        .where((item) =>
            item.accountId != completed.accountId ||
            item.updatedAt > completed.updatedAt)
        .toList(growable: false);
    await _writePending(accountId, pending);
  }

  Future<void> _writeSnapshot(
    String accountId,
    List<UserContact> contacts,
    List<_PendingContactOp> pending,
  ) {
    return WalletIsar.instance.writeTxn((isar) async {
      await _putKvInTxn(
        isar,
        '$_contactsPrefix$accountId',
        jsonEncode(_sorted(contacts).map((item) => item.toJson()).toList()),
      );
      await _putKvInTxn(
        isar,
        '$_pendingPrefix$accountId',
        jsonEncode(pending.map((item) => item.toJson()).toList()),
      );
    });
  }

  Future<void> _writeContacts(String accountId, List<UserContact> contacts) =>
      _writeKv(
        '$_contactsPrefix$accountId',
        jsonEncode(_sorted(contacts).map((item) => item.toJson()).toList()),
      );

  Future<void> _writePending(
          String accountId, List<_PendingContactOp> pending) =>
      _writeKv(
        '$_pendingPrefix$accountId',
        jsonEncode(pending.map((item) => item.toJson()).toList()),
      );

  Future<void> _setSyncState(
    String accountId,
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
      '$_syncPrefix$accountId',
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

  static String normalizeSs58Address(String input) {
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

  static String accountIdFromSs58(String ss58Address) {
    final normalized = normalizeSs58Address(ss58Address);
    final bytes = Keyring().decodeAddress(normalized);
    return '0x${_hex(bytes)}';
  }

  static String requireAccountId(String accountId) {
    if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId)) {
      throw const FormatException('account_id 必须为小写 0x + 64 位十六进制');
    }
    return accountId;
  }
}

enum _PendingAction { upsert, delete }

class _PendingContactOp {
  const _PendingContactOp({
    required this.action,
    required this.accountId,
    required this.updatedAt,
  });

  factory _PendingContactOp.upsert(String accountId, int updatedAt) =>
      _PendingContactOp(
        action: _PendingAction.upsert,
        accountId: accountId,
        updatedAt: updatedAt,
      );

  factory _PendingContactOp.delete(String accountId, int updatedAt) =>
      _PendingContactOp(
        action: _PendingAction.delete,
        accountId: accountId,
        updatedAt: updatedAt,
      );

  final _PendingAction action;
  final String accountId;
  final int updatedAt;

  Map<String, Object?> toJson() => <String, Object?>{
        'action': action.name,
        'account_id': accountId,
        'updated_at': updatedAt,
      };

  factory _PendingContactOp.fromJson(Map<String, dynamic> json) {
    final action = json['action'] == 'delete'
        ? _PendingAction.delete
        : _PendingAction.upsert;
    return _PendingContactOp(
      action: action,
      accountId: UserContactService.requireAccountId(
        json['account_id']?.toString() ?? '',
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
