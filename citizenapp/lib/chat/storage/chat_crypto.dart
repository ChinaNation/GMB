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

  /// 按 accountId 缓存已派生子钥，避免每条消息都重复走一遍 HKDF。
  final Map<String, _ChatKeys> _cache = <String, _ChatKeys>{};

  static final Hmac _hmac = Hmac.sha256();

  /// HMAC 截断长度（字节）。截断换取索引体积，代价是假阳性——由解密后复验兜住。
  static const int _tokenBytes = 8;

  /// 分词粒度：字符 bigram。中文无词边界，英文数字也要支持子串搜索，
  /// 统一用 bigram 两者通吃；查询短于 2 字符时由调用方回落到候选集扫描。
  static const int _gram = 2;

  /// 测试注入口：设为非空后，子钥直接由该固定 LDK 派生，不再触碰
  /// `WalletManager` → 硬件金库 → `flutter_secure_storage` 的平台通道。
  ///
  /// 与 `WalletManager.debugSeedStore` 同一套惯例。**仅测试可用**，
  /// 生产路径必须走真实钱包派生。
  @visibleForTesting
  static LocalDataKey? debugFixedLocalDataKey;

  Future<_ChatKeys> _keysFor(String accountId) async {
    final cached = _cache[accountId];
    if (cached != null) return cached;
    final ldk = debugFixedLocalDataKey ??
        await _walletManager.ensureLocalDataKeyForAccountId(accountId);
    final keys = _ChatKeys(
      content: await ldk.subkey(LocalKeyPurpose.chat),
      index: await ldk.subkey(LocalKeyPurpose.chatIndex),
    );
    _cache[accountId] = keys;
    return keys;
  }

  // 说明：不提供 evict——缓存按 accountId 分键，且 CID 换绑只重 wrap、LDK 与
  // 五把子钥都不变，不存在陈旧子钥问题。加一个永不调用的清理入口只会变成残桩。

  /// 加密聊天正文 / 会话摘要。[recordId] 进 AAD，把密文钉死在该条记录上。
  Future<String> encryptText({
    required String accountId,
    required String recordId,
    required String plaintext,
  }) async {
    final keys = await _keysFor(accountId);
    return LocalCipher.encryptString(
      key: keys.content,
      plaintext: plaintext,
      aad: _aad(recordId),
    );
  }

  /// 解密聊天正文 / 会话摘要。
  ///
  /// 空串代表"本来就没有正文"，直接返回空；真正的解密失败会抛
  /// [LocalCipherException]，**不静默降级**——否则用户会看到聊天记录凭空变空白。
  Future<String> decryptText({
    required String accountId,
    required String recordId,
    required String blob,
  }) async {
    if (blob.isEmpty) return '';
    final keys = await _keysFor(accountId);
    return LocalCipher.decryptString(
      key: keys.content,
      blob: blob,
      aad: _aad(recordId),
    );
  }

  /// 为一条正文生成去重后的 HMAC 分词索引。
  Future<List<String>> buildSearchTokens({
    required String accountId,
    required String text,
  }) async {
    final grams = tokenize(text);
    if (grams.isEmpty) return const <String>[];
    final keys = await _keysFor(accountId);
    final out = <String>[];
    for (final gram in grams) {
      out.add(await _tokenHash(keys.index, gram));
    }
    return out;
  }

  /// 把查询串转成索引 token；返回空表示查询过短，调用方须回落到候选集扫描。
  Future<List<String>> buildQueryTokens({
    required String accountId,
    required String query,
  }) =>
      buildSearchTokens(accountId: accountId, text: query);

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

  static String _aad(String recordId) => 'citizenapp.local/chat|$recordId';
}

class _ChatKeys {
  const _ChatKeys({required this.content, required this.index});

  final Uint8List content;
  final Uint8List index;
}
