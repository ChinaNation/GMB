import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_post_store.dart';

/// 帖子删除的统一生命周期边界，供详情页和“编辑后替换旧帖”共同使用。
abstract class SquarePostDeleteCoordinator {
  Future<void> delete({
    required SquareSession session,
    required String cidNumber,
    required String postId,
  });
}

/// 先确认 Worker 已删除（或明确已不存在），再删除同 CID 的本地副本。
///
/// 归属唯一以 CID 判断，不能用可换绑的 account_id。只有 Worker 精确返回
/// `404/post_not_found` 才视为远端已不存在；权限、会话、网络等错误一律保留本地副本。
class SquarePostDeletionCoordinator implements SquarePostDeleteCoordinator {
  SquarePostDeletionCoordinator({
    SquarePostDeletionService? remoteDeletion,
    SquareLocalPostDeletionStore? localStore,
  })  : _remoteDeletion = remoteDeletion ?? SquareApiClient(),
        _localStore = localStore ?? const SquarePostStore();

  final SquarePostDeletionService _remoteDeletion;
  final SquareLocalPostDeletionStore _localStore;

  @override
  Future<void> delete({
    required SquareSession session,
    required String cidNumber,
    required String postId,
  }) async {
    if (cidNumber != session.cidNumber) {
      throw const SquareApiException(
        '只能删除本人内容',
        statusCode: 403,
        errorCode: 'post_owner_mismatch',
      );
    }

    try {
      await _remoteDeletion.deletePost(
        session: session,
        postId: postId,
      );
    } on SquareApiException catch (error) {
      final alreadyAbsent =
          error.statusCode == 404 && error.errorCode == 'post_not_found';
      if (!alreadyAbsent) rethrow;
    }

    await _localStore.delete(
      cidNumber: cidNumber,
      postId: postId,
    );
  }
}
