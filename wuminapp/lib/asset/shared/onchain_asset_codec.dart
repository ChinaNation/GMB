// 链上发行代币 SubjectId(0x04)编解码 + AssetMeta SCALE 解码。
//
// SubjectId(48B)布局(ADR-010 增量段 + ADR-011 v2 简化):
//   byte[0]:    0x04                             (OnchainAsset kind tag)
//   byte[1..5]: 4B asset_id (u32 LE)
//   byte[5..48]: 43B 零填充(预留)
//
// 链端权威实现:citizenchain/runtime/primitives/src/derive.rs::subject_id_from_onchain_asset。
// 反查发行人走链端 OnchainIssuance::Assets[SubjectId].issuer_subject_id(48B 完整保留)。
// 框架阶段先实现编/解码,后续业务接入时补 AssetMeta 解码。

import 'dart:typed_data';

class OnchainAssetCodec {
  OnchainAssetCodec._();

  /// 构造 OnchainAsset SubjectId(48B)。
  ///
  /// [assetId] u32 LE,链端 NextAssetId 自增分配(从 1 开始)。
  /// 返回 48B Uint8List(byte[0]=0x04, byte[1..5]=asset_id LE, 其余零)。
  static Uint8List buildSubjectId(int assetId) {
    if (assetId < 0 || assetId > 0xFFFFFFFF) {
      throw ArgumentError('assetId 必须在 u32 范围内,实际 $assetId');
    }
    final id = Uint8List(48);
    id[0] = 0x04;
    final aidBytes = ByteData(4)..setUint32(0, assetId, Endian.little);
    id.setRange(1, 5, aidBytes.buffer.asUint8List());
    // byte[5..48] 保持零(Uint8List 默认零)
    return id;
  }

  /// 反向解析 OnchainAsset SubjectId,返回 asset_id(u32)。
  ///
  /// [id] 必须是 48 字节且 byte[0] == 0x04,否则抛 FormatException。
  static int parseSubjectId(Uint8List id) {
    if (id.length != 48) {
      throw FormatException('SubjectId 必须是 48 字节,实际 ${id.length}');
    }
    if (id[0] != 0x04) {
      throw FormatException('非 OnchainAsset SubjectId(byte[0]=${id[0]} != 0x04)');
    }
    final aidBytes = ByteData.sublistView(id, 1, 5);
    return aidBytes.getUint32(0, Endian.little);
  }
}
