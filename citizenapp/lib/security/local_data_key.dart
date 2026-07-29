import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

import 'package:citizenapp/security/local_cipher.dart';

/// 本地静止态密钥用途域。
///
/// 每个用途一把独立子钥，任一泄露不波及其他；且与**云端**通讯录密钥
/// (`citizenapp.contacts/encryption`) 完全分开，云密文与本地密文不共钥。
enum LocalKeyPurpose {
  /// 聊天正文、会话摘要等本地明文字段。
  chat('citizenapp.local/chat'),

  /// 聊天搜索的 HMAC 分词索引钥（只做 HMAC，不做加解密）。
  chatIndex('citizenapp.local/chat-index'),

  /// OpenMLS 状态信封（含设备签名私钥与群 ratchet 秘密）。
  mls('citizenapp.local/mls'),

  /// 聊天附件本地缓存文件。
  attachment('citizenapp.local/attachment'),

  /// 通讯录**本地** Isar KV（区别于上云的通讯录密文）。
  contactsLocal('citizenapp.local/contacts');

  const LocalKeyPurpose(this.domain);

  /// HKDF `info` 串，同时用作 AAD 前缀。
  final String domain;
}

/// 本地数据密钥（LDK）：一次生成、终身不变的 32 字节随机主钥。
///
/// 所有本地静止态子钥都从它派生，因此 **CID 换绑时只需用新账户重新 wrap 一次
/// LDK，已落盘的密文一个字节都不用重写**——否则换绑要把整个聊天历史、MLS 状态
/// 和全部附件重新加密一遍，手机上必然卡死或中断。
class LocalDataKey {
  const LocalDataKey(this.bytes);

  final Uint8List bytes;

  /// 按用途派生 32 字节子钥。LDK 本身已是随机主钥，故 salt 用固定域常量即可。
  Future<Uint8List> subkey(LocalKeyPurpose purpose) async {
    final key = await Hkdf(hmac: Hmac.sha256(), outputLength: 32).deriveKey(
      secretKey: SecretKey(bytes),
      nonce: _subkeySalt,
      info: utf8.encode(purpose.domain),
    );
    return Uint8List.fromList(await key.extractBytes());
  }

  static final List<int> _subkeySalt =
      utf8.encode('citizenapp.local.subkey.v1');
}

/// LDK 信封金库：用账户派生的 KEK 包裹 LDK 并持久化。
///
/// ```text
/// 账户 child mini-secret
///       │ HKDF(info="citizenapp.local/kek", salt=sha256(accountId))
///       ▼
///     KEK ── AES-256-GCM wrap/unwrap ──► LDK(32B 随机，终身不变)
///                                          │ HKDF(info=用途域)
///                                          ▼
///                                     五把用途子钥
/// ```
///
/// 换绑只重 wrap（O(1)），满足死契约 `cid-rebind-subkeys-must-auto-migrate`
/// 而不触发全盘重加密。
class LocalDataKeyVault {
  const LocalDataKeyVault(this._store);

  /// 密文 blob 落地层，由调用方注入（生产为 flutter_secure_storage，单测可注入内存实现）。
  final LocalKeyBlobStore _store;

  static const String _kekDomain = 'citizenapp.local/kek';
  static const String _wrapAadDomain = 'citizenapp.local/ldk';

  static String storageKeyFor(String accountId) =>
      'citizenapp_local_data_key_v1_$accountId';

  /// 确保该账户已有 LDK：没有则**新生成一把**并 wrap 落地；已有则原样返回。
  ///
  /// 幂等——重复调用不会换钥，否则会把已落盘的密文全部作废。
  Future<LocalDataKey> ensureForAccount({
    required String accountId,
    required List<int> accountSecret,
  }) async {
    final existing = await readForAccount(
      accountId: accountId,
      accountSecret: accountSecret,
    );
    if (existing != null) return existing;

    final ldk = LocalDataKey(LocalCipher.randomBytes(32));
    await _writeWrapped(accountId, accountSecret, ldk);
    return ldk;
  }

  /// 读取并解包该账户的 LDK；未建立过返回 null。
  ///
  /// 密钥不匹配或密文损坏会抛 [LocalCipherException]——**不降级为 null**，
  /// 否则会被上层误当成"尚未建立"而新生成一把，导致已有密文永久不可读。
  Future<LocalDataKey?> readForAccount({
    required String accountId,
    required List<int> accountSecret,
  }) async {
    final blob = await _store.read(storageKeyFor(accountId));
    if (blob == null || blob.isEmpty) return null;
    final kek = await _deriveKek(accountId, accountSecret);
    final bytes = await LocalCipher.decryptBytes(
      key: kek,
      blob: blob,
      aad: _wrapAad(accountId),
    );
    if (bytes.length != 32) {
      throw LocalCipherException('LDK 长度无效：期望 32 字节，实际 ${bytes.length}');
    }
    return LocalDataKey(bytes);
  }

  /// CID 换绑：用旧账户解包 LDK，再用新账户重新 wrap，最后删旧条目。
  ///
  /// **数据不动**——子钥由 LDK 派生，LDK 未变，已落盘密文继续可解。
  Future<LocalDataKey> rewrapForRebind({
    required String oldAccountId,
    required List<int> oldAccountSecret,
    required String newAccountId,
    required List<int> newAccountSecret,
  }) async {
    final ldk = await readForAccount(
      accountId: oldAccountId,
      accountSecret: oldAccountSecret,
    );
    if (ldk == null) {
      // 旧账户没建过 LDK（换绑前从未产生本地密文），直接给新账户建一把。
      return ensureForAccount(
        accountId: newAccountId,
        accountSecret: newAccountSecret,
      );
    }
    await _writeWrapped(newAccountId, newAccountSecret, ldk);
    if (newAccountId != oldAccountId) {
      await _store.delete(storageKeyFor(oldAccountId));
    }
    return ldk;
  }

  /// 用**已在手的** LDK 为指定账户 wrap 落地。
  ///
  /// 换绑时若 LDK 已从静默缓存取到，就不必再解一次旧账户（省一次生物识别），
  /// 直接用新账户 child wrap 即可。
  Future<void> writeForAccount({
    required String accountId,
    required List<int> accountSecret,
    required LocalDataKey ldk,
  }) =>
      _writeWrapped(accountId, accountSecret, ldk);

  /// 删除该账户的 LDK 条目（钱包删除 / 换绑清理旧账户用）。
  ///
  /// 注意：删掉后该账户名下的本地密文将永久不可读，调用方须自行确认数据已不再需要。
  Future<void> deleteForAccount(String accountId) =>
      _store.delete(storageKeyFor(accountId));

  Future<void> _writeWrapped(
    String accountId,
    List<int> accountSecret,
    LocalDataKey ldk,
  ) async {
    final kek = await _deriveKek(accountId, accountSecret);
    final wrapped = await LocalCipher.encryptBytes(
      key: kek,
      plaintext: ldk.bytes,
      aad: _wrapAad(accountId),
    );
    await _store.write(storageKeyFor(accountId), wrapped);
  }

  /// KEK = HKDF(账户 child mini-secret, salt=sha256(accountId), info=kek 域)。
  /// salt 取 accountId 哈希，与通讯录密钥派生保持同一套契约。
  static Future<Uint8List> _deriveKek(
    String accountId,
    List<int> accountSecret,
  ) async {
    final salt = (await Sha256().hash(utf8.encode(accountId))).bytes;
    final key = await Hkdf(hmac: Hmac.sha256(), outputLength: 32).deriveKey(
      secretKey: SecretKey(accountSecret),
      nonce: salt,
      info: utf8.encode(_kekDomain),
    );
    return Uint8List.fromList(await key.extractBytes());
  }

  static String _wrapAad(String accountId) => '$_wrapAadDomain|$accountId';
}

/// LDK 密文 blob 的持久化抽象（与安全存储解耦，便于单测注入内存实现）。
abstract interface class LocalKeyBlobStore {
  Future<String?> read(String key);
  Future<void> write(String key, String value);
  Future<void> delete(String key);
}
