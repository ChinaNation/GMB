import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/compose/compose_type.dart';
import 'package:citizenapp/8964/models/square_models.dart';

void main() {
  group('SquareComposeType', () {
    test('无竞选身份下拉两项，竞选身份四项', () {
      expect(
        SquareComposeType.optionsFor(canCampaign: false),
        const [SquareComposeType.post, SquareComposeType.article],
      );
      expect(
        SquareComposeType.optionsFor(canCampaign: true),
        const [
          SquareComposeType.post,
          SquareComposeType.article,
          SquareComposeType.campaignPost,
          SquareComposeType.campaignArticle,
        ],
      );
    });

    test('内容形态与档位映射正确', () {
      expect(SquareComposeType.campaignArticle.isArticle, isTrue);
      expect(SquareComposeType.campaignArticle.isCampaign, isTrue);
      expect(SquareComposeType.campaignArticle.contentFormat,
          SquarePostContentFormat.article);
      expect(SquareComposeType.campaignArticle.category,
          SquarePostCategory.campaign);
      expect(SquareComposeType.post.isArticle, isFalse);
      expect(SquareComposeType.post.isCampaign, isFalse);
    });

    test('无竞选身份时竞选类降级到对应普通类，非竞选不变', () {
      expect(
        SquareComposeType.campaignPost.degradedIfNotCampaignEligible(false),
        SquareComposeType.post,
      );
      expect(
        SquareComposeType.campaignArticle.degradedIfNotCampaignEligible(false),
        SquareComposeType.article,
      );
      // 有竞选身份时保持不变。
      expect(
        SquareComposeType.campaignArticle.degradedIfNotCampaignEligible(true),
        SquareComposeType.campaignArticle,
      );
      // 非竞选类不受影响。
      expect(
        SquareComposeType.article.degradedIfNotCampaignEligible(false),
        SquareComposeType.article,
      );
    });
  });
}
