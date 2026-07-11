import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

const String kOwner = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';

SquareSession fakeSession() => SquareSession(
      sessionToken: 'tok',
      ownerAccount: kOwner,
      expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
    );

class FakeSessionProvider extends SquareSessionProvider {
  FakeSessionProvider(this.session) : super();

  final SquareSession? session;

  @override
  Future<SquareSession?> ensureSession() async => session;
}

SquarePost samplePost({
  String id = 'p1',
  SquarePostCategory category = SquarePostCategory.normal,
  SquarePostContentFormat contentFormat = SquarePostContentFormat.normal,
  String? title,
  String text = '内容',
  String displayName = '轻节点',
  List<SquareMediaItem> media = const [],
}) {
  return SquarePost(
    postId: id,
    author: SquareAuthor(
      ownerAccount: kOwner,
      cidNumber: 'CN001-CTZN-000000001-2026',
      displayName: displayName,
    ),
    postCategory: category,
    contentFormat: contentFormat,
    title: title,
    text: text,
    createdAt: DateTime.fromMillisecondsSinceEpoch(1000),
    mediaItems: media,
  );
}

CitizenProfile sampleProfile({
  bool certified = true,
  bool following = false,
  String displayName = '轻节点',
  String bio = '链上公民',
  String owner = kOwner,
  String? avatarKey,
  String? bannerKey,
  String? identityLevel,
  String? membershipLevel,
  bool? membershipActive,
}) {
  return CitizenProfile(
    ownerAccount: owner,
    displayName: displayName,
    bio: bio,
    avatarObjectKey: avatarKey,
    bannerObjectKey: bannerKey,
    cidNumber: certified ? 'CN001-CTZN-000000001-2026' : null,
    isCertified: certified,
    // 认证真源=链上身份档位；默认认证=投票公民（蓝），未认证=访客（无徽章）。
    identityLevel: identityLevel ?? (certified ? 'voting' : 'visitor'),
    // 会员默认未购买（徽章不带勾，显空心环）；传 membershipLevel 才可能带勾。
    membershipLevel: membershipLevel,
    membershipActive: membershipActive ?? (membershipLevel != null),
    following: 2,
    followers: 128,
    posts: 36,
    isFollowing: following,
    updatedAt: 1,
  );
}

class FakeProfileApi extends CitizenProfileApi {
  FakeProfileApi(
    this.result, {
    this.authorPosts = const [],
    this.follows = const [],
    this.throwOnFollow = false,
  }) : super();

  final CitizenProfile result;
  final List<SquarePost> authorPosts;
  final List<SquareFollowEntry> follows;
  final bool throwOnFollow;
  int calls = 0;
  int followCalls = 0;
  int unfollowCalls = 0;
  Map<String, String?>? lastUpdate;

  @override
  Future<CitizenProfile> fetchProfile(
    String ownerAccount, {
    SquareSession? session,
  }) async {
    calls++;
    return result;
  }

  @override
  Future<({List<SquarePost> posts, int? nextCursor})> fetchAuthorPosts(
    String ownerAccount, {
    SquarePostCategory? category,
    SquarePostContentFormat? contentFormat,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) async {
    final filtered = authorPosts
        .where((post) => category == null || post.postCategory == category)
        .where((post) =>
            contentFormat == null || post.contentFormat == contentFormat)
        .toList();
    return (posts: filtered, nextCursor: null);
  }

  @override
  Future<void> followUser({
    required SquareSession session,
    required String followedAccount,
  }) async {
    followCalls++;
    if (throwOnFollow) {
      throw const SquareApiException('follow failed');
    }
  }

  @override
  Future<void> unfollowUser({
    required SquareSession session,
    required String followedAccount,
  }) async {
    unfollowCalls++;
    if (throwOnFollow) {
      throw const SquareApiException('unfollow failed');
    }
  }

  @override
  Future<({List<SquareFollowEntry> accounts, int? nextCursor})> fetchFollows(
    String ownerAccount, {
    required String type,
    int limit = 20,
    int? cursor,
    SquareSession? session,
  }) async {
    return (accounts: follows, nextCursor: null);
  }

  @override
  Future<CitizenProfile> updateProfile({
    required SquareSession session,
    String? displayName,
    String? bio,
    String? avatarObjectKey,
    String? avatarContentHash,
    String? bannerObjectKey,
    String? bannerContentHash,
  }) async {
    lastUpdate = {'display_name': displayName, 'bio': bio};
    return result.copyWith(displayName: displayName, bio: bio);
  }
}

class FakeProfileCache extends CitizenProfileCache {
  FakeProfileCache([this.seed]) : super();

  CitizenProfile? seed;
  bool wrote = false;

  @override
  Future<CitizenProfile?> read(String ownerAccount) async => seed;

  @override
  Future<void> write(CitizenProfile profile) async {
    seed = profile;
    wrote = true;
  }

  @override
  Future<void> clear(String ownerAccount) async {
    seed = null;
  }
}
