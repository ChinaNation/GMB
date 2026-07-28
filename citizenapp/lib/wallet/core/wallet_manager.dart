import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:bip39/bip39.dart' as bip39;
import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:cryptography/cryptography.dart' hide KeyPair;
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';

class WalletProfile {
  const WalletProfile({
    required this.walletIndex,
    required this.walletName,
    required this.walletIcon,
    required this.balance,
    required this.accountId,
    required this.ss58Address,
    required this.alg,
    required this.ss58,
    required this.createdAtMillis,
    required this.source,
    required this.signMode,
    this.sortOrder = 0,
  });

  final int walletIndex;
  final String walletName;
  final String walletIcon;
  final double balance;
  final String accountId;
  final String ss58Address;
  final String alg;
  final int ss58;
  final int createdAtMillis;
  final String source;

  /// 签名模式：`local`（热钱包）或 `external`（冷钱包）。
  final String signMode;

  /// 用户拖拽排序后的稳定顺序，数值越小越靠前。
  final int sortOrder;

  bool get isHotWallet => signMode == 'local';
  bool get isColdWallet => signMode == 'external';
}

/// 一只钱包(masterId)下的一个账户(`//index`,含账户0 = `//0`)。
///
/// 无根多账户模型的公开视图:只含身份字段,不含任何私钥材料。签名 / 导私钥须回
/// [WalletManager.signForAccountId] / [WalletManager.getAccountPrivateKey],经硬件金库
/// 按 accountId 读回该账户的 child(触发生物识别)。
class Account {
  const Account({
    required this.masterId,
    required this.accountIndex,
    required this.accountId,
    required this.ss58Address,
    required this.accountName,
  });

  final String masterId;
  final int accountIndex;
  final String accountId;
  final String ss58Address;
  final String accountName;

  /// 展示用派生路径:`//index`(账户0 = `//0`)。
  String get derivationPath => '//$accountIndex';
}

class WalletCreationResult {
  const WalletCreationResult({
    required this.profile,
    required this.mnemonic,
  });

  final WalletProfile profile;

  /// 助记词仅在创建时一次性展示，不会持久化。
  final String mnemonic;
}

class WalletAuthException implements Exception {
  const WalletAuthException(this.message);
  final String message;

  @override
  String toString() => 'WalletAuthException: $message';
}

/// 通讯录专用密钥材料。账户0 的 child mini-secret 只在 [WalletManager] 内参与
/// HKDF，业务层只能拿到已域隔离的加密钥和索引钥，不能借此签名、恢复钱包或推导
/// 其他业务密钥。
class ContactKeyMaterial {
  const ContactKeyMaterial({
    required this.encryptionKey,
    required this.indexKey,
  });

  final Uint8List encryptionKey;
  final Uint8List indexKey;
}

/// 钱包创建后注册 P-256 设备子钥的钩子：给定 walletIndex/accountId 与一个对
/// 绑定消息做账户0 sr25519 签名的闭包（返回 `0x` hex）。由 app 启动注入实现，
/// 避免 wallet/core 反向依赖 8964 层。
typedef WalletSubkeyRegistrar = Future<void> Function({
  required int walletIndex,
  required String accountId,
  required Future<String> Function(Uint8List bindingMessage) signBinding,
});

class WalletManager {
  static const int _ss58Format = 2027;

  /// 账户派生序号上界(`//index` 的 index 最大值)。账户0 为锚点主账户,
  /// 追加账户序号取 `1.._maxAccountIndex`。与 citizenwallet 冷端同源。
  static const int _maxAccountIndex = 1989;

  /// [_maxAccountIndex] 的公开只读别名，供 UI 展示 / 校验「指定序号」范围时单源引用，
  /// 避免把上界魔法数抄进界面层。
  static const int maxAccountIndex = _maxAccountIndex;

  /// 钱包身份数据版本号：钱包增删、拖拽排序、改名，以及 **CID 占号 / 换绑改变
  /// 「身份账户」绑定**后自增。
  ///
  /// 身份主键 = CID 号,其落点(CID 绑定的钱包账户)是链上派生而非本地存储字段,
  /// 占号/换绑写完没有任何本地广播。常驻页面（我的 tab、广场首页、Chat 会话列表、
  /// 身份页）监听此版本号,在身份账户变化后立即重读身份,避免「UI 显示旧身份、
  /// 动作以新身份执行」的分叉。余额刷新是高频操作且不影响身份,不计入此版本号。
  static final ValueNotifier<int> walletsRevision = ValueNotifier<int>(0);

  static void _bumpWalletsRevision() {
    walletsRevision.value++;
  }

  /// CID 占号 / 换绑改变了「身份账户」绑定(钱包列表没变,但身份主键的落点变了)。
  /// 复用同一身份版本号广播,避免第二套通知机制;常驻页据此重读身份。
  static void notifyIdentityBindingChanged() => _bumpWalletsRevision();

  /// 账户 child mini-secret 的硬件级安全存储后端（[HardwareBoundSeedVault]：
  /// Keystore/SE auth-bound KEK 信封加密，**读 child 时由硬件 + 生物识别原子
  /// 解锁**，写入静默）；测试经 [debugSeedStore] 注入内存 fake。无根模型只存
  /// child，绝不存母种子 / 助记词。
  static SecureSeedStore _store = HardwareBoundSeedVault();

  /// 通讯录专用密钥是从账户0 child 域隔离派生后的 64 字节材料，静默保存在系统
  /// 安全存储；它不需要每次查看通讯录都重复触发生物识别。
  static VaultBlobStore _contactKeyStore = SecureStorageBlobStore();

  @visibleForTesting
  static set debugSeedStore(SecureSeedStore store) => _store = store;

  @visibleForTesting
  static set debugContactKeyStore(VaultBlobStore store) =>
      _contactKeyStore = store;

  /// 钱包创建后注册 P-256 设备子钥的钩子（app 启动注入；为空则跳过，用于测试 /
  /// 未接后端）。「每次动钱动权都验证」现由硬件金库读 child 时的原子生物识别实现，
  /// 不再需要操作层 local_auth 软门禁。
  static WalletSubkeyRegistrar? _subkeyRegistrar;

  static set subkeyRegistrar(WalletSubkeyRegistrar? registrar) =>
      _subkeyRegistrar = registrar;

  // 查询
  /// 钱包列表查询入口。
  /// - 排序规则：sortOrder 升序优先，相同则回退 walletIndex 兜底（保证稳定）。
  Future<List<WalletProfile>> getWallets() async {
    final rows = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys
          .where()
          .sortBySortOrder()
          .thenByWalletIndex()
          .findAll();
    });
    return rows.map(_toProfile).toList(growable: false);
  }

  /// 按传入的 walletIndex 顺序写入新的 sortOrder。
  /// 在一次 Isar 事务里完成，失败回滚。
  Future<void> reorderWallets(List<int> walletIndexes) async {
    await WalletIsar.instance.writeTxn((isar) async {
      for (var i = 0; i < walletIndexes.length; i++) {
        final entity = await isar.walletProfileEntitys
            .filter()
            .walletIndexEqualTo(walletIndexes[i])
            .findFirst();
        if (entity != null) {
          entity.sortOrder = i;
          await isar.walletProfileEntitys.put(entity);
        }
      }
    });
    _bumpWalletsRevision();
  }

  Future<WalletProfile?> getWallet() async {
    final snapshot = await WalletIsar.instance.read((isar) async {
      final wallets =
          await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
      if (wallets.isEmpty) {
        return null;
      }
      final settings = await isar.walletSettingsEntitys.get(0);
      return (
        wallets: wallets,
        activeIndex: settings?.activeWalletIndex,
      );
    });
    if (snapshot == null) {
      return null;
    }

    WalletProfileEntity selected = snapshot.wallets.last;
    if (snapshot.activeIndex != null) {
      for (final wallet in snapshot.wallets) {
        if (wallet.walletIndex == snapshot.activeIndex) {
          selected = wallet;
          break;
        }
      }
    } else {
      await WalletIsar.instance.writeTxn((isar) async {
        final settings = await _getSettingsInTxn(isar);
        settings.activeWalletIndex = selected.walletIndex;
        settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.walletSettingsEntitys.put(settings);
      });
    }

    return _toProfile(selected);
  }

  Future<WalletProfile?> getWalletByIndex(int walletIndex) async {
    final row = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
    });
    if (row == null) {
      return null;
    }
    return _toProfile(row);
  }

  /// 默认用户钱包：钱包列表中最靠前的**热钱包**。
  ///
  /// 这是公民 App 的统一身份来源，同时用于聊天和发动态。排序沿用
  /// [getWallets] 的 sortOrder（用户拖拽置顶即改默认），冷钱包永不成为
  /// 默认。列表中没有任何热钱包时返回 null，由上层给出创建热钱包引导。
  Future<WalletProfile?> getDefaultWallet() async {
    final wallets = await getWallets();
    for (final wallet in wallets) {
      if (wallet.isHotWallet) {
        return wallet;
      }
    }
    return null;
  }

  /// 默认用户钱包的 walletIndex；无热钱包时返回 null。
  Future<int?> getDefaultWalletIndex() async {
    final wallet = await getDefaultWallet();
    return wallet?.walletIndex;
  }

  /// 「有效热钱包」单源谓词 —— 门控与其他调用方共用同一把尺子。
  ///
  /// 四条全过才算有效：
  /// 1. 是热钱包（冷钱包永不作为身份依据）；
  /// 2. `accountId` 为规范形式（ADR-040）；
  /// 3. `ss58Address` 非空且与 `accountId` 派生结果一致；
  /// 4. 严档 child 条目存在（[SecureSeedStore.hasAccountKey] 静默探测，不弹生物识别）。
  ///
  /// 只判 null 是不够的：Isar 属性改名等原因会留下「行还在、身份字段为空」的
  /// 半残钱包，它能骗过 null 判定进 App，然后下游全部静默降级成「没钱包」。
  Future<bool> isUsableHotWallet(WalletProfile wallet) async {
    if (!wallet.isHotWallet) return false;
    if (!isAccountIdText(wallet.accountId)) return false;
    if (wallet.ss58Address.isEmpty) return false;
    if (ss58FromAccountIdText(wallet.accountId) != wallet.ss58Address) {
      return false;
    }
    return _store.hasAccountKey(wallet.accountId);
  }

  /// 列表中第一个**有效**热钱包；没有则 null。这是账户门禁的唯一依据。
  Future<WalletProfile?> getValidDefaultWallet() async {
    for (final wallet in await getWallets()) {
      if (await isUsableHotWallet(wallet)) return wallet;
    }
    return null;
  }

  Future<int?> getActiveWalletIndex() async {
    return WalletIsar.instance.read((isar) async {
      final settings = await isar.walletSettingsEntitys.get(0);
      return settings?.activeWalletIndex;
    });
  }

  Future<void> setActiveWallet(int walletIndex) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final exists = await isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
      if (exists == null) {
        throw Exception('未找到指定钱包');
      }
      final settings = await _getSettingsInTxn(isar);
      settings.activeWalletIndex = walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });
  }

  // 热钱包创建 / 导入
  /// 创建热钱包（ROOTLESS）：生成助记词 → 派生母种子 → 派生账户0(`//0`) →
  /// **只存账户0 的 child mini-secret**，母种子派生后立即清零，助记词一次性返回
  /// 供备份、**绝不持久化**。
  ///
  /// [wordCount] 助记词个数，12（默认）或 24。
  Future<WalletCreationResult> createWallet({int wordCount = 12}) async {
    await _ensureDeviceSecure();
    await _ensureNoExistingHotWallet();
    assert(wordCount == 12 || wordCount == 24);
    final strength = wordCount == 24 ? 256 : 128;
    final mnemonic = bip39.generateMnemonic(strength: strength);
    final account0 = await _deriveAccount0FromMnemonic(mnemonic);

    final profile = await _appendHotWalletAtomic(
      account0: account0,
      source: 'created',
    );
    try {
      await _persistContactKeys(profile.accountId, account0.childMiniSecret);
      await _verifyWalletPersisted(profile);
      // fail-closed：设备子钥注册必须成功，失败连同钱包一起回滚，绝不留"建了没注册"的中间态。
      // 注册成功后才由 runCreateWalletFlow 展示助记词、进入 App。
      await _registerDeviceSubkey(
          profile.walletIndex, profile.accountId, account0.childMiniSecret);
    } catch (_) {
      await _rollbackWalletCreation(profile.walletIndex);
      rethrow;
    } finally {
      account0.dispose();
    }
    _bumpWalletsRevision();
    return WalletCreationResult(profile: profile, mnemonic: mnemonic);
  }

  /// 导入热钱包（ROOTLESS）：验证助记词 → 派生账户0 → **只存账户0 的 child
  /// mini-secret**，母种子清零，助记词不持久化。
  Future<WalletProfile> importWallet(String mnemonic) async {
    await _ensureDeviceSecure();
    await _ensureNoExistingHotWallet();
    final trimmed = mnemonic.trim();
    if (!bip39.validateMnemonic(trimmed)) {
      throw Exception('助记词无效，请检查拼写和空格');
    }

    final account0 = await _deriveAccount0FromMnemonic(trimmed);
    try {
      // 检测重复：同一公钥的钱包已存在则拒绝
      await _checkDuplicateAccountId(account0.accountId);

      final profile = await _appendHotWalletAtomic(
        account0: account0,
        source: 'imported',
      );
      try {
        await _persistContactKeys(profile.accountId, account0.childMiniSecret);
        await _verifyWalletPersisted(profile);
        // fail-closed：导入一律注册本设备子钥（幂等 upsert），失败连同钱包回滚——导入页保留
        // 助记词供重试。换设备导入必然是本设备新子钥，注册成功即把账户登录迁到本设备。
        await _registerDeviceSubkey(
            profile.walletIndex, profile.accountId, account0.childMiniSecret);
      } catch (_) {
        await _rollbackWalletCreation(profile.walletIndex);
        rethrow;
      }
      _bumpWalletsRevision();
      return profile;
    } finally {
      account0.dispose();
    }
  }

  // 冷钱包导入
  /// 导入冷钱包：只接受本链 SS58 地址，并只保存公开账户资料。
  Future<WalletProfile> importColdWallet({required String ss58Address}) async {
    final trimmed = ss58Address.trim();
    if (trimmed.isEmpty) {
      throw Exception('地址不能为空');
    }

    final List<int> publicKeyBytes;
    try {
      publicKeyBytes = Keyring().decodeAddress(trimmed);
    } catch (_) {
      throw Exception('无效的 SS58 地址');
    }
    // 用本链前缀重新编码并逐字比较，拒绝其他网络和非规范地址。
    final normalizedSs58Address =
        Keyring().encodeAddress(publicKeyBytes, _ss58Format);
    if (normalizedSs58Address != trimmed) {
      throw Exception(
        '地址前缀不匹配（本链 SS58 前缀为 $_ss58Format），请确认地址来自本链',
      );
    }

    final accountId = _accountIdFromBytes(publicKeyBytes);

    // 检测重复：同一公钥的钱包已存在则拒绝
    await _checkDuplicateAccountId(accountId);

    final profile = await _appendColdWalletAtomic(
      ss58Address: normalizedSs58Address,
      accountId: accountId,
    );
    _bumpWalletsRevision();
    return profile;
  }

  // 多账户（一只钱包 masterId = 一套助记词，下辖多个 //index 账户）
  /// 某钱包(masterId)下全部账户,按 accountIndex 升序(账户0 在最前)。
  Future<List<Account>> getAccounts(String masterId) async {
    final rows = await WalletIsar.instance.read((isar) {
      return isar.accountEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .sortByAccountIndex()
          .findAll();
    });
    return rows.map(_toAccount).toList(growable: false);
  }

  /// 按 accountId 取单个账户;不存在返回 null。
  Future<Account?> getAccountByAccountId(String accountId) async {
    final row = await WalletIsar.instance.read((isar) {
      return isar.accountEntitys
          .filter()
          .accountIdEqualTo(accountId)
          .findFirst();
    });
    return row == null ? null : _toAccount(row);
  }

  /// 在钱包(masterId)下批量追加账户(`//index`),原子写入 + 失败整批回滚。
  ///
  /// 无根:本端不存母种子,追加账户须重新录入 [mnemonic]。校验全部先于落库:
  /// 1) 助记词合法 + **归属校验**——由它派生的账户0.accountId 必须等于 [masterId],
  ///    否则抛 [WalletAuthException]（错助记词 / 换钱包早拒）。
  /// 2) [indices] 非空;每个序号在 `1.._maxAccountIndex`(账户0 是锚点不可再加);输入
  ///    内不得重复;不得与既有账户序号冲突。任一违规抛 [Exception]。
  /// 3) 逐个派生 child + accountId + ss58。
  /// 4) 原子批:一次 writeTxn 写全部 AccountEntity → 逐个 putAccountKey;任一步抛错则
  ///    整批回滚(删本批 AccountEntity + deleteAccountKey 已写入的 child)。母种子在
  ///    finally 清零。返回新增账户列表。
  Future<List<Account>> addAccounts(
    String masterId,
    String mnemonic,
    List<int> indices,
  ) async {
    final trimmed = mnemonic.trim();
    if (!bip39.validateMnemonic(trimmed)) {
      throw Exception('助记词无效，请检查拼写和空格');
    }

    // 归属校验:助记词派生的账户0 必须就是这只钱包(masterId)。
    final account0 = await _deriveAccount0FromMnemonic(trimmed);
    try {
      if (account0.accountId != masterId) {
        throw const WalletAuthException('助记词与该钱包不符');
      }

      final profile = await _requireHotWalletProfileByMasterId(masterId);
      final walletIndex = profile.walletIndex;

      // 序号校验:非空 / 范围 / 输入内去重(有重即拒) / 不与既有冲突。
      if (indices.isEmpty) {
        throw Exception('未指定要追加的账户序号');
      }
      final seen = <int>{};
      for (final index in indices) {
        if (index < 1 || index > _maxAccountIndex) {
          throw Exception('账户序号需在 1–$_maxAccountIndex,账户0 为锚点不可再加');
        }
        if (!seen.add(index)) {
          throw Exception('账户序号重复:$index');
        }
      }
      final existing = await _existingAccountIndexes(masterId);
      for (final index in seen) {
        if (existing.contains(index)) {
          throw Exception('账户$index 已存在');
        }
      }
      final targets = seen.toList(growable: false)..sort();

      // 逐个派生(母种子只在此作用域存活,finally 清零)。
      final seed = await _mnemonicToMiniSecret(trimmed);
      final derived = <_Account0>[];
      try {
        final now = DateTime.now().millisecondsSinceEpoch;
        final entities = <AccountEntity>[];
        for (final index in targets) {
          final account = _deriveAccount(seed, index);
          derived.add(account);
          entities.add(AccountEntity()
            ..masterId = masterId
            ..accountIndex = index
            ..accountId = account.accountId
            ..ss58Address = account.ss58Address
            ..accountName = _defaultAccountName(index)
            ..createdAtMillis = now);
        }

        // 原子批 4a:一次事务写全部 AccountEntity。
        await WalletIsar.instance.writeTxn((isar) async {
          for (final entity in entities) {
            await isar.accountEntitys.put(entity);
          }
        });

        // 原子批 4b:逐个写 child;任一失败 → 整批回滚(删本批行 + 已写 child)。
        final stored = <String>[];
        try {
          for (var i = 0; i < targets.length; i++) {
            await _store.putAccountKey(
              walletIndex: walletIndex,
              accountId: derived[i].accountId,
              childMiniSecretHex: _toHex(derived[i].childMiniSecret),
            );
            stored.add(derived[i].accountId);
          }
        } catch (_) {
          await WalletIsar.instance.writeTxn((isar) async {
            for (final entity in entities) {
              final row = await isar.accountEntitys
                  .filter()
                  .accountIdEqualTo(entity.accountId)
                  .findFirst();
              if (row != null) {
                await isar.accountEntitys.delete(row.id);
              }
            }
          });
          for (final accountId in stored) {
            await _store.deleteAccountKey(
                walletIndex: walletIndex, accountId: accountId);
          }
          rethrow;
        }

        return entities.map(_toAccount).toList(growable: false);
      } finally {
        seed.fillRange(0, seed.length, 0);
        for (final account in derived) {
          account.dispose();
        }
      }
    } finally {
      account0.dispose();
    }
  }

  /// 便捷:追加「下一个」账户(既有最大序号 + 1)。
  Future<Account> addNextAccount(String masterId, String mnemonic) async {
    final existing = await _existingAccountIndexes(masterId);
    final maxIndex =
        existing.fold<int>(0, (max, index) => index > max ? index : max);
    final added = await addAccounts(masterId, mnemonic, <int>[maxIndex + 1]);
    return added.single;
  }

  /// 导出指定账户私钥(child mini-secret,`0x` + 64 hex;读硬件金库触发生物识别)。
  Future<String> getAccountPrivateKey(String accountId) async {
    final account = await getAccountByAccountId(accountId);
    if (account == null) {
      throw const WalletAuthException('未找到指定账户');
    }
    final profile = await _requireHotWalletProfileByMasterId(account.masterId);
    final childHex =
        await _readAccountKeyOrThrow(profile.walletIndex, accountId);
    final hex = childHex.startsWith('0x') ? childHex.substring(2) : childHex;
    return '0x$hex';
  }

  /// 用指定 accountId 的私钥对 [payload] 签名(多账户签名入口)。
  ///
  /// 读该账户 child(触发生物识别)→ fromSeed → 校验派生公钥 == accountId → 签名 →
  /// 清零。默认身份 / 统一签名仍走账户0([signWithWallet]),本方法不改默认身份。
  Future<Uint8List> signForAccountId(String accountId, Uint8List payload) async {
    final account = await getAccountByAccountId(accountId);
    if (account == null) {
      throw const WalletAuthException('未找到指定账户');
    }
    final profile = await _requireHotWalletProfileByMasterId(account.masterId);
    final childHex =
        await _readAccountKeyOrThrow(profile.walletIndex, accountId);
    final childBytes = Uint8List.fromList(_hexToBytes(childHex));
    try {
      final pair = Keyring.sr25519.fromSeed(childBytes);
      pair.ss58Format = profile.ss58;
      final localAccountId =
          _accountIdFromBytes(pair.bytes().toList(growable: false));
      if (localAccountId != accountId) {
        throw const WalletAuthException('本地签名密钥与账户不一致，请重新导入钱包');
      }
      return Uint8List.fromList(pair.sign(payload));
    } finally {
      childBytes.fillRange(0, childBytes.length, 0);
    }
  }

  /// 删除单个账户。锚点守卫:账户0 且存在兄弟账户时禁止单删(须删整只钱包);
  /// 删空该钱包全部账户则级联删钱包(同 [deleteWallet])。
  Future<void> deleteAccount(String accountId) async {
    final account = await getAccountByAccountId(accountId);
    if (account == null) {
      throw Exception('未找到账户');
    }
    final masterId = account.masterId;
    final profile = await _requireHotWalletProfileByMasterId(masterId);
    final walletIndex = profile.walletIndex;

    if (account.accountIndex == 0) {
      final siblings = await _existingAccountIndexes(masterId);
      if (siblings.length > 1) {
        throw Exception('账户0是钱包锚点,请删除整个钱包');
      }
    }

    late int remaining;
    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.accountEntitys
          .filter()
          .accountIdEqualTo(accountId)
          .findFirst();
      if (row != null) {
        await isar.accountEntitys.delete(row.id);
      }
      remaining =
          await isar.accountEntitys.filter().masterIdEqualTo(masterId).count();
    });
    await _store.deleteAccountKey(
        walletIndex: walletIndex, accountId: accountId);

    // 删空 = 该助记词已无可用账户,级联删钱包(清 WalletProfile + 剩余账户行 + 通讯录钥)。
    if (remaining == 0) {
      await deleteWallet(walletIndex);
    }
  }

  Future<Set<int>> _existingAccountIndexes(String masterId) async {
    final rows = await WalletIsar.instance.read((isar) {
      return isar.accountEntitys.filter().masterIdEqualTo(masterId).findAll();
    });
    return rows.map((row) => row.accountIndex).toSet();
  }

  Future<WalletProfile> _requireHotWalletProfileByMasterId(
    String masterId,
  ) async {
    final row = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys
          .filter()
          .masterIdEqualTo(masterId)
          .findFirst();
    });
    if (row == null) {
      throw const WalletAuthException('未找到指定钱包');
    }
    final profile = _toProfile(row);
    if (profile.isColdWallet) {
      throw const WalletAuthException('当前钱包为冷钱包，请使用扫码签名');
    }
    return profile;
  }

  /// 一台设备仅允许一个热钱包(冷钱包不限)。createWallet / importWallet 前置。
  Future<void> _ensureNoExistingHotWallet() async {
    final wallets = await getWallets();
    if (wallets.any((wallet) => wallet.isHotWallet)) {
      throw Exception('本设备已存在热钱包,一台设备仅支持一个热钱包');
    }
  }

  String _defaultAccountName(int accountIndex) => '账户$accountIndex';

  Account _toAccount(AccountEntity row) => Account(
        masterId: row.masterId,
        accountIndex: row.accountIndex,
        accountId: row.accountId,
        ss58Address: row.ss58Address,
        accountName: row.accountName,
      );

  // 删除
  Future<void> clearWallet() async {
    final wallets = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys.where().findAll();
    });
    final accounts = await WalletIsar.instance.read((isar) {
      return isar.accountEntitys.where().findAll();
    });
    await WalletIsar.instance.writeTxn((isar) async {
      for (final wallet in wallets) {
        for (final key in _contactCacheKeys(wallet.accountId)) {
          await isar.appKvEntitys.deleteByKey(key);
        }
      }
      await isar.walletProfileEntitys.clear();
      // 清空钱包 = 清空全部账户行。
      await isar.accountEntitys.clear();
      // 钱包被清空时，本机从钱包进入 App 后记录的交易流水也一并清空。
      await isar.localTxEntitys.clear();
      await isar.walletTxSyncCursorEntitys.clear();
      final settings = await _getSettingsInTxn(isar);
      settings.activeWalletIndex = null;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });

    // 身份在事务提交时即已切换,广播必须先于安全存储清理:
    // deleteAccountKey 可能抛错(Keystore 不可用),若 bump 放在其后会被跳过,
    // 常驻页面将停留在已删除的旧身份上。
    _bumpWalletsRevision();

    // masterId → walletIndex(child KEK 的绑定键),据此清每个账户的硬件金库 child。
    final walletIndexByMaster = <String, int>{
      for (final wallet in wallets)
        if (wallet.signMode == 'local') wallet.masterId: wallet.walletIndex,
    };
    for (final account in accounts) {
      final walletIndex = walletIndexByMaster[account.masterId];
      if (walletIndex != null) {
        await _store.deleteAccountKey(
            walletIndex: walletIndex, accountId: account.accountId);
      }
    }
    for (final row in wallets) {
      if (row.signMode == 'local') {
        await _deleteContactKeys(row.accountId);
      }
    }
  }

  Future<void> deleteWallet(int walletIndex) async {
    final target = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
    });
    if (target == null) {
      throw Exception('未找到钱包');
    }

    // 删钱包 = 删这套助记词下的全部账户。先取全部账户 accountId,事务提交后逐个
    // 清硬件金库 child;账户0(= target.accountId)必在其中,兜底再删一次(幂等)。
    final accountRows = await WalletIsar.instance.read((isar) {
      return isar.accountEntitys
          .filter()
          .masterIdEqualTo(target.masterId)
          .findAll();
    });
    final accountIds = <String>{
      for (final row in accountRows) row.accountId,
      target.accountId,
    };

    await WalletIsar.instance.writeTxn((isar) async {
      final current = await isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
      if (current == null) {
        throw Exception('未找到钱包');
      }
      // 删除钱包同时终止该钱包在本机的通讯录生命周期；云端密文仍可在用户
      // 重新导入同一助记词后恢复，但本机缓存、待同步操作和状态不得残留。
      for (final key in _contactCacheKeys(current.accountId)) {
        await isar.appKvEntitys.deleteByKey(key);
      }
      await isar.walletProfileEntitys.delete(current.id);
      // 连带删除该助记词下全部账户行(含账户0),避免钱包已删但账户行残留。
      await isar.accountEntitys
          .filter()
          .masterIdEqualTo(current.masterId)
          .deleteAll();
      // 用户明确删除钱包后，本机交易记录周期结束；再次导入同一地址
      // 会从新的 finalized 区块重新记录，不保留旧本机流水。
      await isar.localTxEntitys
          .filter()
          .accountIdEqualTo(current.accountId)
          .deleteAll();
      await isar.walletTxSyncCursorEntitys
          .filter()
          .accountIdEqualTo(current.accountId)
          .deleteAll();

      final settings = await _getSettingsInTxn(isar);
      if (settings.activeWalletIndex == walletIndex) {
        final remains = await isar.walletProfileEntitys
            .where()
            .sortByWalletIndex()
            .findAll();
        settings.activeWalletIndex =
            remains.isEmpty ? null : remains.last.walletIndex;
        settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.walletSettingsEntitys.put(settings);
      }
    });

    // 同 clearWallet:事务已提交、身份已切换,先广播再做可能抛错的存储清理。
    _bumpWalletsRevision();

    if (target.signMode == 'local') {
      for (final accountId in accountIds) {
        await _store.deleteAccountKey(
            walletIndex: walletIndex, accountId: accountId);
      }
      await _deleteContactKeys(target.accountId);
    }
  }

  // 更新
  Future<void> renameWallet(int walletIndex, String walletName) async {
    await updateWalletDisplay(walletIndex, walletName: walletName);
  }

  Future<void> updateWalletDisplay(
    int walletIndex, {
    String? walletName,
    String? walletIcon,
  }) async {
    if (walletName == null && walletIcon == null) {
      return;
    }

    final nextName = walletName?.trim();
    if (walletName != null && (nextName == null || nextName.isEmpty)) {
      throw Exception('钱包名称不能为空');
    }
    if (walletIcon != null && walletIcon.trim().isEmpty) {
      throw Exception('钱包图标不能为空');
    }

    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
      if (row == null) {
        throw Exception('未找到钱包');
      }
      if (nextName != null) {
        row.walletName = nextName;
      }
      if (walletIcon != null) {
        row.walletIcon = walletIcon.trim();
      }
      await isar.walletProfileEntitys.put(row);
    });
    _bumpWalletsRevision();
  }

  Future<void> setWalletBalance(int walletIndex, double balance) async {
    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
      if (row == null) {
        throw Exception('未找到钱包');
      }
      row.balance = balance;
      await isar.walletProfileEntitys.put(row);
    });
  }

  // 派生（ROOTLESS：母种子只在内存派生账户0，派生后清零，绝不落库）
  /// mnemonic → entropy → PBKDF2 → 64 字节 → 前 32 字节母种子（master mini-secret）。
  ///
  /// 使用 Substrate 特定的 BIP39 派生（非标准 BIP32），与
  /// `polkadart_keyring` 的 `fromMnemonic` 内部逻辑一致。返回可清零的 [Uint8List]。
  Future<Uint8List> _mnemonicToMiniSecret(String mnemonic) async {
    final entropy =
        bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
    return Uint8List.fromList(
        await CryptoScheme.miniSecretFromEntropy(entropy));
  }

  /// 助记词 → 母种子 → 账户0(`//0`)；母种子派生完成后立即清零。
  ///
  /// 返回的 [_Account0] 持有账户0 的 child mini-secret 与公开身份，供上层存储、
  /// 派生通讯录钥、注册设备子钥；用完须调 [_Account0.dispose] 清零 child。
  Future<_Account0> _deriveAccount0FromMnemonic(String mnemonic) async {
    final seed = await _mnemonicToMiniSecret(mnemonic);
    try {
      return _deriveAccount(seed, 0);
    } finally {
      seed.fillRange(0, seed.length, 0);
    }
  }

  /// 从母种子硬派生 `//index` 子密钥的 child mini-secret（32B），逐字节对齐
  /// `derivation_golden_test.dart` 金标与 citizenwallet 冷端。
  List<int> _childMiniSecret(List<int> seed, int index) {
    final junctions = SecretUri.fromStr('//$index').junctions;
    var rootSk = sr.MiniSecretKey.fromRawKey(seed).expandEd25519();
    late List<int> child;
    for (final j in junctions) {
      final cc = j.junctionId.sublist(0, 32);
      final derived = rootSk.hardDeriveMiniSecretKey(const <int>[], cc);
      child = derived.$1.encode();
      rootSk = derived.$1.expandEd25519();
    }
    return child;
  }

  /// 母种子 → 账户[index]（child mini-secret + 公开身份）。
  ///
  /// 账户0 = `//0`（无 bare 根）；`fromSeed(childMiniSecret)` 逐字节等于
  /// `<助记词>//index`。账户0 的 accountId 即钱包身份（S7.1 interim identity）。
  _Account0 _deriveAccount(List<int> seed, int index) {
    final child = Uint8List.fromList(_childMiniSecret(seed, index));
    final pair = Keyring.sr25519.fromSeed(child);
    pair.ss58Format = _ss58Format;
    final publicKeyBytes = pair.bytes().toList(growable: false);
    return _Account0(
      childMiniSecret: child,
      accountId: _accountIdFromBytes(publicKeyBytes),
      ss58Address: pair.address,
    );
  }
  // 签名（child mini-secret 绑定硬件，经 SecureSeedStore；私钥材料不出类）

  /// 用账户0 私钥对 [payload] 签名。
  ///
  /// 统一签名入口：动钱动权（转账 / 投票 / 切换默认身份 / 发布动态）
  /// 一律走此方法。读硬件金库 child 时由硬件 + 生物识别原子解锁（一次操作一次
  /// 验证），派生后用后即弃。广场 / Chat 后台握手统一使用 P-256 设备子钥。
  Future<Uint8List> signWithWallet(
    int walletIndex,
    Uint8List payload,
  ) async {
    final pair = await _loadSigningKey(walletIndex);
    return Uint8List.fromList(pair.sign(payload));
  }

  /// 校验用户身份（弹一次生物识别）并确认能解锁指定热钱包，供「切换默认
  /// 身份」等无签名负载、但属动权、需先验证的场景。验证失败上抛，成功即返回。
  Future<void> verifyWalletAccess(int walletIndex) async {
    // 读硬件金库 child 即触发一次生物识别；成功解锁即视为通过。
    await _loadSigningKey(walletIndex);
  }

  /// 读取当前热钱包的通讯录专用密钥。新建/导入钱包时已经用内存中的账户0 child
  /// 预派生；老钱包第一次进入通讯录时才读取一次硬件金库 child（触发生物识别）
  /// 并持久化。
  Future<ContactKeyMaterial> ensureContactKeyMaterial({
    required int walletIndex,
    required String accountId,
  }) async {
    final profile = await _requireHotWalletProfile(walletIndex);
    if (profile.accountId != accountId) {
      throw const WalletAuthException('通讯录账户与当前钱包不一致');
    }
    final stored = await _readContactKeys(accountId);
    if (stored != null) return stored;

    final childHex = await _readAccountKeyOrThrow(walletIndex, accountId);
    final child = Uint8List.fromList(_hexToBytes(childHex));
    try {
      final derived = await _deriveContactKeys(accountId, child);
      await _writeContactKeys(accountId, derived);
      return derived;
    } finally {
      child.fillRange(0, child.length, 0);
    }
  }

  static String _contactKeyName(String accountId) =>
      'wallet_contacts_key_v1_$accountId';

  static List<String> _contactCacheKeys(String accountId) => <String>[
        'contacts:$accountId',
        'contact_pending_ops:$accountId',
        'contact_sync_state:$accountId',
      ];

  static Future<void> _persistContactKeys(
    String accountId,
    List<int> accountSecret,
  ) async {
    final material = await _deriveContactKeys(accountId, accountSecret);
    await _writeContactKeys(accountId, material);
  }

  /// 从账户0 的 child mini-secret 域隔离派生通讯录加密钥 / 索引钥（HKDF）。
  static Future<ContactKeyMaterial> _deriveContactKeys(
    String accountId,
    List<int> accountSecret,
  ) async {
    // 先哈希 accountId 形成固定 32 字节 salt，严格对应通讯录密码学契约。
    final salt = (await Sha256().hash(utf8.encode(accountId))).bytes;
    Future<Uint8List> derive(String info) async {
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

    return ContactKeyMaterial(
      encryptionKey: await derive('citizenapp.contacts.v1/encryption'),
      indexKey: await derive('citizenapp.contacts.v1/index'),
    );
  }

  static Future<ContactKeyMaterial?> _readContactKeys(
    String accountId,
  ) async {
    final raw = await _contactKeyStore.read(_contactKeyName(accountId));
    if (raw == null || raw.isEmpty) return null;
    try {
      final bytes = base64Decode(raw);
      if (bytes.length != 64) return null;
      return ContactKeyMaterial(
        encryptionKey: Uint8List.fromList(bytes.sublist(0, 32)),
        indexKey: Uint8List.fromList(bytes.sublist(32)),
      );
    } on FormatException {
      return null;
    }
  }

  static Future<void> _writeContactKeys(
    String accountId,
    ContactKeyMaterial material,
  ) {
    final bytes = Uint8List(64)
      ..setAll(0, material.encryptionKey)
      ..setAll(32, material.indexKey);
    return _contactKeyStore.write(
      _contactKeyName(accountId),
      base64Encode(bytes),
    );
  }

  static Future<void> _deleteContactKeys(String accountId) =>
      _contactKeyStore.delete(_contactKeyName(accountId));

  /// 读严档账户0 child → 派生并校验 sr25519 密钥对。
  Future<KeyPair> _loadSigningKey(int walletIndex) async {
    final profile = await _requireHotWalletProfile(walletIndex);
    final childHex =
        await _readAccountKeyOrThrow(walletIndex, profile.accountId);
    return _keyPairFromChildHex(childHex, profile);
  }

  /// 读严档 child（触发生物识别）；fail-closed（无根 = 无自愈）。
  ///
  /// - 用户取消 / 超时（[AuthCancelled]）、无锁屏（[NoDeviceCredential]）、金库
  ///   不可用（[SecureStoreUnavailable]）直接上抛，由上层文案区分。
  /// - KEK 失效（[SeedKeyInvalidated]）或条目缺失 → 抛
  ///   [WalletAuthException]，提示用户用助记词重新导入钱包。
  Future<String> _readAccountKeyOrThrow(
    int walletIndex,
    String accountId,
  ) async {
    try {
      final childHex = await _store.readAccountKey(
        walletIndex: walletIndex,
        accountId: accountId,
      );
      if (childHex == null) {
        throw const WalletAuthException('密钥不可用，请重新导入钱包');
      }
      return childHex;
    } on SeedKeyInvalidated {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }
  }

  /// child hex → sr25519 KeyPair，校验派生公钥与 profile 一致。
  KeyPair _keyPairFromChildHex(String childHex, WalletProfile profile) {
    final childBytes = Uint8List.fromList(_hexToBytes(childHex));
    try {
      // fromSeed 会把 child 展开成独立 SecretKey（不引用输入字节），因此派生后
      // 立即把本地 child 副本清零，缩短明文私钥材料在内存中的存活窗口。
      final pair = Keyring.sr25519.fromSeed(childBytes);
      pair.ss58Format = profile.ss58;
      final localAccountId =
          _accountIdFromBytes(pair.bytes().toList(growable: false));
      if (localAccountId != profile.accountId) {
        throw const WalletAuthException('本地签名密钥与当前钱包不一致，请重新导入钱包');
      }
      return pair;
    } finally {
      childBytes.fillRange(0, childBytes.length, 0);
    }
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

  /// 获取钱包私钥（账户0 child mini-secret hex），读严档金库触发生物识别。
  /// 无根模型下「私钥」= 账户0 的签名 child 钥。仅用于「查看私钥」。
  Future<String?> getSeedHex(int walletIndex) async {
    final profile = await _requireHotWalletProfile(walletIndex);
    return _readAccountKeyOrThrow(walletIndex, profile.accountId);
  }

  /// 注册 P-256 设备子钥（硬绑定）：用**内存里的账户0 child 派生的 sr25519
  /// keypair** 对绑定证明签名（零额外弹窗）。**fail-closed**：注册失败向上抛，由
  /// createWallet / importWallet 连同钱包一起回滚——绝不留"建了钱包却没注册"的
  /// 中间态。registrar 未接线（测试）时跳过。
  Future<void> _registerDeviceSubkey(
    int walletIndex,
    String accountId,
    List<int> childMiniSecret,
  ) async {
    final registrar = _subkeyRegistrar;
    if (registrar == null) {
      return;
    }
    await registrar(
      walletIndex: walletIndex,
      accountId: accountId,
      signBinding: (message) async {
        final pair =
            Keyring.sr25519.fromSeed(Uint8List.fromList(childMiniSecret));
        pair.ss58Format = _ss58Format;
        final signature = pair.sign(message);
        return '0x${_toHex(signature.toList(growable: false))}';
      },
    );
  }

  /// 前置检查：设备必须有锁屏（生物识别 / 数字 / 图案 / PIN），否则拒绝
  /// 创建 / 导入热钱包（D3 fail-closed）。
  Future<void> _ensureDeviceSecure() async {
    final status = await _store.authStatus();
    if (status == SecureAuthStatus.noDeviceLock) {
      throw const WalletAuthException(
        '请先在系统设置中启用屏幕锁定（数字密码、图案或生物识别），才能创建或导入热钱包。',
      );
    }
  }

  // 内部工具
  /// 检查账户 ID 是否已存在，重复则抛出异常。
  Future<void> _checkDuplicateAccountId(String accountId) async {
    final normalized = _normalizeAccountId(accountId);
    final rows = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys.where().findAll();
    });
    for (final row in rows) {
      if (row.accountId == normalized) {
        throw Exception('该钱包已存在（${row.walletName}），无需重复导入');
      }
    }
  }

  /// 原子化创建热钱包：在同一个事务中分配 walletIndex 并写入数据库，
  /// 事务成功后再把账户0 的 child mini-secret 写入 secure storage，避免并发时
  /// index 冲突或密钥覆盖。
  Future<WalletProfile> _appendHotWalletAtomic({
    required _Account0 account0,
    required String source,
  }) async {
    final ss58Address = account0.ss58Address;
    final accountId = account0.accountId;
    late int walletIndex;
    late int createdAtMillis;
    await WalletIsar.instance.writeTxn((isar) async {
      final rows =
          await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
      final used = rows.map((e) => e.walletIndex).toSet();
      walletIndex = 1;
      while (used.contains(walletIndex)) {
        walletIndex++;
      }
      final sortOrder = rows.fold<int>(
            -1,
            (maximum, row) => row.sortOrder > maximum ? row.sortOrder : maximum,
          ) +
          1;
      createdAtMillis = DateTime.now().millisecondsSinceEpoch;

      final normalizedAccountId = _normalizeAccountId(accountId);
      final entity = WalletProfileEntity()
        ..walletIndex = walletIndex
        ..walletName = _defaultWalletName(walletIndex)
        ..walletIcon = _defaultWalletIcon()
        ..balance = 0
        ..ss58Address = ss58Address
        ..accountId = normalizedAccountId
        ..masterId = normalizedAccountId
        ..alg = 'sr25519'
        ..ss58 = _ss58Format
        ..createdAtMillis = createdAtMillis
        ..source = source
        ..signMode = 'local'
        ..sortOrder = sortOrder;
      await isar.walletProfileEntitys.put(entity);

      // 账户0(`//0`)与钱包同事务落库,masterId = 账户0.accountId;它是锚点,
      // 让账户0 也出现在 getAccounts,并成为后续追加账户的 masterId 归属。
      final account0Entity = AccountEntity()
        ..masterId = normalizedAccountId
        ..accountIndex = 0
        ..accountId = normalizedAccountId
        ..ss58Address = ss58Address
        ..accountName = _defaultAccountName(0)
        ..createdAtMillis = createdAtMillis;
      await isar.accountEntitys.put(account0Entity);

      final settings = await _getSettingsInTxn(isar);
      settings.activeWalletIndex = walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });

    final normalizedAccountId = _normalizeAccountId(accountId);
    final profile = WalletProfile(
      walletIndex: walletIndex,
      walletName: _defaultWalletName(walletIndex),
      walletIcon: _defaultWalletIcon(),
      balance: 0,
      ss58Address: ss58Address,
      accountId: normalizedAccountId,
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: createdAtMillis,
      source: source,
      signMode: 'local',
    );
    try {
      await _store.putAccountKey(
        walletIndex: walletIndex,
        accountId: normalizedAccountId,
        childMiniSecretHex: _toHex(account0.childMiniSecret),
      );
      await _verifyWalletPersisted(profile);
    } catch (_) {
      await _rollbackWalletCreation(walletIndex);
      rethrow;
    }
    return profile;
  }

  /// 原子化创建冷钱包：在同一个事务中分配 walletIndex 并写入数据库。
  Future<WalletProfile> _appendColdWalletAtomic({
    required String ss58Address,
    required String accountId,
  }) async {
    late int walletIndex;
    late int createdAtMillis;
    await WalletIsar.instance.writeTxn((isar) async {
      final rows =
          await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
      final used = rows.map((e) => e.walletIndex).toSet();
      walletIndex = 1;
      while (used.contains(walletIndex)) {
        walletIndex++;
      }
      final sortOrder = rows.fold<int>(
            -1,
            (maximum, row) => row.sortOrder > maximum ? row.sortOrder : maximum,
          ) +
          1;
      createdAtMillis = DateTime.now().millisecondsSinceEpoch;

      final normalizedAccountId = _normalizeAccountId(accountId);
      final entity = WalletProfileEntity()
        ..walletIndex = walletIndex
        ..walletName = _defaultWalletName(walletIndex)
        ..walletIcon = _defaultWalletIcon()
        ..balance = 0
        ..ss58Address = ss58Address
        ..accountId = normalizedAccountId
        // 冷钱包无派生概念,masterId 亦取其 accountId,保持 masterId 字段全表非空。
        ..masterId = normalizedAccountId
        ..alg = 'sr25519'
        ..ss58 = _ss58Format
        ..createdAtMillis = createdAtMillis
        ..source = 'imported'
        ..signMode = 'external'
        ..sortOrder = sortOrder;
      await isar.walletProfileEntitys.put(entity);

      final settings = await _getSettingsInTxn(isar);
      settings.activeWalletIndex = walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });

    final profile = WalletProfile(
      walletIndex: walletIndex,
      walletName: _defaultWalletName(walletIndex),
      walletIcon: _defaultWalletIcon(),
      balance: 0,
      ss58Address: ss58Address,
      accountId: _normalizeAccountId(accountId),
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: createdAtMillis,
      source: 'imported',
      signMode: 'external',
    );
    await _verifyWalletPersisted(profile);
    return profile;
  }

  /// 只能在已经进入写事务时调用；这里绝不再开启嵌套 writeTxn。
  Future<WalletSettingsEntity> _getSettingsInTxn(Isar isar) async {
    final row = await isar.walletSettingsEntitys.get(0);
    if (row != null) {
      return row;
    }
    final created = WalletSettingsEntity()
      ..id = 0
      ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
    await isar.walletSettingsEntitys.put(created);
    return created;
  }

  Future<WalletProfile> _requireHotWalletProfile(int walletIndex) async {
    final row = await WalletIsar.instance.read((isar) {
      return isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
    });
    if (row == null) {
      throw const WalletAuthException('未找到指定钱包');
    }
    final profile = _toProfile(row);
    if (profile.isColdWallet) {
      throw const WalletAuthException('当前钱包为冷钱包，请使用扫码签名');
    }
    return profile;
  }

  /// 创建/导入完成后立即复读本地库，防止 UI 已展示助记词但
  /// 钱包索引没有真正落库；失败时上层会回滚并提示用户重试。
  Future<void> _verifyWalletPersisted(WalletProfile profile) async {
    final persisted = await getWalletByIndex(profile.walletIndex);
    if (persisted == null || persisted.accountId != profile.accountId) {
      throw Exception('钱包写入后校验失败，请重试');
    }
    // 账户0 child 已由 SecureSeedStore 写入并隐式校验（putAccountKey 失败即抛），
    // 此处不再回读，避免创建时额外触发一次生物识别。
  }

  Future<void> _rollbackWalletCreation(int walletIndex) async {
    String? accountId;
    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.walletProfileEntitys
          .filter()
          .walletIndexEqualTo(walletIndex)
          .findFirst();
      if (row != null) {
        accountId = row.accountId;
        await isar.walletProfileEntitys.delete(row.id);
        // 创建阶段只有账户0,按 masterId 删掉其 AccountEntity,避免回滚后残留孤儿账户行。
        await isar.accountEntitys
            .filter()
            .masterIdEqualTo(row.masterId)
            .deleteAll();
      }
      final settings = await _getSettingsInTxn(isar);
      if (settings.activeWalletIndex == walletIndex) {
        final remains = await isar.walletProfileEntitys
            .where()
            .sortByWalletIndex()
            .findAll();
        settings.activeWalletIndex =
            remains.isEmpty ? null : remains.last.walletIndex;
        settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.walletSettingsEntitys.put(settings);
      }
    });
    if (accountId != null) {
      await _deleteContactKeys(accountId!);
      await _store.deleteAccountKey(
          walletIndex: walletIndex, accountId: accountId!);
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
    return '0x${_toHex(bytes)}';
  }

  String _normalizeAccountId(String value) {
    if (!isAccountIdText(value)) {
      throw ArgumentError.value(
        value,
        'accountId',
        '账户 ID 必须是小写 0x 加 64 位十六进制',
      );
    }
    return value;
  }

  String _defaultWalletName(int walletIndex) {
    return '钱包$walletIndex';
  }

  String _defaultWalletIcon() {
    return 'wallet';
  }

  WalletProfile _toProfile(WalletProfileEntity row) {
    return WalletProfile(
      walletIndex: row.walletIndex,
      walletName: row.walletName,
      walletIcon: row.walletIcon,
      balance: row.balance,
      ss58Address: row.ss58Address,
      accountId: row.accountId,
      alg: row.alg,
      ss58: row.ss58,
      createdAtMillis: row.createdAtMillis,
      source: row.source,
      signMode: row.signMode,
      sortOrder: row.sortOrder,
    );
  }
}

/// 账户0(`//0`) 派生结果（ROOTLESS）：持有 child mini-secret 与公开身份。
///
/// [childMiniSecret] 是可清零的 [Uint8List]（明文私钥材料），上层用完须调
/// [dispose] 立即清零，缩短明文在内存中的存活窗口。
class _Account0 {
  _Account0({
    required this.childMiniSecret,
    required this.ss58Address,
    required this.accountId,
  });

  final Uint8List childMiniSecret;
  final String ss58Address;
  final String accountId;

  /// 清零 child mini-secret 明文。
  void dispose() {
    childMiniSecret.fillRange(0, childMiniSecret.length, 0);
  }
}
