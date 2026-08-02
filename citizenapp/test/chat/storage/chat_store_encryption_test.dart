import 'dart:io';
import 'dart:typed_data';

import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/storage/chat_crypto.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/security/local_cipher.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:fixnum/fixnum.dart';
import 'package:isar_community/isar.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/isar_test_env.dart';

class _HandoverWalletManager extends WalletManager {
  _HandoverWalletManager(this.activeBinding);

  AccountDataBinding activeBinding;

  Uint8List _key(String accountId, LocalKeyPurpose purpose) {
    if (accountId == activeBinding.accountId &&
        activeBinding.bindingRevision == 1) {
      return Uint8List.fromList(debugChatKeys[purpose]!);
    }
    return Uint8List.fromList(List<int>.generate(
      32,
      (index) => (0x80 + purpose.index * 13 + index) & 0xff,
    ));
  }

  @override
  Future<AccountDataBinding> accountDataBindingForAccountId(
    String accountId,
  ) async {
    if (activeBinding.accountId != accountId) {
      throw StateError('测试账户不是当前绑定账户');
    }
    return activeBinding;
  }

  @override
  Future<Uint8List> readDataKeyForCurrentBinding(
    String accountId,
    LocalKeyPurpose purpose, {
    String? context,
  }) async =>
      _key(accountId, purpose);

  @override
  Future<List<Uint8List>> readDataKeysForBinding(
    AccountDataBinding binding,
    List<({String? context, LocalKeyPurpose purpose})> requests,
  ) async =>
      requests
          .map((request) => _key(binding.accountId, request.purpose))
          .toList(growable: false);

  @override
  Future<List<Uint8List>> deriveDataKeysForBindingHandover(
    AccountDataBinding binding,
    List<({String? context, LocalKeyPurpose purpose})> requests,
  ) =>
      readDataKeysForBinding(binding, requests);
}

/// 聊天正文静止态加密 + HMAC 分词搜索的端到端验收。
///
/// 重点不是"能存能取"，而是：**Isar 原始行里不得出现明文**，且加密后搜索
/// 语义与旧的明文 `contains` 完全一致（含假阳性必须被复验滤掉）。
void main() {
  useIsolatedIsar();

  const accountId = '0x'
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const ownerCidNumber = 'CN220-CTZN2-100000001-2026';
  const peerCidNumber = 'CN220-CTZN2-100000002-2026';

  ChatEnvelope envelopeOf({
    required String envelopeId,
    required String conversationId,
    int createdAtMillis = 1000,
  }) {
    return ChatEnvelope()
      ..envelopeId = envelopeId
      ..conversationId = conversationId
      ..senderCidNumber = peerCidNumber
      ..recipientCidNumber = ownerCidNumber
      ..senderDeviceId = 'dev-1'
      ..mlsMessageKind = MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION
      ..createdAtMillis = Int64(createdAtMillis);
  }

  Future<void> saveText(ChatStore store, String envelopeId, String text,
      {String conversationId = 'conv-1',
      int at = 1000,
      String currentAccountId = accountId}) {
    return store.saveIncomingEnvelope(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: currentAccountId,
      envelope: envelopeOf(
        envelopeId: envelopeId,
        conversationId: conversationId,
        createdAtMillis: at,
      ),
      envelopeBytes: const <int>[1, 2, 3],
      messageKind: ChatMessageKind.text,
      plaintext: text,
    );
  }

  test('正文与会话摘要落盘为密文，Isar 原始行不含明文', () async {
    final store = ChatStore();
    const secret = '这是一条不该出现在磁盘上的悄悄话';
    await saveText(store, 'env-1', secret);

    // 绕过 ChatStore 直接查原始行，确认磁盘上没有明文。
    final rows = await WalletIsar.instance.read((isar) async {
      return isar.chatMessageEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
    });
    expect(rows, hasLength(1));
    final row = rows.single;
    expect(row.plaintextCipher, isNotNull);
    expect(row.plaintextCipher, isNot(contains(secret)));
    expect(row.searchTokens, isNotEmpty);
    // 索引里存的是 HMAC 截断值，不得出现任何明文片段
    for (final token in row.searchTokens) {
      expect(secret.contains(token), isFalse);
    }

    final conversations = await WalletIsar.instance.read((isar) async {
      return isar.chatConversationEntitys
          .filter()
          .idGreaterThan(0, include: true)
          .findAll();
    });
    expect(conversations.single.lastMessageCipher, isNot(contains(secret)));
  });

  test('读取侧解密还原，UI 拿到的仍是明文', () async {
    final store = ChatStore();
    const secret = '你好，公民';
    await saveText(store, 'env-1', secret);

    final messages = await store.readMessages(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
      conversationId: 'conv-1',
    );
    expect(messages.single.plaintext, secret);

    final previews = await store.readConversationPreviews(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
    );
    expect(previews.single.lastMessage, secret);
    expect(
      await store.readMessages(
        ownerCidNumber: 'CN220-CTZN2-999999999-2026',
        currentAccountId: accountId,
        conversationId: 'conv-1',
      ),
      isEmpty,
    );
  });

  test('搜索：中文、英文、数字均可子串命中', () async {
    final store = ChatStore();
    await saveText(store, 'env-1', '今天天气很好', at: 1000);
    await saveText(store, 'env-2', 'hello world', at: 2000);
    await saveText(store, 'env-3', 'order 12345', at: 3000);

    Future<List<String>> hit(String q) async {
      final rows = await store.searchMessages(
        ownerCidNumber: ownerCidNumber,
        currentAccountId: accountId,
        keyword: q,
      );
      return rows.map((r) => r.envelopeId).toList()..sort();
    }

    expect(await hit('天气'), <String>['env-1']);
    expect(await hit('ello'), <String>['env-2']);
    expect(await hit('234'), <String>['env-3']);
  });

  test('搜索大小写不敏感', () async {
    final store = ChatStore();
    await saveText(store, 'env-1', 'Hello World');
    final rows = await store.searchMessages(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
      keyword: 'HELLO',
    );
    expect(rows, hasLength(1));
  });

  test('搜索：bigram 假阳性必须被解密复验滤掉', () async {
    final store = ChatStore();
    // "bcab" 的 bigram 含 ab 与 bc，会被索引当成 "abc" 的候选，
    // 但它并不真的包含 "abc"，必须在复验阶段被剔除。
    await saveText(store, 'env-1', 'bcab', at: 1000);
    await saveText(store, 'env-2', 'xabcx', at: 2000);

    final rows = await store.searchMessages(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
      keyword: 'abc',
    );
    expect(rows.map((r) => r.envelopeId), <String>['env-2'],
        reason: '假阳性 bcab 必须被滤掉，只留真正包含 abc 的记录');
  });

  test('搜索：单字符查询无 bigram，仍能通过回落扫描命中', () async {
    final store = ChatStore();
    await saveText(store, 'env-1', '公民钱包');
    final rows = await store.searchMessages(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
      keyword: '钱',
    );
    expect(rows, hasLength(1));
  });

  test('搜索：不命中返回空', () async {
    final store = ChatStore();
    await saveText(store, 'env-1', '今天天气很好');
    final rows = await store.searchMessages(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
      keyword: '完全不相干的词',
    );
    expect(rows, isEmpty);
  });

  test('换错密钥解密必须抛错，不得静默返回空白', () async {
    final store = ChatStore();
    await saveText(store, 'env-1', '机密内容');

    // 直接篡改密文，模拟密钥不匹配/密文损坏
    await WalletIsar.instance.writeTxn((isar) async {
      final row = await isar.chatMessageEntitys
          .getByOwnerCidNumberEnvelopeId(ownerCidNumber, 'env-1');
      row!.plaintextCipher = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
      await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(row);
    });

    await expectLater(
      store.readMessages(
        ownerCidNumber: ownerCidNumber,
        currentAccountId: accountId,
        conversationId: 'conv-1',
      ),
      throwsA(isA<LocalCipherException>()),
    );
  });

  test('当前钱包签名换绑：finalized 前保留此前密文，提交后新钱包解密历史消息', () async {
    const genesisHash =
        '0x1111111111111111111111111111111111111111111111111111111111111111';
    const newAccountId =
        '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
    const source = AccountDataBinding(
      genesisHash: genesisHash,
      cidNumber: ownerCidNumber,
      bindingRevision: 1,
      accountId: accountId,
    );
    const target = AccountDataBinding(
      genesisHash: genesisHash,
      cidNumber: ownerCidNumber,
      bindingRevision: 2,
      accountId: newAccountId,
    );
    final manager = _HandoverWalletManager(source);
    final previousFixedKeys = ChatCrypto.debugFixedKeys;
    ChatCrypto.debugFixedKeys = null;
    try {
      final store = ChatStore(
        crypto: ChatCrypto(walletManager: manager),
      );
      await saveText(store, 'env-handover', '只有内存中出现的交接明文');
      final before = await WalletIsar.instance.read((isar) async => (await isar
              .chatMessageEntitys
              .getByOwnerCidNumberEnvelopeId(ownerCidNumber, 'env-handover'))!
          .plaintextCipher!);

      await store.stageAccountHandover(source: source, target: target);
      final stagedRows = await WalletIsar.instance.read((isar) async => isar
          .appKvEntitys
          .filter()
          .keyStartsWith('chat_handover_by_cid:')
          .findAll());
      expect(stagedRows, hasLength(1));
      expect(stagedRows.single.stringValue, isNot(contains('只有内存中出现的交接明文')));
      final stillSource = await WalletIsar.instance.read((isar) async =>
          (await isar.chatMessageEntitys.getByOwnerCidNumberEnvelopeId(
                  ownerCidNumber, 'env-handover'))!
              .plaintextCipher!);
      expect(stillSource, before, reason: 'finalized 前正式消息行不得切换');

      await store.commitAccountHandover(source: source, target: target);
      manager.activeBinding = target;
      final committed = await WalletIsar.instance.read((isar) async =>
          (await isar.chatMessageEntitys
              .getByOwnerCidNumberEnvelopeId(ownerCidNumber, 'env-handover'))!);
      final after = committed.plaintextCipher!;
      expect(after, isNot(before));
      expect(committed.bindingRevision, target.bindingRevision);
      expect(committed.accountId, target.accountId);
      expect(
        (await store.readMessages(
          ownerCidNumber: ownerCidNumber,
          currentAccountId: newAccountId,
          conversationId: 'conv-1',
        ))
            .single
            .plaintext,
        '只有内存中出现的交接明文',
      );
      expect(
        await WalletIsar.instance.read((isar) async => isar.appKvEntitys
            .filter()
            .keyStartsWith('chat_handover_by_cid:')
            .count()),
        0,
      );
    } finally {
      ChatCrypto.debugFixedKeys = previousFixedKeys;
    }
  });

  test('无当前账户签名换绑：此前聊天密文保留但新绑定不可见，当前密文损坏仍上抛', () async {
    const genesisHash =
        '0x1111111111111111111111111111111111111111111111111111111111111111';
    const newAccountId =
        '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
    const source = AccountDataBinding(
      genesisHash: genesisHash,
      cidNumber: ownerCidNumber,
      bindingRevision: 1,
      accountId: accountId,
    );
    const target = AccountDataBinding(
      genesisHash: genesisHash,
      cidNumber: ownerCidNumber,
      bindingRevision: 2,
      accountId: newAccountId,
    );
    final manager = _HandoverWalletManager(source);
    final previousFixedKeys = ChatCrypto.debugFixedKeys;
    ChatCrypto.debugFixedKeys = null;
    try {
      final store = ChatStore(crypto: ChatCrypto(walletManager: manager));
      await saveText(store, 'env-inaccessible', '没有交接签名的历史内容');
      final before = await WalletIsar.instance.read((isar) async => (await isar
          .chatMessageEntitys
          .getByOwnerCidNumberEnvelopeId(ownerCidNumber, 'env-inaccessible'))!);

      manager.activeBinding = target;
      expect(
        await store.readMessages(
          ownerCidNumber: ownerCidNumber,
          currentAccountId: newAccountId,
          conversationId: 'conv-1',
        ),
        isEmpty,
      );
      final preserved = await WalletIsar.instance.read((isar) async =>
          (await isar.chatMessageEntitys.getByOwnerCidNumberEnvelopeId(
              ownerCidNumber, 'env-inaccessible'))!);
      expect(preserved.plaintextCipher, before.plaintextCipher);
      expect(preserved.bindingRevision, source.bindingRevision);
      expect(preserved.accountId, source.accountId);

      await saveText(
        store,
        'env-current',
        '当前绑定密文',
        currentAccountId: newAccountId,
      );
      await WalletIsar.instance.writeTxn((isar) async {
        final current = await isar.chatMessageEntitys
            .getByOwnerCidNumberEnvelopeId(ownerCidNumber, 'env-current');
        current!.plaintextCipher = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
        await isar.chatMessageEntitys.putByOwnerCidNumberEnvelopeId(current);
      });
      await expectLater(
        store.readMessages(
          ownerCidNumber: ownerCidNumber,
          currentAccountId: newAccountId,
          conversationId: 'conv-1',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    } finally {
      ChatCrypto.debugFixedKeys = previousFixedKeys;
    }
  });

  test('双签交接提交后原子把附件与 MLS 文件树切到新绑定分区', () async {
    const genesisHash =
        '0x1111111111111111111111111111111111111111111111111111111111111111';
    const newAccountId =
        '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
    const source = AccountDataBinding(
      genesisHash: genesisHash,
      cidNumber: ownerCidNumber,
      bindingRevision: 1,
      accountId: accountId,
    );
    const target = AccountDataBinding(
      genesisHash: genesisHash,
      cidNumber: ownerCidNumber,
      bindingRevision: 2,
      accountId: newAccountId,
    );
    final root = await Directory.systemTemp.createTemp('gmb-chat-binding-');
    addTearDown(() async {
      if (root.existsSync()) await root.delete(recursive: true);
    });
    final sourceDirectory = Directory(
      '${root.path}/chat/by_cid/$ownerCidNumber/by_binding/'
      '${source.bindingRevision}/${source.accountId}',
    );
    await sourceDirectory.create(recursive: true);
    await File('${sourceDirectory.path}/cipher-marker.bin')
        .writeAsBytes(const <int>[1, 2, 3]);
    final runtime = ChatRuntime(
      store: ChatStore(),
      documentsDirectoryProvider: () async => root,
    );

    await runtime.commitAccountHandover(source: source, target: target);

    final targetDirectory = Directory(
      '${root.path}/chat/by_cid/$ownerCidNumber/by_binding/'
      '${target.bindingRevision}/${target.accountId}',
    );
    expect(sourceDirectory.existsSync(), isFalse);
    expect(
        File('${targetDirectory.path}/cipher-marker.bin').existsSync(), isTrue);
  });
}
