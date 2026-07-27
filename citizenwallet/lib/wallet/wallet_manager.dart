import 'dart:convert';
import 'dart:typed_data';

import 'package:bip39/bip39.dart' as bip39;
import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:flutter/services.dart' show PlatformException;
import 'package:isar/isar.dart';
import 'package:local_auth/local_auth.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenwallet/chain/chain_constants.dart';
import 'package:citizenwallet/isar/wallet_isar.dart';
import 'package:citizenwallet/security/secure_storage.dart';
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
    required this.accountId,
    required this.signerPublicKey,
    required this.alg,
    required this.signatureHex,
  });

  final String accountId;
  final String signerPublicKey;
  final String alg;
  final String signatureHex;
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

  @visibleForTesting
  static set debugAuthGate(Future<void> Function()? gate) =>
      _authGate = gate ?? _defaultAuthGate;

  // ── 查询 ──
  Future<List<Wallet>> getWallets() async {
    final isar = await WalletIsar.instance.db();
    final rows =
        await isar.walletEntitys.where().sortBySortOrder().findAll();
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
    final strength = wordCount == 24 ? 256 : 128;
    final mnemonic = bip39.generateMnemonic(strength: strength);
    return _establishWallet(mnemonic, 'created');
  }

  /// 导入钱包：校验助记词 → 派生账户0（`//0`）→ 存 master 种子 + 助记词。
  Future<WalletCreationResult> importWallet(String mnemonic) async {
    final trimmed = mnemonic.trim();
    if (!bip39.validateMnemonic(trimmed)) {
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
      await _checkDuplicateMaster(masterId);

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

  /// 在指定钱包下新增账户：读存储种子，派生下一个 `//index`（不产生新助记词）。
  Future<Account> addAccount(String masterId) async {
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
      final accounts = await isar.accountEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .findAll();
      final nextIndex = accounts
              .map((e) => e.accountIndex)
              .fold<int>(-1, (m, i) => i > m ? i : m) +
          1;

      final derived = _deriveAccount(seed, nextIndex);
      final now = DateTime.now().millisecondsSinceEpoch;
      final entity = AccountEntity()
        ..masterId = masterId
        ..accountIndex = nextIndex
        ..accountId = derived.accountId
        ..ss58Address = derived.ss58Address
        ..accountName = '账户$nextIndex'
        ..createdAtMillis = now;
      await isar.writeTxn(() async {
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
  /// 先删 Isar 行(钱包存在性的事实来源),提交成功后再清 SecureStorage,
  /// 避免"密钥已清、钱包行还在"的僵尸钱包。
  Future<void> _deleteWalletInternal(String masterId) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.accountEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .deleteAll();
      final wallet = await isar.walletEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .findFirst();
      if (wallet != null) {
        await isar.walletEntitys.delete(wallet.id);
      }
    });
    // 尽力而为:Isar 行(钱包存在性事实来源)已删,SecureStorage 清理不应因单点
    // 失败让整体判为"删除失败"(那样 UI 会误导用户重试一个已不存在的钱包)。
    try {
      await _deleteMasterSeed(masterId);
    } catch (_) {}
    try {
      await _deleteMasterMnemonic(masterId);
    } catch (_) {}
  }

  /// 删除单个账户；若删空该钱包全部账户则连带删钱包与密钥。删前强制认证。
  ///
  /// 账户0 是 master 锚点:尚有兄弟账户时禁止单独删账户0(否则 masterId 悬空、
  /// addAccount 只 max+1 无法重建、重导入又被查重挡住),须改删整只钱包。
  Future<void> deleteAccount(String accountId) async {
    await _authGate();
    final isar = await WalletIsar.instance.db();
    final acct = await isar.accountEntitys
        .filter()
        .accountIdEqualTo(accountId)
        .findFirst();
    if (acct == null) {
      throw Exception('未找到账户');
    }
    final masterId = acct.masterId;
    if (acct.accountIndex == 0) {
      final siblings = await isar.accountEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .count();
      if (siblings > 1) {
        throw Exception('账户0是钱包锚点,请删除整个钱包');
      }
    }
    // 删除 + 剩余计数并入同一写事务,消除计数误读的竞态窗口。
    late int remaining;
    await isar.writeTxn(() async {
      await isar.accountEntitys.delete(acct.id);
      remaining = await isar.accountEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .count();
    });
    if (remaining == 0) {
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
    final row = await isar.walletEntitys
        .filter()
        .masterIdEqualTo(masterId)
        .findFirst();
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

  // ── 签名（按账户；读种子现场派生，用后即弃）──
  // 注:polkadart_keyring 0.7.0 的 KeyPair.lock() 内部用 fromEd25519Bytes(空)
  // 会抛 ArgumentError,该版本不可用于清私钥;派生密钥对随作用域结束由 GC 回收。
  Future<Uint8List> signForAccount(
    String accountId,
    Uint8List payload,
  ) async {
    final pair = await _loadAccountKeyPair(accountId);
    return Uint8List.fromList(pair.sign(payload));
  }

  Future<WalletSignResult> signUtf8ForAccount(
    String accountId,
    String message,
  ) async {
    final pair = await _loadAccountKeyPair(accountId);
    final signature = pair.sign(Uint8List.fromList(utf8.encode(message)));
    return WalletSignResult(
      accountId: accountId,
      signerPublicKey: accountId,
      alg: 'sr25519',
      signatureHex: '0x${_toHex(signature.toList(growable: false))}',
    );
  }

  /// 定位账户 → 读 master 种子（触发生物识别）→ 按 accountIndex 派生 → 校验公钥。
  Future<KeyPair> _loadAccountKeyPair(String accountId) async {
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
      final child = _childMiniSecret(seed, acct.accountIndex);
      final childBytes = Uint8List.fromList(child);
      try {
        final pair = Keyring.sr25519.fromSeed(childBytes);
        pair.ss58Format = _ss58Prefix;
        final localAccountId =
            _accountIdFromBytes(pair.bytes().toList(growable: false));
        if (localAccountId != acct.accountId) {
          throw const WalletAuthException('本地签名密钥与账户不一致，请重新导入钱包');
        }
        return pair;
      } finally {
        childBytes.fillRange(0, childBytes.length, 0);
      }
    } finally {
      _zeroList(seed);
    }
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
    for (final j in junctions) {
      if (!j.isHard) {
        throw StateError('仅支持硬派生 //index');
      }
      final cc = j.junctionId.sublist(0, 32);
      final derived = rootSk.hardDeriveMiniSecretKey(const <int>[], cc);
      childBytes = derived.$1.encode();
      rootSk = derived.$1.expandEd25519();
    }
    return childBytes;
  }

  // ── Secure Storage（按 master 存种子 + 助记词，均 AES-256-GCM 密文）──
  Future<void> _writeMasterSeed(String masterId, String seedHex) async {
    final encrypted = await SecretCipher.encrypt(seedHex);
    await _secureStorage.write(
        key: WalletSecureKeys.masterSeedHexV1(masterId), value: encrypted);
  }

  Future<String?> _readMasterSeedRaw(String masterId) async {
    final stored = await _secureStorage.read(
        key: WalletSecureKeys.masterSeedHexV1(masterId));
    if (stored == null) return null;
    final String seedHex;
    try {
      seedHex = await SecretCipher.decrypt(stored);
    } on FormatException {
      // 密文损坏或 AEK 丢失(不可解):与格式非法同口径,提示重导(助记词仍可恢复)。
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
    if (!_seedHexPattern.hasMatch(seedHex)) {
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
    return seedHex;
  }

  Future<void> _deleteMasterSeed(String masterId) => _secureStorage.delete(
      key: WalletSecureKeys.masterSeedHexV1(masterId));

  Future<void> _writeMasterMnemonic(String masterId, String mnemonic) async {
    final encrypted = await SecretCipher.encrypt(mnemonic);
    await _secureStorage.write(
        key: WalletSecureKeys.masterMnemonicV1(masterId), value: encrypted);
  }

  Future<String?> _readMasterMnemonic(String masterId) async {
    final stored = await _secureStorage.read(
        key: WalletSecureKeys.masterMnemonicV1(masterId));
    if (stored == null) return null;
    try {
      return await SecretCipher.decrypt(stored);
    } on FormatException {
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
  }

  Future<void> _deleteMasterMnemonic(String masterId) =>
      _secureStorage.delete(key: WalletSecureKeys.masterMnemonicV1(masterId));

  // ── 生物识别 ──
  /// 优先生物识别，回退设备密码/图案；无锁屏或认证异常一律拒绝访问。
  static Future<void> _defaultAuthGate() async {
    bool supported;
    try {
      supported = await _localAuth.isDeviceSupported();
    } on PlatformException catch (e) {
      throw WalletAuthException('认证服务异常：${e.message}，无法访问钱包');
    }
    if (!supported) {
      throw const WalletAuthException(
        '设备未设置锁屏密码或安全措施，请先在系统设置中启用锁屏保护后再使用冷钱包',
      );
    }
    try {
      final ok = await _localAuth.authenticate(
        localizedReason: '请验证身份以访问钱包密钥',
        options: const AuthenticationOptions(
          biometricOnly: false,
          stickyAuth: true,
          useErrorDialogs: true,
        ),
      );
      if (!ok) {
        throw const WalletAuthException('未通过身份验证');
      }
    } on PlatformException catch (e) {
      final code = e.code;
      if (code == 'NotAvailable' || code == 'NotEnrolled') {
        throw const WalletAuthException(
          '设备未设置锁屏密码或安全措施，请先在系统设置中启用锁屏保护后再使用冷钱包',
        );
      }
      throw WalletAuthException('认证服务异常：${e.message}，请稍后重试');
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

  Future<void> _checkDuplicateMaster(String masterId) async {
    final isar = await WalletIsar.instance.db();
    final exists =
        await isar.walletEntitys.filter().masterIdEqualTo(masterId).findFirst();
    if (exists != null) {
      throw Exception('该钱包已存在（${exists.walletName}），无需重复导入');
    }
  }

  static void _zeroList(List<int> bytes) {
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = 0;
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
      throw ArgumentError.value(bytes.length, 'bytes.length', '账户 ID 必须是 32 字节');
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
