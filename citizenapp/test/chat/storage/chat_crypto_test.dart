import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/chat/storage/chat_crypto.dart';

void main() {
  group('ChatCrypto.tokenize 字符 bigram 分词', () {
    test('中文按字符 bigram 切分', () {
      expect(ChatCrypto.tokenize('公民钱包'), <String>['公民', '民钱', '钱包']);
    });

    test('英文数字同样走 bigram，支持子串搜索', () {
      expect(ChatCrypto.tokenize('ab1'), <String>['ab', 'b1']);
    });

    test('大小写归一化', () {
      expect(ChatCrypto.tokenize('AbC'), ChatCrypto.tokenize('abc'));
    });

    test('重复 bigram 去重且保持稳定顺序', () {
      // "ababa" 的 bigram 序列是 ab,ba,ab,ba → 去重后 [ab, ba]
      expect(ChatCrypto.tokenize('ababa'), <String>['ab', 'ba']);
    });

    test('不足 2 字符无 token（调用方须回落到扫描）', () {
      expect(ChatCrypto.tokenize('公'), isEmpty);
      expect(ChatCrypto.tokenize(''), isEmpty);
      expect(ChatCrypto.tokenize('   '), isEmpty);
    });

    test('首尾空格被裁掉，不产生含空格的 token', () {
      expect(ChatCrypto.tokenize('  ab  '), <String>['ab']);
    });

    test('查询串的 token 是原文 token 的子集（索引能收窄到正确候选）', () {
      final doc = ChatCrypto.tokenize('今天天气很好').toSet();
      final query = ChatCrypto.tokenize('天气');
      expect(query, isNotEmpty);
      expect(doc.containsAll(query), isTrue);
    });

    test('bigram 命中不等于原串命中：乱序串会成为假阳性', () {
      // 这正是 ChatStore 必须在解密后复验真实子串的原因。
      final doc = ChatCrypto.tokenize('bcab').toSet();
      final query = ChatCrypto.tokenize('abc');
      expect(query, <String>['ab', 'bc']);
      expect(doc.containsAll(query), isTrue,
          reason: '索引层会把 "bcab" 当成 "abc" 的候选，必须靠复验滤掉');
      expect('bcab'.contains('abc'), isFalse);
    });
  });
}
