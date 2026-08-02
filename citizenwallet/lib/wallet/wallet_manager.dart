import 'dart:convert';
import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter/foundation.dart' show kReleaseMode, visibleForTesting;
import 'package:isar_community/isar.dart';
import 'package:local_auth/local_auth.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenwallet/chain/chain_constants.dart';
import 'package:citizenwallet/isar/wallet_isar.dart';
import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/security/secure_storage.dart';
import 'package:citizenwallet/signer/qr_signer.dart';
import 'package:citizenwallet/wallet/secret_cipher.dart';
import 'package:citizenwallet/wallet/wallet_secure_keys.dart';

/// 钱包（master）：一套助记词 = 一个种子 = 一个 master，其下派生多个账户。
class Wallet {
  const Wallet({
    required this.walletIndex,
    required this.walletName,
    required this.masterId,
    required this.createdAtMillis,
    required this.source,
    this.sortOrder = 0,
  });

  final int walletIndex;
  final String walletName;

  /// 主指纹 = 账户0（`//0`）的 accountId，唯一标识一套助记词。
  final String masterId;
  final int createdAtMillis;
  final String source;

  final int sortOrder;
}

/// 账户：钱包下按派生序号展开的一对公私钥。
/// 全部 `//index` 硬派生（含账户0 = `//0`，无 bare 根）；每账户密钥独立、单向。
class Account {
  const Account({
    required this.masterId,
    required this.accountIndex,
    required this.accountId,
    required this.ss58Address,
    required this.accountName,
    required this.createdAtMillis,
  });

  final String masterId;
  final int accountIndex;

  /// 小写 `0x` + 64 位十六进制（= 派生公钥原字节）。
  final String accountId;
  final String ss58Address;
  final String accountName;
  final int createdAtMillis;

  /// 展示用派生路径：`//index`（含账户0 = `//0`）。
  String get derivationPath => '//$accountIndex';
}

class WalletCreationResult {
  const WalletCreationResult({
    required this.wallet,
    required this.primaryAccount,
    required this.mnemonic,
  });

  final Wallet wallet;

  /// 账户0（`//0`），创建/导入后即存在。
  final Account primaryAccount;

  /// 助记词（创建时一次性展示；同时按钱包加密存储，供钱包详情备份查看）。
  final String mnemonic;
}

class WalletSignResult {
  const WalletSignResult({
    required this.signerPublicKey,
    required this.alg,
    required this.signatureHex,
  });

  final String signerPublicKey;
  final String alg;
  final String signatureHex;
}

/// 登录 QR 签名原文中的一次性请求边界。
///
/// 登录页面保持任务前 UI 不变；一次性占位与墙钟校验下沉到唯一的 UTF-8
/// 签名入口，确保生物识别和私钥调用前已经拒绝过期或重复请求。
class _LoginSignatureClaim {
  const _LoginSignatureClaim({
    required this.requestId,
    required this.expiresAt,
  });

  final String requestId;
  final int expiresAt;
}

class WalletAuthException implements Exception {
  const WalletAuthException(this.message);
  final String message;

  @override
  String toString() => 'WalletAuthException: $message';
}

/// 钱包管理（HD model B：一套助记词 → 多账户，全 `//index` 硬派生，无 bare 根）。
///
/// **冷钱包存种子**：本设备是冷签保管方，按钱包(master)加密存 master 种子 +
/// 助记词（`SecretCipher` AES-256-GCM），做冷签与备份。账户不单独持久化密钥：
/// 签名 / 私钥导出时按 accountIndex 从种子**现场派生**、用后即弃。每账户 child
/// mini-secret 单向硬派生，导出单账户私钥只暴露该账户；助记词是钱包级根备份。
///
/// 派生金标（冷热共享单源）：`test/wallet/derivation_golden_test.dart`，钉死
/// `fromSeed(childMiniSecret) == <助记词>//index` 逐字节相等。
class WalletManager {
  static const int _ss58Prefix = ChainConstants.ss58Prefix;
  static final RegExp _accountIdPattern = RegExp(r'^0x[0-9a-f]{64}$');
  static final RegExp _seedHexPattern = RegExp(r'^[0-9a-fA-F]{64}$');
  // 单源加固实例(选项集中在 security/secure_storage.dart)。
  static const _secureStorage = appSecureStorage;
  static final LocalAuthentication _localAuth = LocalAuthentication();

  /// 生物识别门禁（读密钥 / 签名前调用）。测试可经 [debugAuthGate] 注入。
  static Future<void> Function() _authGate = _defaultAuthGate;

  /// 仅供测试注入放行门禁;**release 构建下彻底无效**(拒绝任何注入),杜绝生产
  /// 环境经此绕过生物识别——门禁恒为真实强制生物识别。
  @visibleForTesting
  static set debugAuthGate(Future<void> Function()? gate) {
    if (kReleaseMode) return;
    _authGate = gate ?? _defaultAuthGate;
  }

  // ── 查询 ──
  Future<List<Wallet>> getWallets() async {
    final isar = await WalletIsar.instance.db();
    final rows = await isar.walletEntitys.where().sortBySortOrder().findAll();
    return rows.map(_toWallet).toList();
  }

  Future<Wallet?> getWalletByMasterId(String masterId) async {
    final isar = await WalletIsar.instance.db();
    final row =
        await isar.walletEntitys.filter().masterIdEqualTo(masterId).findFirst();
    return row == null ? null : _toWallet(row);
  }

  /// 某钱包下全部账户，按 accountIndex 升序。
  Future<List<Account>> getAccounts(String masterId) async {
    final isar = await WalletIsar.instance.db();
    final rows = await isar.accountEntitys
        .filter()
        .masterIdEqualTo(masterId)
        .sortByAccountIndex()
        .findAll();
    return rows.map(_toAccount).toList();
  }

  Future<Account?> getAccountByAccountId(String accountId) async {
    final isar = await WalletIsar.instance.db();
    final row = await isar.accountEntitys
        .filter()
        .accountIdEqualTo(accountId)
        .findFirst();
    return row == null ? null : _toAccount(row);
  }

  // ── 创建 / 导入 / 加账户 ──
  /// 新建钱包：生成助记词 → 派生账户0（`//0`）→ 存 master 种子 + 助记词。
  ///
  /// [wordCount] 助记词个数，12（默认）或 24。助记词同时一次性返回展示。
  Future<WalletCreationResult> createWallet({int wordCount = 12}) async {
    assert(wordCount == 12 || wordCount == 24);
    final mnemonic = bip39m.Mnemonic.generate(
      bip39m.Language.english,
      length: wordCount == 24
          ? bip39m.MnemonicLength.words24
          : bip39m.MnemonicLength.words12,
    ).sentence;
    return _establishWallet(mnemonic, 'created');
  }

  /// 导入钱包：校验助记词 → 派生账户0（`//0`）→ 存 master 种子 + 助记词。
  Future<WalletCreationResult> importWallet(String mnemonic) async {
    final trimmed = mnemonic.trim();
    if (!_isValidMnemonic(trimmed)) {
      throw Exception('助记词无效，请检查拼写和空格');
    }
    return _establishWallet(trimmed, 'imported');
  }

  Future<WalletCreationResult> _establishWallet(
    String mnemonic,
    String source,
  ) async {
    // 冷签设备:创建/导入钱包(落密钥)前强制设备认证;无锁则 fail-closed。
    await _authGate();
    final seed = await _mnemonicToSeed(mnemonic);
    final seedHex = _toHex(seed);
    try {
      // 账户0 = //0（无 bare 根）。其 accountId 即 master 指纹。
      final acct0 = _deriveAccount(seed, 0);
      final masterId = acct0.accountId;

      final result = await _appendWalletAtomic(
        masterId: masterId,
        source: source,
        primary: acct0,
      );
      try {
        await _writeMasterSeed(masterId, seedHex);
        await _writeMasterMnemonic(masterId, mnemonic);
      } catch (_) {
        // 种子/助记词任一写失败即回滚已提交的 Isar 行 + 已写入的种子,避免留下
        // "能签名但助记词备份永久丢失"的半成品钱包。回滚尽力而为,不掩盖原异常。
        try {
          await _deleteWalletInternal(masterId);
        } catch (_) {}
        rethrow;
      }
      return WalletCreationResult(
        wallet: result.$1,
        primaryAccount: result.$2,
        mnemonic: mnemonic,
      );
    } finally {
      _zeroList(seed);
    }
  }

  /// 账户序号上界（`//index` 的 index 最大值;账户0 为创建时主账户）。
  static const int maxAccountIndex = 1989;

  /// 在指定钱包下新增账户：读存储种子，派生 `//index`（不产生新助记词）。
  ///
  /// [index] 为空 = 添加"下一个"(max+1);非空 = 指定序号(`1..maxAccountIndex`,
  /// 用于恢复非连续账户 / 加别处已注资的特定账户)。序号可非连续。校验先于读种子。
  Future<Account> addAccount(String masterId, {int? index}) async {
    await _authGate();
    final isar = await WalletIsar.instance.db();
    final wallet =
        await isar.walletEntitys.filter().masterIdEqualTo(masterId).findFirst();
    if (wallet == null) {
      throw const WalletAuthException('未找到钱包');
    }
    final seedHex = await _readMasterSeedRaw(masterId);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }
    final seed = _hexToBytes(seedHex);
    try {
      late AccountEntity entity;
      await isar.writeTxn(() async {
        final accounts = await isar.accountEntitys
            .filter()
            .masterIdEqualTo(masterId)
            .findAll();
        final existing = accounts.map((e) => e.accountIndex).toSet();
        final int targetIndex;
        if (index == null) {
          final maxIndex = existing.fold<int>(-1, (m, i) => i > m ? i : m);
          targetIndex = maxIndex + 1;
          if (targetIndex > maxAccountIndex) {
            throw const WalletAuthException('已达账户序号上限 $maxAccountIndex');
          }
        } else {
          if (index < 1 || index > maxAccountIndex) {
            throw const WalletAuthException('账户序号需在 1–$maxAccountIndex');
          }
          if (existing.contains(index)) {
            throw WalletAuthException('账户$index 已存在');
          }
          targetIndex = index;
        }

        final derived = _deriveAccount(seed, targetIndex);
        entity = AccountEntity()
          ..masterId = masterId
          ..accountIndex = targetIndex
          ..accountId = derived.accountId
          ..ss58Address = derived.ss58Address
          ..accountName = '账户$targetIndex'
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.accountEntitys.put(entity);
      });
      return _toAccount(entity);
    } finally {
      _zeroList(seed);
    }
  }

  // ── 删除 ──
  /// 删除整只钱包：连带其全部账户 + master 种子/助记词。删密钥前强制认证。
  Future<void> deleteWallet(String masterId) async {
    await _authGate();
    await _deleteWalletInternal(masterId);
  }

  /// 无认证的删钱包实现（供 deleteWallet 与 deleteAccount 级联复用，避免二次弹窗）。
  ///
  /// 先删除并回读确认根机密，再删事实行；任何 SecureStorage 异常都显式失败。
  Future<void> _deleteWalletInternal(String masterId) async {
    await _deleteMasterSeed(masterId);
    await _deleteMasterMnemonic(masterId);
    if (await _secureStorage.read(
              key: WalletSecureKeys.masterSeedHexV1(masterId),
            ) !=
            null ||
        await _secureStorage.read(
              key: WalletSecureKeys.masterMnemonicV1(masterId),
            ) !=
            null) {
      throw const WalletAuthException('钱包机密未能彻底删除，请重试');
    }

    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.accountEntitys.filter().masterIdEqualTo(masterId).deleteAll();
      final wallet = await isar.walletEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .findFirst();
      if (wallet != null) {
        await isar.walletEntitys.delete(wallet.id);
      }
    });
    final walletCount = await isar.walletEntitys.count();
    if (walletCount == 0) {
      await SecretCipher.deleteAekIfNoWalletSecrets();
    }
  }

  /// 删除单个账户；若删空该钱包全部账户则连带删钱包与密钥。删前强制认证。
  ///
  /// 账户0 是 master 锚点:尚有兄弟账户时禁止单独删账户0(否则 masterId 悬空、
  /// addAccount 只 max+1 无法重建、重导入又被查重挡住),须改删整只钱包。
  Future<void> deleteAccount(String accountId) async {
    await _authGate();
    final isar = await WalletIsar.instance.db();
    late String masterId;
    var deleteWallet = false;
    await isar.writeTxn(() async {
      // 查找、计数和删除必须处于同一事务，避免并发删账户时留下无账户钱包。
      final acct = await isar.accountEntitys
          .filter()
          .accountIdEqualTo(accountId)
          .findFirst();
      if (acct == null) {
        throw Exception('未找到账户');
      }
      masterId = acct.masterId;
      final accountCount =
          await isar.accountEntitys.filter().masterIdEqualTo(masterId).count();
      if (acct.accountIndex == 0 && accountCount > 1) {
        throw Exception('账户0是钱包锚点,请删除整个钱包');
      }
      deleteWallet = accountCount == 1;
      if (!deleteWallet) {
        await isar.accountEntitys.delete(acct.id);
      }
    });
    if (deleteWallet) {
      await _deleteWalletInternal(masterId);
    }
  }

  // ── 更新 ──
  static const int maxWalletNameLength = 5;

  // 寻址单源用 masterId(稳定主键;walletIndex 是可复用槽位,删钱包后会被
  // 新钱包重占,拿它定位有指向另一只钱包的窗口)。删除/查账户/签名已按 masterId,
  // 改名/重排同口径。
  Future<void> renameWallet(String masterId, String walletName) async {
    final nextName = walletName.trim();
    if (nextName.isEmpty) {
      throw Exception('钱包名称不能为空');
    }
    if (nextName.runes.length > maxWalletNameLength) {
      throw Exception('钱包名称最多$maxWalletNameLength个字');
    }
    final isar = await WalletIsar.instance.db();
    final row =
        await isar.walletEntitys.filter().masterIdEqualTo(masterId).findFirst();
    if (row == null) {
      throw Exception('未找到钱包');
    }
    await isar.writeTxn(() async {
      row.walletName = nextName;
      await isar.walletEntitys.put(row);
    });
  }

  static const int maxAccountNameLength = 5;

  /// 重命名账户(仅改显示名,不动任何密钥)。按 accountId 定位。
  Future<void> renameAccount(String accountId, String accountName) async {
    final nextName = accountName.trim();
    if (nextName.isEmpty) {
      throw Exception('账户名称不能为空');
    }
    if (nextName.runes.length > maxAccountNameLength) {
      throw Exception('账户名称最多$maxAccountNameLength个字');
    }
    final isar = await WalletIsar.instance.db();
    final row = await isar.accountEntitys
        .filter()
        .accountIdEqualTo(accountId)
        .findFirst();
    if (row == null) {
      throw Exception('未找到账户');
    }
    await isar.writeTxn(() async {
      row.accountName = nextName;
      await isar.accountEntitys.put(row);
    });
  }

  /// 批量更新钱包排序。[masterIds] 顺序即新 sortOrder。
  Future<void> reorderWallets(List<String> masterIds) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      for (var i = 0; i < masterIds.length; i++) {
        final row = await isar.walletEntitys
            .filter()
            .masterIdEqualTo(masterIds[i])
            .findFirst();
        if (row != null) {
          row.sortOrder = i;
          await isar.walletEntitys.put(row);
        }
      }
    });
  }

  // ── 签名（按账户；读种子现场派生，签名完成立即清零可控密钥缓冲）──
  Future<Uint8List> signForAccount(
    String accountId,
    Uint8List payload,
  ) =>
      _signWithAccount(accountId, payload);

  Future<WalletSignResult> signUtf8ForAccount(
    String accountId,
    String message,
  ) async {
    final loginClaim = _parseLoginSignatureClaim(
      accountId: accountId,
      message: message,
    );
    if (loginClaim != null) {
      final claimed = await SignedQrRequestStore.claim(
        requestId: loginClaim.requestId,
        expiresAt: loginClaim.expiresAt,
      );
      if (!claimed) {
        throw const WalletAuthException('该登录请求已处理或已过期，请重新扫描');
      }
    }

    final messageBytes = Uint8List.fromList(utf8.encode(message));
    try {
      final signature = await _signWithAccount(
        accountId,
        messageBytes,
        expiresAt: loginClaim?.expiresAt,
      );
      return WalletSignResult(
        signerPublicKey: accountId,
        alg: 'sr25519',
        signatureHex: '0x${_toHex(signature.toList(growable: false))}',
      );
    } catch (_) {
      if (loginClaim != null) {
        await SignedQrRequestStore.release(loginClaim.requestId);
      }
      rethrow;
    } finally {
      messageBytes.fillRange(0, messageBytes.length, 0);
    }
  }

  /// 定位账户 → 读种子 → 派生 → 校验公钥 → 签名；所有可控私钥缓冲在本方法内清零。
  Future<Uint8List> _signWithAccount(
    String accountId,
    Uint8List payload, {
    int? expiresAt,
  }) async {
    await _authGate();
    // 生物识别可能跨越到期边界；在读取根机密前按当前墙钟再次校验。
    if (expiresAt != null &&
        expiresAt <= DateTime.now().millisecondsSinceEpoch ~/ 1000) {
      throw const WalletAuthException('登录请求已过期，请重新扫描');
    }
    final isar = await WalletIsar.instance.db();
    final acct = await isar.accountEntitys
        .filter()
        .accountIdEqualTo(accountId)
        .findFirst();
    if (acct == null) {
      throw const WalletAuthException('未找到指定账户');
    }
    final seedHex = await _readMasterSeedRaw(acct.masterId);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }
    final seed = _hexToBytes(seedHex);
    try {
      final child = _childMiniSecret(seed, acct.accountIndex);
      final childBytes = Uint8List.fromList(child);
      final miniSecret = sr.MiniSecretKey.fromRawKey(childBytes);
      final secretKey = miniSecret.expandEd25519();
      try {
        final localAccountId = _accountIdFromBytes(secretKey.public().encode());
        if (localAccountId != acct.accountId) {
          throw const WalletAuthException('本地签名密钥与账户不一致，请重新导入钱包');
        }
        return sr.Sr25519.sign(secretKey, payload).encode();
      } finally {
        childBytes.fillRange(0, childBytes.length, 0);
        _zeroList(miniSecret.key);
        _zeroSecretKey(secretKey);
      }
    } finally {
      _zeroList(seed);
    }
  }

  /// 识别登录 QR 的规范签名原文；普通非 QR 文本签名保持原有行为。
  ///
  /// 格式由 `buildSignatureMessage` 唯一生成：
  /// `QR_V1|2|<request_id>|onchina|<expires_at>|<account_id_without_0x>`。
  _LoginSignatureClaim? _parseLoginSignatureClaim({
    required String accountId,
    required String message,
  }) {
    if (!message.startsWith('${QrProtocols.qrV1}|')) {
      return null;
    }
    if (!_accountIdPattern.hasMatch(accountId)) {
      throw const WalletAuthException('登录签名账户格式无效');
    }
    final parts = message.split('|');
    final expiresAt = parts.length == 6 ? int.tryParse(parts[4]) : null;
    final valid = parts.length == 6 &&
        parts[0] == QrProtocols.qrV1 &&
        parts[1] == QrKind.signResponse.code.toString() &&
        QrSigner.isValidRequestId(parts[2]) &&
        parts[3] == 'onchina' &&
        expiresAt != null &&
        parts[5] == accountId.substring(2);
    if (!valid) {
      throw const WalletAuthException('登录签名请求格式无效');
    }
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if (expiresAt <= now) {
      throw const WalletAuthException('登录请求已过期，请重新扫描');
    }
    return _LoginSignatureClaim(
      requestId: parts[2],
      expiresAt: expiresAt,
    );
  }

  /// 查看钱包助记词（钱包级根备份；触发生物识别）。找不到即抛(与私钥导出同口径,
  /// 不让 UI 把 null 当"无数据"正常渲染)。
  Future<String> getMasterMnemonic(String masterId) async {
    await _authGate();
    final mnemonic = await _readMasterMnemonic(masterId);
    if (mnemonic == null || mnemonic.isEmpty) {
      throw const WalletAuthException('未找到该钱包的助记词备份，请重新导入钱包');
    }
    return mnemonic;
  }

  /// 导出该账户私钥（child mini-secret，`0x`+64hex；触发生物识别）。
  ///
  /// 从存储的 master 种子按 accountIndex 现场派生。child mini-secret 单向隔离:
  /// 导出单账户只暴露该账户,推不出根/兄弟(根级备份走助记词)。
  Future<String> getAccountPrivateKey(String accountId) async {
    await _authGate();
    final isar = await WalletIsar.instance.db();
    final acct = await isar.accountEntitys
        .filter()
        .accountIdEqualTo(accountId)
        .findFirst();
    if (acct == null) {
      throw const WalletAuthException('未找到指定账户');
    }
    final seedHex = await _readMasterSeedRaw(acct.masterId);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }
    final seed = _hexToBytes(seedHex);
    try {
      return '0x${_toHex(_childMiniSecret(seed, acct.accountIndex))}';
    } finally {
      _zeroList(seed);
    }
  }

  // ── 派生 ──
  Future<List<int>> _mnemonicToSeed(String mnemonic) async {
    final entropy =
        bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
    return CryptoScheme.miniSecretFromEntropy(entropy);
  }

  /// 从 master mini-secret 派生账户 [index]（全部 `//index` 硬派生，无 bare 根）,
  /// 返回该账户公钥 accountId + ss58。
  _DerivedAccount _deriveAccount(List<int> seed, int index) {
    if (index < 0) {
      throw ArgumentError.value(index, 'index', '不能为负');
    }
    final childBytes = Uint8List.fromList(_childMiniSecret(seed, index));
    try {
      final pair = Keyring.sr25519.fromSeed(childBytes);
      pair.ss58Format = _ss58Prefix;
      return _DerivedAccount(
        accountId: _accountIdFromBytes(pair.bytes().toList(growable: false)),
        ss58Address: pair.address,
      );
    } finally {
      childBytes.fillRange(0, childBytes.length, 0);
    }
  }

  /// 提取账户 `//index` 的 child mini-secret（32B）。
  ///
  /// 复用 `SecretUri` 解析 junction 的 chaincode，对 root SecretKey 做硬派生
  /// （`hardDeriveMiniSecretKey`，单向：child 推不出 parent/兄弟）。多级路径逐段
  /// 推进，返回末段 mini-secret。金标钉死 `fromSeed(child) == //index`。
  List<int> _childMiniSecret(List<int> seed, int index) {
    final junctions = SecretUri.fromStr('//$index').junctions;
    if (junctions.isEmpty) {
      throw StateError('派生路径 //$index 解析失败');
    }
    var rootSk = sr.MiniSecretKey.fromRawKey(seed).expandEd25519();
    late List<int> childBytes;
    try {
      for (final j in junctions) {
        if (!j.isHard) {
          throw StateError('仅支持硬派生 //index');
        }
        final cc = j.junctionId.sublist(0, 32);
        final derived = rootSk.hardDeriveMiniSecretKey(const <int>[], cc);
        childBytes = derived.$1.encode();
        _zeroSecretKey(rootSk);
        rootSk = derived.$1.expandEd25519();
        _zeroList(derived.$1.key);
        _zeroList(cc);
      }
      return childBytes;
    } finally {
      _zeroSecretKey(rootSk);
    }
  }

  // ── Secure Storage（按 master 存种子 + 助记词，均 AES-256-GCM 密文）──
  Future<void> _writeMasterSeed(String masterId, String seedHex) async {
    final storageKey = WalletSecureKeys.masterSeedHexV1(masterId);
    final encrypted = await SecretCipher.encrypt(
      seedHex,
      associatedData: storageKey,
    );
    await _secureStorage.write(
      key: storageKey,
      value: encrypted,
    );
  }

  Future<String?> _readMasterSeedRaw(String masterId) async {
    final storageKey = WalletSecureKeys.masterSeedHexV1(masterId);
    final stored = await _secureStorage.read(key: storageKey);
    if (stored == null) return null;
    final String seedHex;
    try {
      seedHex = await SecretCipher.decrypt(
        stored,
        associatedData: storageKey,
      );
    } on FormatException {
      // 密文损坏或 AEK 丢失(不可解):与格式非法同口径,提示重导(助记词仍可恢复)。
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
    if (!_seedHexPattern.hasMatch(seedHex)) {
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
    final seed = _hexToBytes(seedHex);
    try {
      if (_deriveAccount(seed, 0).accountId != masterId) {
        throw const WalletAuthException('钱包密钥归属异常，请重新导入钱包');
      }
    } finally {
      _zeroList(seed);
    }
    return seedHex;
  }

  Future<void> _deleteMasterSeed(String masterId) =>
      _secureStorage.delete(key: WalletSecureKeys.masterSeedHexV1(masterId));

  Future<void> _writeMasterMnemonic(String masterId, String mnemonic) async {
    final storageKey = WalletSecureKeys.masterMnemonicV1(masterId);
    final encrypted = await SecretCipher.encrypt(
      mnemonic,
      associatedData: storageKey,
    );
    await _secureStorage.write(
      key: storageKey,
      value: encrypted,
    );
  }

  Future<String?> _readMasterMnemonic(String masterId) async {
    final storageKey = WalletSecureKeys.masterMnemonicV1(masterId);
    final stored = await _secureStorage.read(key: storageKey);
    if (stored == null) return null;
    try {
      final mnemonic = await SecretCipher.decrypt(
        stored,
        associatedData: storageKey,
      );
      if (!_isValidMnemonic(mnemonic)) {
        throw const WalletAuthException('钱包助记词数据异常，请重新导入钱包');
      }
      final seed = await _mnemonicToSeed(mnemonic);
      try {
        if (_deriveAccount(seed, 0).accountId != masterId) {
          throw const WalletAuthException('钱包助记词归属异常，请重新导入钱包');
        }
      } finally {
        _zeroList(seed);
      }
      return mnemonic;
    } on FormatException {
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
  }

  Future<void> _deleteMasterMnemonic(String masterId) =>
      _secureStorage.delete(key: WalletSecureKeys.masterMnemonicV1(masterId));

  // ── 生物识别 ──
  /// **强制生物识别(指纹/面容),绝不回退设备密码/图案**;无生物识别硬件/未录入
  /// 一律 fail-closed 拒绝(连创建/导入/签名都不给)——冷钱包死规则。
  static Future<void> _defaultAuthGate() async {
    final List<Object?> available;
    try {
      available = await _localAuth.getAvailableBiometrics();
    } on LocalAuthException catch (e) {
      throw WalletAuthException(
        '认证服务异常：${e.description ?? e.code.name}，无法访问钱包',
      );
    }
    if (available.isEmpty) {
      throw const WalletAuthException(
        '必须启用生物识别（指纹/面容）才能使用冷钱包，请先在系统设置中录入',
      );
    }
    try {
      final ok = await _localAuth.authenticate(
        localizedReason: '请用生物识别验证身份以访问钱包密钥',
        biometricOnly: true,
        persistAcrossBackgrounding: true,
        sensitiveTransaction: true,
      );
      if (!ok) {
        throw const WalletAuthException('未通过生物识别验证');
      }
    } on LocalAuthException catch (e) {
      if (e.code == LocalAuthExceptionCode.noBiometricHardware ||
          e.code == LocalAuthExceptionCode.noBiometricsEnrolled ||
          e.code == LocalAuthExceptionCode.noCredentialsSet) {
        throw const WalletAuthException('设备未录入生物识别，无法使用冷钱包');
      }
      throw WalletAuthException(
        '认证服务异常：${e.description ?? e.code.name}，请稍后重试',
      );
    }
  }

  // ── 内部工具 ──
  /// 原子建钱包：同一事务分配 walletIndex、写 WalletEntity + 账户0（`//0`）。
  Future<(Wallet, Account)> _appendWalletAtomic({
    required String masterId,
    required String source,
    required _DerivedAccount primary,
  }) async {
    final isar = await WalletIsar.instance.db();
    final now = DateTime.now().millisecondsSinceEpoch;
    late int walletIndex;
    late int sortOrder;
    await isar.writeTxn(() async {
      final duplicate = await isar.walletEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .findFirst();
      if (duplicate != null) {
        throw Exception('该钱包已存在（${duplicate.walletName}），无需重复导入');
      }
      final wallets =
          await isar.walletEntitys.where().sortByWalletIndex().findAll();
      final used = wallets.map((e) => e.walletIndex).toSet();
      walletIndex = 1;
      while (used.contains(walletIndex)) {
        walletIndex++;
      }
      final maxSort = wallets.isEmpty
          ? -1
          : wallets.map((e) => e.sortOrder).reduce((a, b) => a > b ? a : b);
      sortOrder = maxSort + 1;

      final wallet = WalletEntity()
        ..walletIndex = walletIndex
        ..walletName = '钱包$walletIndex'
        ..masterId = masterId
        ..createdAtMillis = now
        ..source = source
        ..sortOrder = sortOrder;
      await isar.walletEntitys.put(wallet);

      final account = AccountEntity()
        ..masterId = masterId
        ..accountIndex = 0
        ..accountId = primary.accountId
        ..ss58Address = primary.ss58Address
        ..accountName = '账户0'
        ..createdAtMillis = now;
      await isar.accountEntitys.put(account);
    });

    return (
      Wallet(
        walletIndex: walletIndex,
        walletName: '钱包$walletIndex',
        masterId: masterId,
        createdAtMillis: now,
        source: source,
        sortOrder: sortOrder,
      ),
      Account(
        masterId: masterId,
        accountIndex: 0,
        accountId: primary.accountId,
        ss58Address: primary.ss58Address,
        accountName: '账户0',
        createdAtMillis: now,
      ),
    );
  }

  static void _zeroList(List<int> bytes) {
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = 0;
    }
  }

  static void _zeroSecretKey(sr.SecretKey secretKey) {
    _zeroList(secretKey.key);
    _zeroList(secretKey.nonce);
  }

  static bool _isValidMnemonic(String mnemonic) {
    try {
      bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english);
      return true;
    } catch (_) {
      return false;
    }
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }

  String _accountIdFromBytes(List<int> bytes) {
    if (bytes.length != 32) {
      throw ArgumentError.value(
          bytes.length, 'bytes.length', '账户 ID 必须是 32 字节');
    }
    final text = '0x${_toHex(bytes)}';
    if (!_accountIdPattern.hasMatch(text)) {
      throw StateError('派生出的 accountId 非规范形式');
    }
    return text;
  }

  List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }

  Wallet _toWallet(WalletEntity row) => Wallet(
        walletIndex: row.walletIndex,
        walletName: row.walletName,
        masterId: row.masterId,
        createdAtMillis: row.createdAtMillis,
        source: row.source,
        sortOrder: row.sortOrder,
      );

  Account _toAccount(AccountEntity row) => Account(
        masterId: row.masterId,
        accountIndex: row.accountIndex,
        accountId: row.accountId,
        ss58Address: row.ss58Address,
        accountName: row.accountName,
        createdAtMillis: row.createdAtMillis,
      );
}

class _DerivedAccount {
  const _DerivedAccount({required this.accountId, required this.ss58Address});
  final String accountId;
  final String ss58Address;
}
