import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

const String kOwner =
    '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

SquareSession fakeSession() => SquareSession(
      sessionToken: 'tok',
      accountId: kOwner,
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
      accountId: kOwner,
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
  bool notifying = false,
  String displayName = '轻节点',
  String bio = '链上公民',
  String accountId = kOwner,
  String? avatarKey,
  String? bannerKey,
  String? identityLevel,
  String? membershipLevel,
  bool? membershipActive,
}) {
  return CitizenProfile(
    accountId: accountId,
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
    isNotifying: notifying,
    updatedAt: 1,
  );
}

class FakeProfileApi extends CitizenProfileApi {
  FakeProfileApi(
    this.result, {
    this.authorPosts = const [],
    this.follows = const [],
    this.throwOnFollow = false,
    this.throwOnProfile = false,
  }) : super();

  final CitizenProfile result;
  final List<SquarePost> authorPosts;
  final List<SquareFollowEntry> follows;
  final bool throwOnFollow;
  final bool throwOnProfile;
  int calls = 0;
  int followCalls = 0;
  int unfollowCalls = 0;
  int notifyCalls = 0;
  bool? lastNotifyEnabled;
  Map<String, String?>? lastUpdate;

  @override
  Future<CitizenProfile> fetchProfile(
    String accountId, {
    SquareSession? session,
  }) async {
    calls++;
    if (throwOnProfile) {
      throw const SquareApiException('profile failed');
    }
    return result;
  }

  @override
  Future<({List<SquarePost> posts, int? nextCursor})> fetchAuthorPosts(
    String accountId, {
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
    required String followedAccountId,
  }) async {
    followCalls++;
    if (throwOnFollow) {
      throw const SquareApiException('follow failed');
    }
  }

  @override
  Future<void> unfollowUser({
    required SquareSession session,
    required String followedAccountId,
  }) async {
    unfollowCalls++;
    if (throwOnFollow) {
      throw const SquareApiException('unfollow failed');
    }
  }

  @override
  Future<void> setNotify({
    required SquareSession session,
    required String followedAccountId,
    required bool enabled,
  }) async {
    notifyCalls++;
    lastNotifyEnabled = enabled;
    if (throwOnFollow) {
      throw const SquareApiException('notify failed');
    }
  }

  @override
  Future<({List<SquareFollowEntry> accounts, int? nextCursor})> fetchFollows(
    String accountId, {
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
  Future<CitizenProfile?> read(String accountId) async => seed;

  @override
  Future<void> write(CitizenProfile profile) async {
    seed = profile;
    wrote = true;
  }

  @override
  Future<void> clear(String accountId) async {
    seed = null;
  }
}
