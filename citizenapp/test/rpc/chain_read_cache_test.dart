// ChainReadCache 单测(ADR-018 卡⑤)。
//
// 纯逻辑覆盖(不依赖 smoldot):命中/换块/门控/负缓存/forceFresh/in-flight 合并/invalidate。
// finalizedHash 与底层下沉都用注入的 fake,now 用注入时钟避免真实时间依赖。

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/rpc/chain_read_cache.dart';

void main() {
  final cache = ChainReadCache.instance;

  setUp(() {
    cache.invalidate();
    cache.recheckInterval = const Duration(seconds: 15);
  });

  Uint8List bytesOf(int v) => Uint8List.fromList([v]);

  test('命中:同命名空间下同 key 第二次读不再下沉', () async {
    var fetchCount = 0;
    Future<Map<String, Uint8List?>> fetch(List<String> ks) async {
      fetchCount++;
      return {for (final k in ks) k: bytesOf(1)};
    }

    final now = DateTime(2026, 6, 13);
    final r1 = await cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);
    final r2 = await cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);

    expect(r1['a'], bytesOf(1));
    expect(r2['a'], bytesOf(1));
    expect(fetchCount, 1);
  });

  test('换块:finalizedHash 变化清空缓存并重新下沉', () async {
    var fetchCount = 0;
    Future<Map<String, Uint8List?>> fetch(List<String> ks) async {
      fetchCount++;
      return {for (final k in ks) k: bytesOf(fetchCount)};
    }

    await cache.read(['a'],
        finalizedHashProvider: () async => 'H1',
        fetchMissing: fetch,
        now: DateTime(2026, 6, 13, 0, 0, 0));
    final r = await cache.read(['a'],
        finalizedHashProvider: () async => 'H2',
        fetchMissing: fetch,
        now: DateTime(2026, 6, 13, 0, 1, 0));

    expect(fetchCount, 2);
    expect(r['a'], bytesOf(2));
  });

  test('门控:间隔内不复查 finalizedHash(provider 不被再调)', () async {
    var providerCalls = 0;
    var fetchCount = 0;
    Future<String?> provider() async {
      providerCalls++;
      return 'H1';
    }

    Future<Map<String, Uint8List?>> fetch(List<String> ks) async {
      fetchCount++;
      return {for (final k in ks) k: bytesOf(1)};
    }

    final t0 = DateTime(2026, 6, 13, 0, 0, 0);
    await cache.read(['a'],
        finalizedHashProvider: provider, fetchMissing: fetch, now: t0);
    await cache.read(['b'],
        finalizedHashProvider: provider,
        fetchMissing: fetch,
        now: t0.add(const Duration(seconds: 5)));

    expect(providerCalls, 1);
    expect(fetchCount, 2);
  });

  test('负缓存:key 缺失也缓存,不重复下沉', () async {
    var fetchCount = 0;
    Future<Map<String, Uint8List?>> fetch(List<String> ks) async {
      fetchCount++;
      return {for (final k in ks) k: null};
    }

    final now = DateTime(2026, 6, 13);
    final r1 = await cache.read(['x'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);
    await cache.read(['x'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);

    expect(r1.containsKey('x'), isTrue);
    expect(r1['x'], isNull);
    expect(fetchCount, 1);
  });

  test('forceFresh:旁路缓存强制重新下沉', () async {
    var fetchCount = 0;
    Future<Map<String, Uint8List?>> fetch(List<String> ks) async {
      fetchCount++;
      return {for (final k in ks) k: bytesOf(fetchCount)};
    }

    final now = DateTime(2026, 6, 13);
    await cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);
    final r = await cache.read(['a'],
        finalizedHashProvider: () async => 'H1',
        fetchMissing: fetch,
        now: now,
        forceFresh: true);

    expect(fetchCount, 2);
    expect(r['a'], bytesOf(2));
  });

  test('in-flight 合并:并发同 key 只下沉一次', () async {
    var fetchCount = 0;
    final gate = Completer<Map<String, Uint8List?>>();
    Future<Map<String, Uint8List?>> fetch(List<String> ks) {
      fetchCount++;
      return gate.future;
    }

    final now = DateTime(2026, 6, 13);
    final f1 = cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);
    final f2 = cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);
    gate.complete({'a': bytesOf(7)});

    expect((await f1)['a'], bytesOf(7));
    expect((await f2)['a'], bytesOf(7));
    expect(fetchCount, 1);
  });

  test('invalidate 清空缓存', () async {
    var fetchCount = 0;
    Future<Map<String, Uint8List?>> fetch(List<String> ks) async {
      fetchCount++;
      return {for (final k in ks) k: bytesOf(1)};
    }

    final now = DateTime(2026, 6, 13);
    await cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);
    cache.invalidate();
    await cache.read(['a'],
        finalizedHashProvider: () async => 'H1', fetchMissing: fetch, now: now);

    expect(fetchCount, 2);
  });
}
