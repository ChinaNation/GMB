import 'package:citizenapp/8964/models/square_models.dart';

/// 统一发布页的内容类型（头像右侧下拉项）：动态/文章 × 普通/竞选。
///
/// 图片/视频不在此区分——动态子类由用户第一次选中的媒体类型锁定（发布页首选逻辑），
/// 不进下拉。竞选两项仅认证公民可选。
enum SquareComposeType {
  post('动态', SquarePostContentFormat.normal, SquarePostCategory.normal),
  article('文章', SquarePostContentFormat.article, SquarePostCategory.normal),
  campaignPost(
      '竞选动态', SquarePostContentFormat.normal, SquarePostCategory.campaign),
  campaignArticle(
      '竞选文章', SquarePostContentFormat.article, SquarePostCategory.campaign);

  const SquareComposeType(this.label, this.contentFormat, this.category);

  final String label;
  final SquarePostContentFormat contentFormat;
  final SquarePostCategory category;

  bool get isArticle => contentFormat == SquarePostContentFormat.article;
  bool get isCampaign => category == SquarePostCategory.campaign;

  /// 由既有帖的内容形态/档位映射回发布类型（编辑入口用）。
  static SquareComposeType fromPost({
    required bool isArticle,
    required bool isCampaign,
  }) {
    if (isArticle) return isCampaign ? campaignArticle : article;
    return isCampaign ? campaignPost : post;
  }

  /// 下拉可选项：普通用户仅"动态/文章"两项；认证公民多"竞选动态/竞选文章"共四项。
  static List<SquareComposeType> optionsFor({required bool certified}) =>
      certified
          ? const [post, article, campaignPost, campaignArticle]
          : const [post, article];

  /// 认证失效时把竞选类降级到对应普通类（动态↔竞选动态、文章↔竞选文章）。
  SquareComposeType degradedIfNotCertified(bool certified) {
    if (certified || !isCampaign) return this;
    return isArticle ? article : post;
  }
}
