import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

const String _owner = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';

Map<String, dynamic> _profileJson({
  String displayName = '轻节点',
  String? cidNumber = 'CN001-CTZN-000000001-2026',
  bool isFollowing = false,
  int following = 2,
  int followers = 128,
  int posts = 36,
}) {
  return <String, dynamic>{
    'owner_account': _owner,
    'display_name': displayName,
    'bio': '链上公民',
    'avatar_object_key': 'profile/$_owner/avatar',
    'banner_object_key': null,
    'cid_number': cidNumber,
    'is_certified': cidNumber != null,
    'counts': {'following': following, 'followers': followers, 'posts': posts},
    'is_following': isFollowing,
    'updated_at': 123,
  };
}

SquareApiClient _client(MockClient mock) =>
    SquareApiClient(baseUrl: 'https://example.com', httpClient: mock);

/// http.Response(String) 默认按 Latin1 编码，中文会抛异常；显式声明 utf-8。
http.Response _ok(Map<String, dynamic> body) => http.Response(
      jsonEncode(body),
      200,
      headers: {'content-type': 'application/json; charset=utf-8'},
    );

// `_headers` 对带 session 的请求强制要求设备请求签名器（发布会员体系后新增硬校验）；
// 测试用固定假签名占位，MockClient 不校验签名头。
SquareSession _session() => SquareSession(
      sessionToken: 'tok',
      ownerAccount: _owner,
      expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      signRequest: (_) async => 'test-device-signature',
    );

void main() {
  group('CitizenProfile model', () {
    test('maps counts, certification and follow state from json', () {
      final profile = CitizenProfile.fromJson(
        _profileJson(isFollowing: true),
      );

      expect(profile.ownerAccount, _owner);
      expect(profile.isCertified, isTrue);
      expect(profile.cidNumber, 'CN001-CTZN-000000001-2026');
      expect(profile.isFollowing, isTrue);
      expect(profile.following, 2);
      expect(profile.followers, 128);
      expect(profile.posts, 36);
    });

    test(
        'resolvedDisplayName uses wallet truth, public mirror, then local name',
        () {
      final named = CitizenProfile.fromJson(_profileJson(displayName: '张三'));
      expect(named.resolvedDisplayName('钱包A'), '钱包A');
      expect(named.resolvedDisplayName(''), '张三');

      final unnamed = CitizenProfile.fromJson(_profileJson(displayName: ''));
      expect(unnamed.resolvedDisplayName('钱包A'), '钱包A');
      final fallback = ProfilePresentation.forAccount(_owner).fallbackName;
      expect(unnamed.resolvedDisplayName(''), fallback);
      expect(fallback, isNot(contains(_owner.substring(0, 6))));
    });

    test('local defaults are stable and reject account-derived nicknames', () {
      final first = ProfilePresentation.forAccount(_owner);
      final second = ProfilePresentation.forAccount(_owner);
      final short =
          '${_owner.substring(0, 6)}...${_owner.substring(_owner.length - 6)}';

      expect(second.fallbackName, first.fallbackName);
      expect(second.avatarAsset, first.avatarAsset);
      expect(second.bannerAsset, first.bannerAsset);
      expect(first.avatarAsset, isNot(first.bannerAsset));
      expect(first.resolveDisplayName(publicName: _owner), first.fallbackName);
      expect(first.resolveDisplayName(publicName: short), first.fallbackName);
      expect(ProfilePresentation.assets, hasLength(11));
    });

    test('SquareAuthor never falls back to its wallet account', () {
      const author = SquareAuthor(ownerAccount: _owner, displayName: '');
      expect(author.title, ProfilePresentation.forAccount(_owner).fallbackName);
      expect(author.title, isNot(_owner));
    });

    test('survives a json round-trip', () {
      final original = CitizenProfile.fromJson(_profileJson());
      final restored =
          CitizenProfile.fromJson(jsonDecode(jsonEncode(original.toJson())));
      expect(restored.displayName, original.displayName);
      expect(restored.followers, original.followers);
      expect(restored.avatarObjectKey, original.avatarObjectKey);
    });
  });

  group('CitizenProfileCache', () {
    setUp(() {
      TestWidgetsFlutterBinding.ensureInitialized();
      SharedPreferences.setMockInitialValues({});
    });

    test('round-trips a profile through local storage', () async {
      const cache = CitizenProfileCache();
      final profile = CitizenProfile.fromJson(_profileJson());

      expect(await cache.read(_owner), isNull);
      await cache.write(profile);
      final loaded = await cache.read(_owner);

      expect(loaded, isNotNull);
      expect(loaded!.displayName, '轻节点');
      expect(loaded.followers, 128);
    });

    test('clear removes the cached profile', () async {
      const cache = CitizenProfileCache();
      await cache.write(CitizenProfile.fromJson(_profileJson()));
      await cache.clear(_owner);
      expect(await cache.read(_owner), isNull);
    });
  });

  group('SquareApiClient profile endpoints', () {
    test('fetchUserProfile parses the profile and forwards the session',
        () async {
      String? authHeader;
      final client = _client(MockClient((request) async {
        authHeader = request.headers['authorization'];
        expect(request.url.path, '/v1/square/users/$_owner');
        return _ok({'ok': true, 'profile': _profileJson(isFollowing: true)});
      }));

      final profile = await client.fetchUserProfile(
          ownerAccount: _owner, session: _session());

      expect(authHeader, 'Bearer tok');
      expect(profile.isFollowing, isTrue);
      expect(profile.followers, 128);
    });

    test('fetchAuthorPosts filters by category and returns the cursor',
        () async {
      Uri? seen;
      final client = _client(MockClient((request) async {
        seen = request.url;
        return _ok({
          'ok': true,
          'posts': [
            {
              'post_id': 'c1',
              'owner_account': _owner,
              'post_category': 'campaign',
              'text': '竞选宣言',
              'created_at': 300,
            },
          ],
          'next_cursor': 300,
        });
      }));

      final page = await client.fetchAuthorPosts(
        ownerAccount: _owner,
        category: SquarePostCategory.campaign,
        limit: 2,
      );

      expect(seen!.path, '/v1/square/users/$_owner/posts');
      expect(seen!.queryParameters['category'], 'campaign');
      expect(seen!.queryParameters['limit'], '2');
      expect(page.posts.single.postId, 'c1');
      expect(page.posts.single.postCategory, SquarePostCategory.campaign);
      expect(page.nextCursor, 300);
    });

    test('fetchAuthorPosts parses content_format and title for articles',
        () async {
      final client = _client(MockClient((request) async {
        return _ok({
          'ok': true,
          'posts': [
            {
              'post_id': 'a1',
              'owner_account': _owner,
              'post_category': 'normal',
              'content_format': 'article',
              'title': '我的文章',
              'text': '正文',
              'created_at': 100,
            },
          ],
          'next_cursor': null,
        });
      }));

      final page = await client.fetchAuthorPosts(ownerAccount: _owner);
      final post = page.posts.single;

      expect(post.contentFormat, SquarePostContentFormat.article);
      expect(post.title, '我的文章');
    });

    test('fetchAuthorPosts sends the content_format query', () async {
      Uri? seen;
      final client = _client(MockClient((request) async {
        seen = request.url;
        return _ok({'ok': true, 'posts': [], 'next_cursor': null});
      }));

      await client.fetchAuthorPosts(
        ownerAccount: _owner,
        contentFormat: SquarePostContentFormat.article,
      );

      expect(seen!.queryParameters['content_format'], 'article');
    });

    test('mediaUrl builds an encoded wallet media url', () {
      final client = _client(MockClient((_) async => http.Response('', 200)));
      expect(
        client.mediaUrl('profile/acct/avatar'),
        'https://example.com/v1/square/media/profile/acct/avatar',
      );
    });

    test('updateProfile PUTs only the provided fields', () async {
      String? method;
      Map<String, dynamic>? body;
      final client = _client(MockClient((request) async {
        method = request.method;
        body = jsonDecode(request.body) as Map<String, dynamic>;
        return _ok({'ok': true, 'profile': _profileJson(displayName: '新名字')});
      }));

      final updated = await client.updateProfile(
        session: _session(),
        displayName: '新名字',
      );

      expect(method, 'PUT');
      expect(body, {'display_name': '新名字'});
      expect(updated.displayName, '新名字');
    });
  });
}
