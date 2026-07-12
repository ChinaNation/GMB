import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

/// 用户主页数据层门面：把主页资料、按作者拉帖、更新资料收敛到一个入口。
///
/// 网络细节（登录态、解析、Worker 地址）复用 [SquareApiClient]，本类只做语义聚合。
class CitizenProfileApi {
  CitizenProfileApi({SquareApiClient? client})
      : _client = client ?? SquareApiClient();

  final SquareApiClient _client;

  /// R2 object_key → 钱包 session 保护的资料媒体 URL。
  String mediaUrl(String objectKey) => _client.mediaUrl(objectKey);

  /// 拉取主页资料；[session] 存在时响应附带 is_following。
  Future<CitizenProfile> fetchProfile(
    String ownerAccount, {
    SquareSession? session,
  }) {
    return _client.fetchUserProfile(
      ownerAccount: ownerAccount,
      session: session,
    );
  }

  /// 按作者分页拉帖。[category]/[contentFormat] 为空表示不过滤。
  Future<({List<SquarePost> posts, int? nextCursor})> fetchAuthorPosts(
    String ownerAccount, {
    SquarePostCategory? category,
    SquarePostContentFormat? contentFormat,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) {
    return _client.fetchAuthorPosts(
      ownerAccount: ownerAccount,
      category: category,
      contentFormat: contentFormat,
      limit: limit,
      cursor: cursor,
      session: session,
    );
  }

  /// 关注一个账户。
  Future<void> followUser({
    required SquareSession session,
    required String followedAccount,
  }) {
    return _client.followUser(
      session: session,
      followedAccount: followedAccount,
    );
  }

  /// 取消关注一个账户。
  Future<void> unfollowUser({
    required SquareSession session,
    required String followedAccount,
  }) {
    return _client.unfollowUser(
      session: session,
      followedAccount: followedAccount,
    );
  }

  /// 拉取关注/粉丝列表。
  Future<({List<SquareFollowEntry> accounts, int? nextCursor})> fetchFollows(
    String ownerAccount, {
    required String type,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) {
    return _client.fetchFollows(
      ownerAccount: ownerAccount,
      type: type,
      limit: limit,
      cursor: cursor,
      session: session,
    );
  }

  /// 更新本人公开资料，返回更新后的完整主页资料。
  Future<CitizenProfile> updateProfile({
    required SquareSession session,
    String? displayName,
    String? bio,
    String? avatarObjectKey,
    String? avatarContentHash,
    String? bannerObjectKey,
    String? bannerContentHash,
  }) {
    return _client.updateProfile(
      session: session,
      displayName: displayName,
      bio: bio,
      avatarObjectKey: avatarObjectKey,
      avatarContentHash: avatarContentHash,
      bannerObjectKey: bannerObjectKey,
      bannerContentHash: bannerContentHash,
    );
  }
}
