import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

// 发布会员体系后，`SquareApiClient._headers` 对带 session 的请求强制要求设备请求签名器，
// 缺失即抛「设备请求签名器缺失」。测试用固定假签名占位；MockClient 不校验签名头。
SquareSession _session() => SquareSession(
      sessionToken: 'sqs_test',
      ownerAccount: 'owner',
      expiresAt: 1800000000000,
      signRequest: (_) async => 'test-device-signature',
    );

void main() {
  test('SquareApiClient 解析 Worker feed 动态和媒体元数据', () async {
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        expect(request.url.path, '/v1/square/feed/recommended');
        return http.Response(
          '''
          {
            "ok": true,
            "feed_kind": "recommended",
            "posts": [
              {
                "post_id": "sqp_001",
                "owner_account": "owner_001",
                "cid_number": "CN001-CTZN-000000001-2026",
                "post_category": "campaign",
                "text": "竞选动态",
                "content_hash": "0x1111",
                "storage_receipt_id": "sqr_001",
                "chain_block": 88,
                "created_at": 1800000000000,
                "post_state": "published",
                "media_items": [
                  {
                    "media_kind": "image",
                    "provider": "cloudflare_images",
                    "provider_asset_id": "img_001",
                    "asset_state": "ready",
                    "url": "https://imagedelivery.net/account/img_001/public",
                    "byte_size": 1024
                  }
                ]
              }
            ]
          }
          ''',
          200,
          headers: {'content-type': 'application/json'},
        );
      }),
    );

    final posts = await client.fetchFeed(feedKind: SquareFeedKind.recommended);

    expect(posts, hasLength(1));
    expect(posts.first.postCategory, SquarePostCategory.campaign);
    expect(posts.first.author.cidNumber, 'CN001-CTZN-000000001-2026');
    expect(posts.first.chainBlock, 88);
    expect(posts.first.mediaItems.single.mediaKind, SquareMediaKind.image);
    expect(posts.first.mediaItems.single.url,
        'https://imagedelivery.net/account/img_001/public');
    expect(posts.first.mediaItems.single.byteSize, 1024);
  });

  test('SquareApiConfig 只允许 HTTPS 或本地调试 HTTP', () {
    expect(
      SquareApiConfig.normalizeBaseUrl('https://square.example/'),
      'https://square.example',
    );
    expect(
      SquareApiConfig.normalizeBaseUrl('http://127.0.0.1:8787/'),
      'http://127.0.0.1:8787',
    );
    expect(
      () => SquareApiConfig.normalizeBaseUrl('http://square.example'),
      throwsUnsupportedError,
    );
  });

  test('SquareApiClient prepareUpload 发送内容形态和额度声明', () async {
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        expect(request.url.path, '/v1/square/uploads/prepare');
        expect(request.headers['authorization'], 'Bearer sqs_test');
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['post_category'], 'campaign');
        expect(body['content_format'], 'article');
        expect(body['title_length'], 12);
        expect(body['text_length'], 30000);
        return http.Response(
          jsonEncode({
            'ok': true,
            'upload_id': 'squ_test',
            'post_id': 'sqp_test',
            'storage_receipt_id': 'sqr_test',
            'expires_at': 1800000000000,
            'estimated_bytes': 1024,
            'manifest_object_key': 'square/owner/posts/sqp/manifest.json',
            'manifest_upload_url': 'https://r2.test/manifest',
            'media_items': [
              {
                'media_kind': 'image',
                'content_type': 'image/jpeg',
                'byte_size': 1024,
                'provider': 'cloudflare_images',
                'provider_asset_id': 'img_test',
                'upload_method': 'worker',
                'upload_url': 'https://upload.test/image',
              }
            ],
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      }),
    );

    final prepared = await client.prepareUpload(
      session: _session(),
      postCategory: SquarePostCategory.campaign,
      contentFormat: SquarePostContentFormat.article,
      titleLength: 12,
      textLength: 30000,
      manifestHash: '11' * 32,
      mediaItems: const [
        SquareUploadMediaRequest(
          mediaKind: SquareMediaKind.image,
          contentType: 'image/jpeg',
          byteSize: 1024,
          fileExt: 'jpg',
        ),
      ],
    );

    expect(prepared.postId, 'sqp_test');
  });

  test('SquareApiClient deletePost 使用登录态删除指定动态', () async {
    final client = SquareApiClient(
      baseUrl: 'https://square.test',
      httpClient: MockClient((request) async {
        expect(request.method, 'DELETE');
        expect(request.url.path, '/v1/square/posts/sqp_old');
        expect(request.headers['authorization'], 'Bearer sqs_test');
        return http.Response(
          jsonEncode({'ok': true, 'post_id': 'sqp_old'}),
          200,
          headers: {'content-type': 'application/json'},
        );
      }),
    );

    await client.deletePost(
      session: _session(),
      postId: 'sqp_old',
    );
  });
}
