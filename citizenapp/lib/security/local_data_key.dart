import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

/// 当前钱包账户派生的私有数据密钥用途域。
///
/// 每个用途独立派生；同一钱包账户在同一 CID 绑定版本下可跨设备得到相同子钥，
/// 换绑后的新账户或新 [AccountDataBinding.bindingRevision] 必然得到不同子钥。
enum LocalKeyPurpose {
  /// 聊天正文、会话摘要等本地明文字段。
  chat('citizenapp.account-data/chat'),

  /// 聊天搜索的 HMAC 分词索引钥（只做 HMAC，不做加解密）。
  chatIndex('citizenapp.account-data/chat-index'),

  /// OpenMLS 状态信封（含设备签名私钥与群 ratchet 秘密）。
  mls('citizenapp.account-data/mls'),

  /// 聊天附件本地缓存文件。
  attachment('citizenapp.account-data/attachment'),

  /// 通讯录本地 Isar KV。
  contactsLocal('citizenapp.account-data/contacts-local'),

  /// 通讯录云端端到端密文。
  contactsCloud('citizenapp.account-data/contacts-cloud');

  const LocalKeyPurpose(this.domain);

  /// HKDF `info` 主域，同时作为业务 AAD 的用途来源。
  final String domain;
}

/// 本机当前激活的 CID 钱包绑定元数据。
///
/// 这里只保存公开绑定事实，不保存任何数据密钥。真实数据访问确认缺钥时由当前账户
/// child 结合本结构派生用途钥并交给独立设备硬件钥封装；已有钥直接静默解封。正式换绑
/// 交接可在用户确认的交易作用域内显式读取旧、新账户材料。
class AccountDataBinding {
  const AccountDataBinding({
    required this.genesisHash,
    required this.cidNumber,
    required this.bindingRevision,
    required this.accountId,
  });

  final String genesisHash;
  final String cidNumber;
  final int bindingRevision;
  final String accountId;

  /// 校验链上绑定字段，防止调用方直接构造对象时绕过反序列化检查。
  void validate() {
    final cidBytes = utf8.encode(cidNumber);
    if (!_hashPattern.hasMatch(genesisHash)) {
      throw const AccountDataKeyException('创世哈希格式无效');
    }
    if (cidBytes.isEmpty || cidBytes.length > 32) {
      throw const AccountDataKeyException('cid_number 的 UTF-8 长度必须为 1-32 字节');
    }
    if (bindingRevision <= 0) {
      throw const AccountDataKeyException('CID 绑定版本必须大于 0');
    }
    if (!_accountIdPattern.hasMatch(accountId)) {
      throw const AccountDataKeyException('account_id 格式无效');
    }
  }

  Map<String, Object> toJson() => <String, Object>{
        'genesis_hash': genesisHash,
        'cid_number': cidNumber,
        'binding_revision': bindingRevision,
        'account_id': accountId,
      };

  static AccountDataBinding? fromJson(String raw) {
    try {
      final value = jsonDecode(raw);
      if (value is! Map<String, dynamic>) return null;
      final genesisHash = value['genesis_hash'];
      final cidNumber = value['cid_number'];
      final bindingRevision = value['binding_revision'];
      final accountId = value['account_id'];
      if (genesisHash is! String ||
          !_hashPattern.hasMatch(genesisHash) ||
          cidNumber is! String ||
          cidNumber.isEmpty ||
          utf8.encode(cidNumber).length > 32 ||
          bindingRevision is! int ||
          bindingRevision <= 0 ||
          accountId is! String ||
          !_accountIdPattern.hasMatch(accountId)) {
        return null;
      }
      return AccountDataBinding(
        genesisHash: genesisHash,
        cidNumber: cidNumber,
        bindingRevision: bindingRevision,
        accountId: accountId,
      );
    } on FormatException {
      return null;
    }
  }

  static final RegExp _hashPattern = RegExp(r'^0x[0-9a-f]{64}$');
  static final RegExp _accountIdPattern = RegExp(r'^0x[0-9a-f]{64}$');
}

/// 当前钱包绑定元数据存储。
///
/// 激活版本只能单调推进；同一版本出现不同 CID、账户或创世哈希时失败关闭。存储内容
/// 全是公开绑定字段，绝不形成额外用户私有数据主钥或领取凭证。
class AccountDataBindingStore {
  const AccountDataBindingStore(this._store);

  final LocalKeyBlobStore _store;

  static const String activeBindingKey =
      'citizenapp_account_data_active_binding';
  static const String pendingHandoverKey =
      'citizenapp_account_data_pending_handover';

  Future<AccountDataBinding?> readActiveBinding() async {
    final raw = await _store.read(activeBindingKey);
    if (raw == null || raw.isEmpty) return null;
    return AccountDataBinding.fromJson(raw);
  }

  Future<void> activate(AccountDataBinding next) async {
    next.validate();
    final previous = await readActiveBinding();
    if (previous != null) {
      if (previous.bindingRevision > next.bindingRevision) {
        throw const AccountDataKeyException('CID 本地绑定版本禁止回退');
      }
      if (previous.bindingRevision == next.bindingRevision &&
          (previous.genesisHash != next.genesisHash ||
              previous.cidNumber != next.cidNumber ||
              previous.accountId != next.accountId)) {
        throw const AccountDataKeyException('同一绑定版本的创世、CID 或账户不一致');
      }
    }
    await _store.write(activeBindingKey, jsonEncode(next.toJson()));
  }

  Future<void> clearActiveBinding() => _store.delete(activeBindingKey);

  Future<void> writePendingHandover({
    required AccountDataBinding source,
    required AccountDataBinding target,
  }) async {
    source.validate();
    target.validate();
    if (!_isValidHandover(source, target)) {
      throw const AccountDataKeyException('CID 钱包换绑交接上下文无效');
    }
    await _store.write(
      pendingHandoverKey,
      jsonEncode(<String, Object>{
        'source': source.toJson(),
        'target': target.toJson(),
      }),
    );
  }

  Future<({AccountDataBinding source, AccountDataBinding target})?>
      readPendingHandover() async {
    final raw = await _store.read(pendingHandoverKey);
    if (raw == null || raw.isEmpty) return null;
    try {
      final value = jsonDecode(raw);
      if (value is! Map<String, dynamic>) return null;
      final sourceRaw = value['source'];
      final targetRaw = value['target'];
      if (sourceRaw is! Map<String, dynamic> ||
          targetRaw is! Map<String, dynamic>) {
        return null;
      }
      final source = AccountDataBinding.fromJson(jsonEncode(sourceRaw));
      final target = AccountDataBinding.fromJson(jsonEncode(targetRaw));
      if (source == null || target == null) return null;
      if (!_isValidHandover(source, target)) {
        throw const AccountDataKeyException('CID 钱包换绑交接记录损坏');
      }
      return (source: source, target: target);
    } on FormatException {
      return null;
    }
  }

  Future<void> clearPendingHandover() => _store.delete(pendingHandoverKey);

  static bool _isValidHandover(
    AccountDataBinding source,
    AccountDataBinding target,
  ) =>
      source.genesisHash == target.genesisHash &&
      source.cidNumber == target.cidNumber &&
      target.bindingRevision == source.bindingRevision + 1 &&
      source.accountId != target.accountId;
}

/// 唯一私有数据密钥派生器。
///
/// 输入密钥只能是 CID 当前绑定钱包账户的 child mini-secret。创世、CID、绑定版本、
/// `account_id` 和用途共同参与 HKDF；因此同账户换设备可重建，同 CID 换绑到新账户后
/// 不能直接解密换绑前当前账户的历史私有密文。返回值只允许在内存中短期使用。
abstract final class AccountDataKeyDeriver {
  static const String _bindingDomain = 'citizenapp.account-data/binding';

  static Future<Uint8List> derive({
    required List<int> accountSecret,
    required AccountDataBinding binding,
    required LocalKeyPurpose purpose,
    String? context,
  }) async {
    binding.validate();
    if (accountSecret.length != 32) {
      throw AccountDataKeyException(
        '钱包账户私钥长度无效：期望 32 字节，实际 ${accountSecret.length}',
      );
    }
    final saltMaterial = utf8.encode(
      '$_bindingDomain|${binding.genesisHash}|${binding.cidNumber}|'
      '${binding.bindingRevision}|${binding.accountId}',
    );
    final salt = (await Sha256().hash(saltMaterial)).bytes;
    final info = context == null || context.isEmpty
        ? purpose.domain
        : '${purpose.domain}/$context';
    final key = await Hkdf(
      hmac: Hmac.sha256(),
      outputLength: 32,
    ).deriveKey(
      secretKey: SecretKey(accountSecret),
      nonce: salt,
      info: utf8.encode(info),
    );
    return Uint8List.fromList(await key.extractBytes());
  }
}

class AccountDataKeyException implements Exception {
  const AccountDataKeyException(this.message);

  final String message;

  @override
  String toString() => 'AccountDataKeyException: $message';
}

/// 钱包安全存储的最小字符串接口。这里只保存当前绑定公开元数据，不保存派生密钥。
abstract interface class LocalKeyBlobStore {
  Future<String?> read(String key);
  Future<void> write(String key, String value);
  Future<void> delete(String key);
}
