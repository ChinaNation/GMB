import 'dart:typed_data';

import 'package:crypto/crypto.dart';

import 'package:citizenapp/8964/services/square_api_client.dart';

class ProfileAssetResult {
  const ProfileAssetResult({
    required this.objectKey,
    required this.contentHash,
  });

  final String objectKey;
  final String contentHash;
}

/// 头像/背景资产上传编排：算 sha256 → 申请授权 → PUT 字节 → 返回 object_key + hash。
/// 内容只进 R2，不上链。
class ProfileAssetService {
  ProfileAssetService({SquareApiClient? client})
      : _client = client ?? SquareApiClient();

  final SquareApiClient _client;

  Future<ProfileAssetResult> upload({
    required SquareSession session,
    required String kind,
    required Uint8List bytes,
    required String contentType,
  }) async {
    final maxBytes = kind == 'avatar' ? 512 * 1024 : 1536 * 1024;
    if (bytes.isEmpty || bytes.length > maxBytes) {
      throw SquareApiException(
        kind == 'avatar' ? '头像不能超过 512KB' : '背景图不能超过 1.5MB',
      );
    }
    final sha = sha256.convert(bytes).toString();
    final prepared = await _client.prepareProfileAsset(
      session: session,
      kind: kind,
      contentType: contentType,
      byteSize: bytes.length,
      sha256Hex: sha,
    );
    await _client.uploadBytesTo(
      prepared.uploadUrl,
      bytes,
      contentType,
      session: session,
    );
    return ProfileAssetResult(
      objectKey: prepared.objectKey,
      contentHash: prepared.contentHash,
    );
  }
}
