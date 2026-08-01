import 'dart:convert';

import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart';

import 'package:citizenapp/security/local_cipher.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 聊天本地静止态的**唯一**加解密边界。
///
/// `ChatStore` 之外的任何地方都不得直接接触密文或密钥：UI 与业务层拿到的始终是
/// 明文对象，落盘的始终是密文。密钥来自 [LocalKeyPurpose.chat]（正文/摘要）与
/// [LocalKeyPurpose.chatIndex]（搜索索引），二者域隔离、互不可解。
class ChatCrypto {
  ChatCrypto({WalletManager? walletManager})
      : _walletManager = walletManager ?? WalletManager();

  final WalletManager _walletManager;

  static final Hmac _hmac = Hmac.sha256();

  /// HMAC 截断长度（字节）。截断换取索引体积，代价是假阳性——由解密后复验兜住。
  static const int _tokenBytes = 8;

  /// 分词粒度：字符 bigram。中文无词边界，英文数字也要支持子串搜索，
  /// 统一用 bigram 两者通吃；查询短于 2 字符时由调用方回落到候选集扫描。
  static const int _gram = 2;

  /// 测试注入口：设为非空后直接使用固定用途子钥，不再触碰
  /// `WalletManager` → 硬件金库的平台通道。
  ///
  /// 与 `WalletManager.debugSeedStore` 同一套惯例。**仅测试可用**，
  /// 生产路径必须走真实钱包派生。
  @visibleForTesting
  static Map<LocalKeyPurpose, Uint8List>? debugFixedKeys;

  /// 解析聊天密文的 finalized 绑定元数据。CID 仍是唯一数据属主，绑定版本与账户
  /// 只用于判定当前钱包能否认证这条密文，绝不进入聊天关系主键。
  Future<ChatCipherBinding> resolveCipherBinding({
    required String ownerCidNumber,
    required String currentAccountId,
  }) async {
    if (debugFixedKeys != null) {
      return ChatCipherBinding(
        bindingRevision: 0,
        accountId: currentAccountId,
      );
    }
    final binding =
        await _walletManager.accountDataBindingForAccountId(currentAccountId);
    if (binding.cidNumber != ownerCidNumber) {
      throw StateError('聊天属主 CID 与当前钱包绑定不一致');
    }
    return ChatCipherBinding(
      bindingRevision: binding.bindingRevision,
      accountId: binding.accountId,
    );
  }

  Future<_ChatKeys> _keysFor({
    required String ownerCidNumber,
    required String currentAccountId,
  }) async {
    final fixed = debugFixedKeys;
    if (fixed != null) {
      final content = fixed[LocalKeyPurpose.chat];
      final index = fixed[LocalKeyPurpose.chatIndex];
      if (content == null || index == null) {
        throw StateError('聊天测试用途子钥不完整');
      }
      return _ChatKeys(
        content: Uint8List.fromList(content),
        index: Uint8List.fromList(index),
      );
    }
    final binding =
        await _walletManager.accountDataBindingForAccountId(currentAccountId);
    if (binding.cidNumber != ownerCidNumber) {
      throw StateError('聊天属主 CID 与当前钱包绑定不一致');
    }
    final keys = await _walletManager.deriveDataKeysForBinding(
      binding,
      const <({LocalKeyPurpose purpose, String? context})>[
        (purpose: LocalKeyPurpose.chat, context: null),
        (purpose: LocalKeyPurpose.chatIndex, context: null),
      ],
    );
    return _ChatKeys(content: keys[0], index: keys[1]);
  }

  /// 为一次钱包换绑交接显式派生当前或新绑定版本的聊天用途子钥。
  ///
  /// 不读取“当前激活账户”，因此能在交易提交前同时验证此前密文可解、新密钥可用；
  /// 返回值只在交接内存中使用，不能用于签名或恢复钱包。
  Future<ChatHandoverKeys> handoverKeys(AccountDataBinding binding) async {
    final keys = await _walletManager.deriveDataKeysForBinding(
      binding,
      const <({LocalKeyPurpose purpose, String? context})>[
        (purpose: LocalKeyPurpose.chat, context: null),
        (purpose: LocalKeyPurpose.chatIndex, context: null),
      ],
    );
    return ChatHandoverKeys(content: keys[0], index: keys[1]);
  }

  Future<String> decryptForHandover({
    required AccountDataBinding binding,
    required ChatHandoverKeys keys,
    required String recordId,
    required String blob,
  }) {
    if (blob.isEmpty) return Future<String>.value('');
    return LocalCipher.decryptString(
      key: keys.content,
      blob: blob,
      aad: _aad(binding.cidNumber, recordId),
    );
  }

  Future<String> encryptForHandover({
    required AccountDataBinding binding,
    required ChatHandoverKeys keys,
    required String recordId,
    required String plaintext,
  }) =>
      LocalCipher.encryptString(
        key: keys.content,
        plaintext: plaintext,
        aad: _aad(binding.cidNumber, recordId),
      );

  Future<List<String>> searchTokensForHandover({
    required ChatHandoverKeys keys,
    required String text,
  }) async {
    final out = <String>[];
    for (final gram in tokenize(text)) {
      out.add(await _tokenHash(keys.index, gram));
    }
    return out;
  }

  /// 加密聊天正文 / 会话摘要。[recordId] 进 AAD，把密文钉死在该条记录上。
  Future<String> encryptText({
    required String ownerCidNumber,
    required String currentAccountId,
    required String recordId,
    required String plaintext,
  }) async {
    final keys = await _keysFor(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    try {
      return await LocalCipher.encryptString(
        key: keys.content,
        plaintext: plaintext,
        aad: _aad(ownerCidNumber, recordId),
      );
    } finally {
      keys.dispose();
    }
  }

  /// 解密聊天正文 / 会话摘要。
  ///
  /// 空串代表"本来就没有正文"，直接返回空；真正的解密失败会抛
  /// [LocalCipherException]，**不静默降级**——否则用户会看到聊天记录凭空变空白。
  Future<String> decryptText({
    required String ownerCidNumber,
    required String currentAccountId,
    required String recordId,
    required String blob,
  }) async {
    if (blob.isEmpty) return '';
    final keys = await _keysFor(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    try {
      return await LocalCipher.decryptString(
        key: keys.content,
        blob: blob,
        aad: _aad(ownerCidNumber, recordId),
      );
    } finally {
      keys.dispose();
    }
  }

  /// 为一条正文生成去重后的 HMAC 分词索引。
  Future<List<String>> buildSearchTokens({
    required String ownerCidNumber,
    required String currentAccountId,
    required String text,
  }) async {
    final grams = tokenize(text);
    if (grams.isEmpty) return const <String>[];
    final keys = await _keysFor(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
    );
    try {
      final out = <String>[];
      for (final gram in grams) {
        out.add(await _tokenHash(keys.index, gram));
      }
      return out;
    } finally {
      keys.dispose();
    }
  }

  /// 把查询串转成索引 token；返回空表示查询过短，调用方须回落到候选集扫描。
  Future<List<String>> buildQueryTokens({
    required String ownerCidNumber,
    required String currentAccountId,
    required String query,
  }) =>
      buildSearchTokens(
        ownerCidNumber: ownerCidNumber,
        currentAccountId: currentAccountId,
        text: query,
      );

  /// 字符 bigram 切分：小写归一化后按滑动窗口取 2 字符，去重且保持稳定顺序。
  ///
  /// 用字符而非词：中文没有词边界，英文/数字也要能子串匹配，bigram 两者通吃。
  static List<String> tokenize(String text) {
    final normalized = text.trim().toLowerCase();
    if (normalized.isEmpty) return const <String>[];
    final runes = normalized.runes.toList(growable: false);
    if (runes.length < _gram) return const <String>[];
    final seen = <String>{};
    final out = <String>[];
    for (var i = 0; i + _gram <= runes.length; i += 1) {
      final gram = String.fromCharCodes(runes.sublist(i, i + _gram));
      if (seen.add(gram)) out.add(gram);
    }
    return out;
  }

  static Future<String> _tokenHash(List<int> indexKey, String gram) async {
    final mac = await _hmac.calculateMac(
      utf8.encode(gram),
      secretKey: SecretKey(indexKey),
    );
    final bytes = mac.bytes.sublist(0, _tokenBytes);
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  static String _aad(String ownerCidNumber, String recordId) =>
      'citizenapp.local/chat|$ownerCidNumber|$recordId';
}

/// 聊天密文的公开绑定上下文；只做密文隔离，不是身份或密钥。
class ChatCipherBinding {
  const ChatCipherBinding({
    required this.bindingRevision,
    required this.accountId,
  });

  final int bindingRevision;
  final String accountId;
}

class _ChatKeys {
  const _ChatKeys({required this.content, required this.index});

  final Uint8List content;
  final Uint8List index;

  void dispose() {
    content.fillRange(0, content.length, 0);
    index.fillRange(0, index.length, 0);
  }
}

/// CID 钱包换绑期间短时持有的聊天用途子钥；只允许此前密文解开后立即重加密。
class ChatHandoverKeys {
  const ChatHandoverKeys({required this.content, required this.index});

  final Uint8List content;
  final Uint8List index;

  void dispose() {
    content.fillRange(0, content.length, 0);
    index.fillRange(0, index.length, 0);
  }
}
