// 链上发行代币 asset_id 编解码 + AssetMeta SCALE 解码。
//
// asset_id 只标识资产本身，治理身份只来自机构多签 AccountId。
// 发行人/治理主体必须是机构多签 AccountId，AssetMeta 的发行人字段按 AccountId 解码。

import 'dart:typed_data';

class OnchainAssetCodec {
  OnchainAssetCodec._();

  /// 将 asset_id 编码为 u32 little-endian 字节。
  static Uint8List encodeAssetId(int assetId) {
    if (assetId < 0 || assetId > 0xFFFFFFFF) {
      throw ArgumentError('assetId 必须在 u32 范围内,实际 $assetId');
    }
    final bytes = ByteData(4)..setUint32(0, assetId, Endian.little);
    return bytes.buffer.asUint8List();
  }

  /// 从 u32 little-endian 字节解析 asset_id。
  static int decodeAssetId(Uint8List bytes) {
    if (bytes.length != 4) {
      throw FormatException('asset_id 必须是 4 字节,实际 ${bytes.length}');
    }
    return ByteData.sublistView(bytes).getUint32(0, Endian.little);
  }
}
