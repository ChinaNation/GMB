import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

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
                    "url": "square/owner/posts/sqp_001/media_001.webp",
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
}
