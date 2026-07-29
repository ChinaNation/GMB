import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/security/local_cipher.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 通讯录唯一业务模型。
///
/// `cid_number` 是联系人关系的永久主键；`account_id` / `ss58_address` 是该 CID
/// 当前绑定的签名账户与展示地址，换绑后允许更新。公开昵称、头像和签名属于用户公开
/// 资料，不复制进通讯录；`contact_remark` 只保存当前用户自己的私人备注。
class UserContact {
  const UserContact({
    required this.cidNumber,
    required this.accountId,
    required this.ss58Address,
    required this.contactRemark,
    required this.createdAt,
    required this.updatedAt,
  });

  final String cidNumber;
  final String accountId;
  final String ss58Address;
  final String contactRemark;
  final int createdAt;
  final int updatedAt;

  UserContact copyWith({
    String? cidNumber,
    String? accountId,
    String? ss58Address,
    String? contactRemark,
    int? createdAt,
    int? updatedAt,
  }) {
    return UserContact(
      cidNumber: cidNumber ?? this.cidNumber,
      accountId: accountId ?? this.accountId,
      ss58Address: ss58Address ?? this.ss58Address,
      contactRemark: contactRemark ?? this.contactRemark,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
        'cid_number': cidNumber,
        'account_id': accountId,
        'ss58_address': ss58Address,
        'contact_remark': contactRemark,
        'created_at': createdAt,
        'updated_at': updatedAt,
      };

  factory UserContact.fromJson(Map<String, dynamic> json) {
    final cidNumber = UserContactService.requireCidNumber(
      json['cid_number']?.toString() ?? '',
    );
    final accountId = json['account_id']?.toString() ?? '';
    final ss58Address = json['ss58_address']?.toString().trim() ?? '';
    final contactRemark = json['contact_remark'];
    if (!isAccountIdText(accountId) ||
        ss58Address.isEmpty ||
        contactRemark is! String) {
      throw const FormatException('通讯录 CID、账户、地址或私人备注不合法');
    }
    if (UserContactService.accountIdFromSs58(ss58Address) != accountId) {
      throw const FormatException('通讯录 account_id 与 ss58_address 不匹配');
    }
    final createdAt = _asInt(json['created_at']);
    final updatedAt = _asInt(json['updated_at']);
    if (createdAt <= 0 || updatedAt <= 0) {
      throw const FormatException('通讯录时间戳不合法');
    }
    final normalizedRemark =
        UserContactService.normalizeContactRemark(contactRemark);
    return UserContact(
      cidNumber: cidNumber,
      accountId: accountId,
      ss58Address: UserContactService.normalizeSs58Address(ss58Address),
      contactRemark: normalizedRemark,
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
  ContactCryptor({
    required String ownerCidNumber,
    required this.keys,
  }) : ownerCidNumber = UserContactService.requireCidNumber(ownerCidNumber);

  static const String _domain = 'citizenapp.contacts';
  final String ownerCidNumber;
  final ContactKeyMaterial keys;
  final AesGcm _aes = AesGcm.with256bits();
  final Hmac _hmac = Hmac.sha256();

  Future<String> contactId(String cidNumber) async {
    final mac = await _hmac.calculateMac(
      utf8.encode(UserContactService.requireCidNumber(cidNumber)),
      secretKey: SecretKey(keys.indexKey),
    );
    return _hex(mac.bytes);
  }

  Future<SquareEncryptedContact> encrypt(UserContact contact) async {
    final id = await contactId(contact.cidNumber);
    final clear = utf8.encode(jsonEncode(<String, Object?>{
      'owner_cid_number': ownerCidNumber,
      'cid_number': contact.cidNumber,
      'account_id': contact.accountId,
      'ss58_address': contact.ss58Address,
      'contact_remark': contact.contactRemark,
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
          decoded['owner_cid_number'] != ownerCidNumber) {
        throw const FormatException('通讯录密文归属不匹配');
      }
      final contact = UserContact.fromJson(<String, dynamic>{
        'cid_number': decoded['cid_number'],
        'account_id': decoded['account_id'],
        'ss58_address': decoded['ss58_address'],
        'contact_remark': decoded['contact_remark'],
        'created_at': decoded['created_at'],
        'updated_at': decoded['updated_at'],
      });
      if (await contactId(contact.cidNumber) != record.contactId) {
        throw const FormatException('通讯录密文索引不匹配');
      }
      return contact;
    } on SecretBoxAuthenticationError {
      throw const FormatException('通讯录密文认证失败');
    }
  }

  List<int> _aad(String id) => utf8.encode('$_domain|$ownerCidNumber|$id');
}

/// 本地优先的加密通讯录服务。Isar 保存按 accountId 隔离的可用缓存与待同步操作；
/// Cloudflare 只接收 [SquareEncryptedContact]，网络失败不会阻塞本地增删改。
class UserContactService {
  UserContactService({
    WalletManager? walletManager,
    SquareSessionProvider? sessionProvider,
    SquareApiClient? apiClient,
    IdentityAccountCache? identityAccountCache,
    bool autoSync = true,
  })  : _walletManager = walletManager ?? WalletManager(),
        _sessionProvider = sessionProvider ?? SquareSessionProvider.instance,
        _apiClient = apiClient ?? SquareApiClient(),
        _identityAccountCache = identityAccountCache,
        _autoSync = autoSync;

  static const int _ss58Prefix = 2027;
  static const String _contactsPrefix = 'contact_book_by_account:';
  static const String _pendingPrefix = 'contact_pending_by_account:';
  static const String _syncPrefix = 'contact_sync_by_account:';
  static const String _cloudResetPrefix = 'contact_cloud_reset_by_account:';

  final WalletManager _walletManager;
  final SquareSessionProvider _sessionProvider;
  final SquareApiClient _apiClient;
  final IdentityAccountCache? _identityAccountCache;
  final bool _autoSync;

  /// 通讯录归属 = **身份账户**(CID 绑定账户,唯一身份主键);通讯录密钥从身份账户
  /// child 派生、云会话 accountId 亦身份账户,三者一致(彻底切,不留账户0)。
  IdentityAccountCache get _identityCache =>
      _identityAccountCache ?? IdentityAccountCache.instance;

  final ValueNotifier<ContactSyncState> syncState =
      ValueNotifier<ContactSyncState>(
    const ContactSyncState(phase: ContactSyncPhase.idle),
  );

  /// 通讯录只属于当前 CID 身份，调用方不得用交易付款钱包覆盖其身份账户。
  Future<List<UserContact>> getContacts() async {
    final accountId = await _requireIdentityAccountId();
    return _getContacts(accountId);
  }

  Future<List<UserContact>> _getContacts(String accountId) async {
    return _readContacts(accountId);
  }

  /// 返回通讯录当前所属的身份账户，供扫码页做“不能添加自己”校验。
  Future<String> getAccountId() async => await _requireIdentityAccountId();

  Future<ContactImportResult> addContact({
    required String cidNumber,
    required String ss58Address,
    required String contactRemark,
  }) async {
    final ownerAccountId = await _requireIdentityAccountId();
    final normalizedCidNumber = requireCidNumber(cidNumber);
    final normalizedSs58Address = normalizeSs58Address(ss58Address);
    final contactAccountId = accountIdFromSs58(normalizedSs58Address);
    final normalizedRemark = normalizeContactRemark(contactRemark);
    if (contactAccountId == ownerAccountId) {
      throw const FormatException('不能把自己加入通讯录');
    }

    final contacts =
        (await _readContacts(ownerAccountId)).toList(growable: true);
    final index =
        contacts.indexWhere((item) => item.cidNumber == normalizedCidNumber);
    final created = index < 0;
    final now = _nextTimestamp(created ? 0 : contacts[index].updatedAt);
    final contact = created
        ? UserContact(
            cidNumber: normalizedCidNumber,
            accountId: contactAccountId,
            ss58Address: normalizedSs58Address,
            contactRemark: normalizedRemark,
            createdAt: now,
            updatedAt: now,
          )
        : contacts[index].copyWith(
            accountId: contactAccountId,
            ss58Address: normalizedSs58Address,
            // 扫码得到的空备注不得抹掉用户已经填写的私人备注。
            contactRemark: normalizedRemark.isEmpty
                ? contacts[index].contactRemark
                : normalizedRemark,
            updatedAt: now,
          );
    if (created) {
      contacts.add(contact);
    } else {
      contacts[index] = contact;
    }
    await _writeContactsAndPending(
      ownerAccountId,
      contacts,
      _PendingContactOp.upsert(contact.cidNumber, contact.updatedAt),
    );
    if (_autoSync) {
      unawaited(_syncWallet(ownerAccountId));
    }
    return ContactImportResult(contact: contact, created: created);
  }

  Future<List<UserContact>> renameContact(
    String contactCidNumber,
    String contactRemark,
  ) async {
    final ownerAccountId = await _requireIdentityAccountId();
    final normalizedContactCidNumber = requireCidNumber(contactCidNumber);
    final normalizedRemark = normalizeContactRemark(contactRemark);
    final contacts =
        (await _getContacts(ownerAccountId)).toList(growable: true);
    final index = contacts
        .indexWhere((item) => item.cidNumber == normalizedContactCidNumber);
    if (index < 0) {
      throw Exception('未找到联系人');
    }
    contacts[index] = contacts[index].copyWith(
      contactRemark: normalizedRemark,
      updatedAt: _nextTimestamp(contacts[index].updatedAt),
    );
    await _writeContactsAndPending(
      ownerAccountId,
      contacts,
      _PendingContactOp.upsert(
        contacts[index].cidNumber,
        contacts[index].updatedAt,
      ),
    );
    if (_autoSync) {
      unawaited(_syncWallet(ownerAccountId));
    }
    return _sorted(contacts);
  }

  Future<List<UserContact>> deleteContact(String contactCidNumber) async {
    final ownerAccountId = await _requireIdentityAccountId();
    final normalizedContactCidNumber = requireCidNumber(contactCidNumber);
    final contacts = (await _getContacts(ownerAccountId))
        .where((item) => item.cidNumber != normalizedContactCidNumber)
        .toList(growable: false);
    await _writeContactsAndPending(
      ownerAccountId,
      contacts,
      _PendingContactOp.delete(
        normalizedContactCidNumber,
        _nextTimestamp(),
      ),
    );
    if (_autoSync) {
      unawaited(_syncWallet(ownerAccountId));
    }
    return _sorted(contacts);
  }

  /// 拉云端快照后重放本机待同步操作。损坏或属于其他钱包的密文只被忽略，
  /// 绝不覆盖本机有效缓存；下一次正常写入会修复对应云端记录。
  /// 同步入口同样只接受身份账户；付款钱包和调用方参数不能改变密文归属。
  Future<List<UserContact>> sync() async {
    return _syncWallet(await _requireIdentityAccountId());
  }

  Future<List<UserContact>> _syncWallet(String accountId) async {
    await _setSyncState(accountId, ContactSyncPhase.syncing);
    try {
      final keys =
          await _walletManager.ensureContactKeyMaterialForAccountId(accountId);
      final session = await _sessionProvider.ensureSession();
      if (session == null || session.accountId != accountId) {
        throw const SquareApiException('通讯录云同步需要当前 CID 身份账户会话');
      }
      final cryptor = ContactCryptor(
        ownerCidNumber: requireCidNumber(session.cidNumber),
        keys: keys,
      );
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

      // CID 换绑会更换端到端密钥。旧密文无法用新密钥识别，必须先按属主 CID
      // 清空旧不透明记录，再用本地明文权威副本重建。标记在全量删除成功前不移除，
      // 中途断网后下次同步会继续删除剩余记录。
      final resetCloud =
          await _readKv('$_cloudResetPrefix$accountId') == 'true';
      if (resetCloud) {
        for (final record in cloudRecords) {
          await _apiClient.deleteEncryptedContact(
            session: session,
            contactId: record.contactId,
          );
        }
        cloudRecords.clear();
        await _deleteKvKeys(<String>['$_cloudResetPrefix$accountId']);
      }

      final pending = await _readPending(accountId);
      final pendingCidNumbers = pending.map((item) => item.cidNumber).toSet();
      final local = await _readContacts(accountId);
      final merged = <String, UserContact>{};
      final localByContactId = <String, UserContact>{};
      for (final contact in local) {
        localByContactId[await cryptor.contactId(contact.cidNumber)] = contact;
      }
      for (final record in cloudRecords) {
        try {
          final contact = await cryptor.decrypt(record);
          if (!pendingCidNumbers.contains(contact.cidNumber)) {
            merged[contact.cidNumber] = contact;
          }
        } on FormatException {
          // 单条损坏不应让整个通讯录不可用，也不能覆盖同 ID 的本地有效缓存。
          final cached = localByContactId[record.contactId];
          if (cached != null) merged[cached.cidNumber] = cached;
        }
      }
      for (final contact in local) {
        if (pendingCidNumbers.contains(contact.cidNumber)) {
          merged[contact.cidNumber] = contact;
        }
      }
      await _writeContacts(accountId, merged.values.toList(growable: false));

      for (final op in List<_PendingContactOp>.from(pending)) {
        if (op.action == _PendingAction.delete) {
          await _apiClient.deleteEncryptedContact(
            session: session,
            contactId: await cryptor.contactId(op.cidNumber),
          );
        } else {
          final contact = merged[op.cidNumber];
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

  /// CID 换绑重建:把通讯录从旧身份账户 [oldAccountId] 迁到新身份账户 [newAccountId]
  /// (死契约 [[cid-rebind-subkeys-must-auto-migrate]])。
  ///
  /// 本地明文缓存直接搬(无需解密),每条造 upsert 待办以强制用**新账户 child 密钥**重加密
  /// 上云;随后清旧账户本地缓存与通讯录密钥,并触发一次新账户云同步。上云失败落既有待办
  /// 机制,下次 sync 自动重试——最终一致收敛到新账户。迁移只搬本地权威副本,不依赖旧云端
  /// 密文可解(旧密钥已换),故换绑后旧密钥即可安全丢弃。
  Future<void> migrateContactsToNewIdentity(
    String oldAccountId,
    String newAccountId,
  ) async {
    final oldNorm = requireAccountId(oldAccountId);
    final newNorm = requireAccountId(newAccountId);
    if (oldNorm == newNorm) return;

    final oldContacts = await _readContacts(oldNorm);
    // 新账户换绑前通常无通讯录；按永久 CID 合并，同 CID 以旧身份本地副本为准。
    // 即使旧账户本地为空，也要把新账户已有缓存全部置为待上传，避免云快照清空后
    // 非 pending 的本地记录在合并阶段被误删。
    final merged = <String, UserContact>{};
    for (final contact in await _readContacts(newNorm)) {
      merged[contact.cidNumber] = contact;
    }
    for (final contact in oldContacts) {
      merged[contact.cidNumber] = contact;
    }
    final contacts = merged.values.toList(growable: false);
    if (contacts.isNotEmpty) {
      // 全量 upsert 待办:强制用新账户 child 密钥重新加密上云。
      final pending = contacts
          .map((contact) =>
              _PendingContactOp.upsert(contact.cidNumber, contact.updatedAt))
          .toList(growable: false);
      await _writeSnapshot(newNorm, contacts, pending);
    }
    // 云端记录属于永久 CID，但旧记录的 contact_id 与密文均由旧账户 child 密钥生成。
    // 持久化重建标记，确保删除旧快照成功后才上传新密文。
    await _writeKv('$_cloudResetPrefix$newNorm', 'true');
    // 清旧账户本地缓存(明文/待办/同步态)与通讯录密钥,杜绝残留。
    await _deleteKvKeys(<String>[
      '$_contactsPrefix$oldNorm',
      '$_pendingPrefix$oldNorm',
      '$_syncPrefix$oldNorm',
      '$_cloudResetPrefix$oldNorm',
    ]);
    await _walletManager.deleteContactKeysForAccountId(oldNorm);
    // 触发新账户云重建(内部吞异常并落待办,不阻断换绑重建主流程)。
    await _syncWallet(newNorm);
  }

  Future<ContactSyncState> readSyncState() async {
    final accountId = await _requireIdentityAccountId();
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

  /// 通讯录归属的**身份账户** accountId(身份主键 = CID 号)。无热钱包 / 无身份则抛。
  Future<String> _requireIdentityAccountId() async {
    final accountId = await _identityCache.accountId();
    if (accountId == null || accountId.isEmpty) {
      throw const WalletAuthException('请先创建默认热钱包');
    }
    return accountId;
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
      ..removeWhere((item) => item.cidNumber == next.cidNumber)
      ..add(next);
    await _writeSnapshot(accountId, contacts, pending);
    await _setSyncState(accountId, ContactSyncPhase.pending);
  }

  Future<void> _removePending(
      String accountId, _PendingContactOp completed) async {
    final pending = (await _readPending(accountId))
        .where((item) =>
            item.cidNumber != completed.cidNumber ||
            item.updatedAt > completed.updatedAt)
        .toList(growable: false);
    await _writePending(accountId, pending);
  }

  Future<void> _writeSnapshot(
    String accountId,
    List<UserContact> contacts,
    List<_PendingContactOp> pending,
  ) async {
    // 两份都在事务外先加密，不让密码学运算占住 Isar 写事务。
    final contactsKey = '$_contactsPrefix$accountId';
    final pendingKey = '$_pendingPrefix$accountId';
    final sealedContacts = await _sealKv(
      contactsKey,
      jsonEncode(_sorted(contacts).map((item) => item.toJson()).toList()),
    );
    final sealedPending = await _sealKv(
      pendingKey,
      jsonEncode(pending.map((item) => item.toJson()).toList()),
    );
    await WalletIsar.instance.writeTxn((isar) async {
      await _putKvInTxn(isar, contactsKey, sealedContacts);
      await _putKvInTxn(isar, pendingKey, sealedPending);
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


  /// 本地 KV 密文层的子钥缓存(按归属账户)。
  final Map<String, Uint8List> _localKvKeys = <String, Uint8List>{};

  /// 取某条 KV 的本地加密子钥。
  ///
  /// 四个前缀都以 `:<accountId>` 结尾,归属账户可直接从键名解析,无需层层透传。
  /// 用 `LocalKeyPurpose.contactsLocal` 而**不复用云端通讯录钥**:两者域隔离,
  /// 本地密文被拿到也不等于同时暴露云端密文。
  Future<Uint8List> _localKvKey(String kvKey) async {
    final accountId = kvKey.substring(kvKey.indexOf(':') + 1);
    final cached = _localKvKeys[accountId];
    if (cached != null) return cached;
    final ldk = await _walletManager.ensureLocalDataKeyForAccountId(accountId);
    final key = await ldk.subkey(LocalKeyPurpose.contactsLocal);
    _localKvKeys[accountId] = key;
    return key;
  }

  /// AAD 绑完整 KV 键名,防止三份(通讯录 / 待同步 / 同步态)密文被互换。
  Future<String> _sealKv(String kvKey, String value) async => LocalCipher
      .encryptString(key: await _localKvKey(kvKey), plaintext: value, aad: kvKey);

  Future<String> _openKv(String kvKey, String blob) async => LocalCipher
      .decryptString(key: await _localKvKey(kvKey), blob: blob, aad: kvKey);

  /// 读本地 KV 并解密。解密失败直接抛 [LocalCipherException],不静默返回 null——
  /// 静默会被上层当成"本地无缓存"而拉云端整表覆盖,悄悄丢掉待同步的本地改动。
  Future<String?> _readKv(String key) async {
    final blob = await WalletIsar.instance.read((isar) async {
      return (await isar.appKvEntitys.getByKey(key))?.stringValue;
    });
    if (blob == null || blob.isEmpty) return null;
    return _openKv(key, blob);
  }

  /// 加密在事务外完成,不让密码学运算占住 Isar 写事务。
  Future<void> _writeKv(String key, String value) async {
    final sealed = await _sealKv(key, value);
    await WalletIsar.instance.writeTxn((isar) => _putKvInTxn(isar, key, sealed));
  }

  Future<void> _deleteKvKeys(List<String> keys) =>
      WalletIsar.instance.writeTxn((isar) async {
        for (final key in keys) {
          await isar.appKvEntitys.deleteByKey(key);
        }
      });

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
    if (!isAccountIdText(accountId)) {
      throw const FormatException('account_id 必须为小写 0x + 64 位十六进制');
    }
    return accountId;
  }

  static String requireCidNumber(String cidNumber) {
    final normalized = cidNumber.trim();
    if (normalized.isEmpty || utf8.encode(normalized).length > 32) {
      throw const FormatException('cid_number 必须为 1 到 32 字节');
    }
    return normalized;
  }

  static String normalizeContactRemark(String contactRemark) {
    final normalized = contactRemark.trim();
    if (normalized.runes.length > 40) {
      throw const FormatException('联系人私人备注不能超过 40 个字符');
    }
    return normalized;
  }
}

enum _PendingAction { upsert, delete }

class _PendingContactOp {
  const _PendingContactOp({
    required this.action,
    required this.cidNumber,
    required this.updatedAt,
  });

  factory _PendingContactOp.upsert(String cidNumber, int updatedAt) =>
      _PendingContactOp(
        action: _PendingAction.upsert,
        cidNumber: cidNumber,
        updatedAt: updatedAt,
      );

  factory _PendingContactOp.delete(String cidNumber, int updatedAt) =>
      _PendingContactOp(
        action: _PendingAction.delete,
        cidNumber: cidNumber,
        updatedAt: updatedAt,
      );

  final _PendingAction action;
  final String cidNumber;
  final int updatedAt;

  Map<String, Object?> toJson() => <String, Object?>{
        'action': action.name,
        'cid_number': cidNumber,
        'updated_at': updatedAt,
      };

  factory _PendingContactOp.fromJson(Map<String, dynamic> json) {
    final action = json['action'] == 'delete'
        ? _PendingAction.delete
        : _PendingAction.upsert;
    return _PendingContactOp(
      action: action,
      cidNumber: UserContactService.requireCidNumber(
        json['cid_number']?.toString() ?? '',
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
