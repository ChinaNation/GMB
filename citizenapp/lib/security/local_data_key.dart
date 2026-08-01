import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

import 'package:citizenapp/security/local_cipher.dart';

/// 本地静止态密钥用途域。
///
/// 每个用途一把独立子钥，任一泄露不波及其他；本地通讯录域
/// (`citizenapp.cid/contacts-local`) 与云端通讯录域
/// (`citizenapp.cid/contacts-cloud`) 完全分开，云密文与本地密文不共钥。
enum LocalKeyPurpose {
  /// 聊天正文、会话摘要等本地明文字段。
  chat('citizenapp.cid/chat'),

  /// 聊天搜索的 HMAC 分词索引钥（只做 HMAC，不做加解密）。
  chatIndex('citizenapp.cid/chat-index'),

  /// OpenMLS 状态信封（含设备签名私钥与群 ratchet 秘密）。
  mls('citizenapp.cid/mls'),

  /// 聊天附件本地缓存文件。
  attachment('citizenapp.cid/attachment'),

  /// 通讯录**本地** Isar KV（区别于上云的通讯录密文）。
  contactsLocal('citizenapp.cid/contacts-local'),

  /// 通讯录云端端到端密文。
  contactsCloud('citizenapp.cid/contacts-cloud'),

  /// 本地草稿。
  drafts('citizenapp.cid/drafts');

  const LocalKeyPurpose(this.domain);

  /// HKDF `info` 串，同时用作 AAD 前缀。
  final String domain;
}

/// CID 稳定数据根：每个 CID 首次随机生成、换绑时不变的 32 字节主钥。
///
/// 数据根不从任何钱包助记词、账户 child 或设备密钥派生。否则旧钱包完全丢失后，
/// 新钱包只能得到另一把数据根，无法接管该 CID 的旧密文。当前绑定账户只负责链上
/// 签名授权和本机 KEK 包装；独立恢复层按 finalized 当前绑定向新账户发放**同一把**
/// 数据根，业务密文一律不动。
class CidDataRoot {
  const CidDataRoot(this.bytes);

  final Uint8List bytes;

  /// 按用途派生 32 字节子钥。数据根本身已是随机主钥，salt 只承担域隔离。
  Future<Uint8List> subkey(LocalKeyPurpose purpose) async {
    final key = await Hkdf(hmac: Hmac.sha256(), outputLength: 32).deriveKey(
      secretKey: SecretKey(bytes),
      nonce: _subkeySalt,
      info: utf8.encode(purpose.domain),
    );
    return Uint8List.fromList(await key.extractBytes());
  }

  static final List<int> _subkeySalt = utf8.encode('citizenapp.cid/subkey');
}

/// 已在本机激活的 CID 绑定快照。
class CidDataRootBinding {
  const CidDataRootBinding({
    required this.cidNumber,
    required this.bindingRevision,
    required this.accountId,
    required this.dataRootHash,
    required this.wrapperKey,
  });

  final String cidNumber;
  final int bindingRevision;
  final String accountId;
  final String dataRootHash;
  final String wrapperKey;

  Map<String, Object> toJson() => <String, Object>{
        'cid_number': cidNumber,
        'binding_revision': bindingRevision,
        'account_id': accountId,
        'data_root_hash': dataRootHash,
        'wrapper_key': wrapperKey,
      };

  static CidDataRootBinding? fromJson(String raw) {
    try {
      final value = jsonDecode(raw);
      if (value is! Map<String, dynamic>) return null;
      final cidNumber = value['cid_number'];
      final bindingRevision = value['binding_revision'];
      final accountId = value['account_id'];
      final dataRootHash = value['data_root_hash'];
      final wrapperKey = value['wrapper_key'];
      if (cidNumber is! String ||
          cidNumber.isEmpty ||
          bindingRevision is! int ||
          bindingRevision <= 0 ||
          accountId is! String ||
          !RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId) ||
          dataRootHash is! String ||
          !RegExp(r'^[0-9a-f]{64}$').hasMatch(dataRootHash) ||
          wrapperKey is! String ||
          wrapperKey.isEmpty) {
        return null;
      }
      return CidDataRootBinding(
        cidNumber: cidNumber,
        bindingRevision: bindingRevision,
        accountId: accountId,
        dataRootHash: dataRootHash,
        wrapperKey: wrapperKey,
      );
    } catch (_) {
      return null;
    }
  }
}

/// CID 数据根信封金库：当前绑定账户用自己的 child 派生 KEK 包装数据根。
///
/// ```text
/// 当前绑定账户 child mini-secret
///       │ HKDF(info="citizenapp.cid-data-root/kek",
///       │      salt=sha256(cid|revision|account))
///       ▼
///     KEK ── AES-256-GCM wrap/unwrap ──► CID 数据根(32B，换绑不变)
///                                          │ HKDF(info=用途域)
///                                          ▼
///                                     各业务用途子钥
/// ```
///
/// 本金库只保存当前账户可解开的本地包装。数据根真源属于 CID，由独立恢复层密封；
/// 只有 finalized 当前绑定账户通过一次性签名挑战才能取得。恢复响应另用一次性 X25519
/// 会话钥加密，HTTP JSON 不出现明文数据根。
///
/// 写新包装、读回验证、落激活标记之后才删低版本包装；换绑只重新包装这 32 字节，
/// 业务密文一律不动。
class CidDataRootVault {
  const CidDataRootVault(this._store);

  /// 密文 blob 落地层，由调用方注入（生产为 flutter_secure_storage，单测可注入内存实现）。
  final LocalKeyBlobStore _store;

  static const String _kekDomain = 'citizenapp.cid-data-root/kek';
  static const String _wrapAadDomain = 'citizenapp.cid-data-root/wrapper';
  static const String activeBindingKey =
      'citizenapp_cid_data_root_active_binding';

  static String wrapperKeyFor({
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
  }) =>
      'citizenapp_cid_data_root_wrapper_${Uri.encodeComponent(cidNumber)}_'
      '${bindingRevision}_$accountId';

  /// 用当前绑定账户的 child 包装并缓存 CID 数据根。
  ///
  /// [expectedDataRootHash] 来自当前绑定恢复授权；本机先核验数据根摘要，再以同一摘要
  /// 验证新包装读回，防止响应、绑定参数或安全存储落地错配。
  ///
  /// 新包装读回验证成功并写入激活标记后，才删除此前版本的包装；清理失败不回滚已激活的新绑定。
  Future<CidDataRoot> installForCurrentBinding({
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
    required List<int> accountSecret,
    required CidDataRoot dataRoot,
    required String expectedDataRootHash,
  }) async {
    _validateBinding(cidNumber, bindingRevision, accountId);
    final actualHash = await dataRootHash(dataRoot);
    if (actualHash != expectedDataRootHash) {
      throw const LocalCipherException('CID 数据根摘要自校验不一致');
    }
    final previous = await readActiveBinding();
    if (previous != null) {
      if (previous.bindingRevision > bindingRevision) {
        throw const LocalCipherException('CID 本地绑定版本禁止回退');
      }
      if (previous.bindingRevision == bindingRevision &&
          (previous.cidNumber != cidNumber ||
              previous.accountId != accountId ||
              previous.dataRootHash != expectedDataRootHash)) {
        throw const LocalCipherException('同一绑定版本的 CID、账户或数据根不一致');
      }
    }
    final wrapperKey = wrapperKeyFor(
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
    );
    await _writeWrapped(
      wrapperKey: wrapperKey,
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
      accountSecret: accountSecret,
      dataRoot: dataRoot,
    );
    final verified = await _readWrapped(
      wrapperKey: wrapperKey,
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
      accountSecret: accountSecret,
    );
    if (await dataRootHash(verified) != expectedDataRootHash) {
      throw const LocalCipherException('新账户数据根包装读回验证失败');
    }
    final next = CidDataRootBinding(
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
      dataRootHash: expectedDataRootHash,
      wrapperKey: wrapperKey,
    );
    await _store.write(activeBindingKey, jsonEncode(next.toJson()));
    if (previous != null && previous.wrapperKey != wrapperKey) {
      try {
        await _store.delete(previous.wrapperKey);
      } catch (_) {
        // 新绑定已经验证并激活；低版本包装清理失败只能重试，不能回滚控制权。
      }
    }
    return verified;
  }

  /// 读取当前精确绑定的数据根。没有激活标记或三元组不匹配时失败关闭。
  Future<CidDataRoot> readForCurrentBinding({
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
    required List<int> accountSecret,
  }) async {
    final active = await readActiveBinding();
    if (active == null ||
        active.cidNumber != cidNumber ||
        active.bindingRevision != bindingRevision ||
        active.accountId != accountId) {
      throw const LocalCipherException('当前 CID 绑定尚未完成数据根接管');
    }
    final root = await _readWrapped(
      wrapperKey: active.wrapperKey,
      cidNumber: cidNumber,
      bindingRevision: bindingRevision,
      accountId: accountId,
      accountSecret: accountSecret,
    );
    if (await dataRootHash(root) != active.dataRootHash) {
      throw const LocalCipherException('当前 CID 数据根摘要校验失败');
    }
    return root;
  }

  Future<CidDataRootBinding?> readActiveBinding() async {
    final raw = await _store.read(activeBindingKey);
    if (raw == null || raw.isEmpty) return null;
    return CidDataRootBinding.fromJson(raw);
  }

  Future<void> clearActiveBinding() async {
    final active = await readActiveBinding();
    if (active != null) await _store.delete(active.wrapperKey);
    await _store.delete(activeBindingKey);
  }

  Future<void> _writeWrapped({
    required String wrapperKey,
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
    required List<int> accountSecret,
    required CidDataRoot dataRoot,
  }) async {
    final kek = await _deriveKek(
      cidNumber,
      bindingRevision,
      accountId,
      accountSecret,
    );
    final wrapped = await LocalCipher.encryptBytes(
      key: kek,
      plaintext: dataRoot.bytes,
      aad: _wrapAad(cidNumber, bindingRevision, accountId),
    );
    await _store.write(wrapperKey, wrapped);
  }

  Future<CidDataRoot> _readWrapped({
    required String wrapperKey,
    required String cidNumber,
    required int bindingRevision,
    required String accountId,
    required List<int> accountSecret,
  }) async {
    final blob = await _store.read(wrapperKey);
    if (blob == null || blob.isEmpty) {
      throw const LocalCipherException('当前 CID 数据根包装不存在');
    }
    final kek = await _deriveKek(
      cidNumber,
      bindingRevision,
      accountId,
      accountSecret,
    );
    final bytes = await LocalCipher.decryptBytes(
      key: kek,
      blob: blob,
      aad: _wrapAad(cidNumber, bindingRevision, accountId),
    );
    if (bytes.length != 32) {
      throw LocalCipherException(
        'CID 数据根长度无效：期望 32 字节，实际 ${bytes.length}',
      );
    }
    return CidDataRoot(bytes);
  }

  static Future<Uint8List> _deriveKek(
    String cidNumber,
    int bindingRevision,
    String accountId,
    List<int> accountSecret,
  ) async {
    final salt = (await Sha256().hash(
      utf8.encode('$cidNumber|$bindingRevision|$accountId'),
    ))
        .bytes;
    final key = await Hkdf(hmac: Hmac.sha256(), outputLength: 32).deriveKey(
      secretKey: SecretKey(accountSecret),
      nonce: salt,
      info: utf8.encode(_kekDomain),
    );
    return Uint8List.fromList(await key.extractBytes());
  }

  static String _wrapAad(
    String cidNumber,
    int bindingRevision,
    String accountId,
  ) =>
      '$_wrapAadDomain|$cidNumber|$bindingRevision|$accountId';

  static Future<String> dataRootHash(CidDataRoot dataRoot) async {
    final digest = await Sha256().hash(dataRoot.bytes);
    return digest.bytes
        .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
        .join();
  }

  static void _validateBinding(
    String cidNumber,
    int bindingRevision,
    String accountId,
  ) {
    if (cidNumber.trim().isEmpty || utf8.encode(cidNumber).length > 32) {
      throw ArgumentError.value(cidNumber, 'cidNumber', 'CID 长度必须为 1 到 32 字节');
    }
    if (bindingRevision <= 0) {
      throw ArgumentError.value(
        bindingRevision,
        'bindingRevision',
        '绑定版本必须为正整数',
      );
    }
    if (!RegExp(r'^0x[0-9a-f]{64}$').hasMatch(accountId)) {
      throw ArgumentError.value(accountId, 'accountId', '账户标识格式不合法');
    }
  }
}

/// CID 数据根密文 blob 的持久化抽象。
abstract interface class LocalKeyBlobStore {
  Future<String?> read(String key);
  Future<void> write(String key, String value);
  Future<void> delete(String key);
}
