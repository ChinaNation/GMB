import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/security/local_cipher.dart';
import 'package:fixnum/fixnum.dart';
import 'package:isar_community/isar.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/isar_test_env.dart';

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
      {String conversationId = 'conv-1', int at = 1000}) {
    return store.saveIncomingEnvelope(
      ownerCidNumber: ownerCidNumber,
      currentAccountId: accountId,
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
      currentAccountId:
          '0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
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
}
