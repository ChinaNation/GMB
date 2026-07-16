// OnchainIssuance 资产查询模型与严格 SCALE 解码。
//
// 链端唯一布局：
// - OnchainAssetMeta = actor_cid_number + execution_account + class + decimals + state
// - AssetIssued = asset_id + actor_cid_number + execution_account
//
// actor_cid_number 是发行机构身份唯一真源；execution_account 只承担该资产的
// 链上执行，不得作为机构身份、管理员或权限查询 key。

import 'dart:convert';
import 'dart:typed_data';

enum OnchainAssetClass {
  plain,
  pegged,
}

enum OnchainAssetStateKind {
  active,
  closed,
  forceClosed,
}

class OnchainAssetState {
  const OnchainAssetState._({required this.kind, this.closeBlock});

  const OnchainAssetState.active() : this._(kind: OnchainAssetStateKind.active);

  const OnchainAssetState.closed() : this._(kind: OnchainAssetStateKind.closed);

  const OnchainAssetState.forceClosed({required int closeBlock})
      : this._(
          kind: OnchainAssetStateKind.forceClosed,
          closeBlock: closeBlock,
        );

  final OnchainAssetStateKind kind;

  /// 仅 ForceClosed 携带，对齐 runtime `ForceClosed { close_block: u32 }`。
  final int? closeBlock;
}

class OnchainAssetMeta {
  OnchainAssetMeta({
    required this.assetId,
    required this.actorCidNumber,
    required Uint8List executionAccount,
    required this.assetClass,
    required this.decimals,
    required this.state,
  }) : executionAccount = Uint8List.fromList(executionAccount);

  /// storage key `Assets[asset_id]`，不是 OnchainAssetMeta value 的内嵌字段。
  final int assetId;

  /// 发行机构 CID；机构身份和管理员权限只按此字段寻址。
  final String actorCidNumber;

  /// 资产执行账户 AccountId32；只承担资产执行，不表示机构身份。
  final Uint8List executionAccount;

  final OnchainAssetClass assetClass;
  final int decimals;
  final OnchainAssetState state;
}

class OnchainAssetIssued {
  OnchainAssetIssued({
    required this.assetId,
    required this.actorCidNumber,
    required Uint8List executionAccount,
  }) : executionAccount = Uint8List.fromList(executionAccount);

  final int assetId;
  final String actorCidNumber;
  final Uint8List executionAccount;
}

/// runtime storage/event 的局部 SCALE 镜像。
///
/// 解码必须完整消费输入；未知枚举、超长 CID、畸形 UTF-8 或尾随字节一律拒绝，
/// 禁止把任何非当前布局误读为当前协议。
class OnchainAssetScaleCodec {
  OnchainAssetScaleCodec._();

  static const int _accountIdLength = 32;
  static const int _maxCidNumberLength = 32;

  static OnchainAssetMeta decodeMeta({
    required int assetId,
    required Uint8List scale,
  }) {
    _ensureAssetId(assetId);
    final reader = _ScaleReader(scale);
    final actorCidNumber = reader.readCidNumber();
    final executionAccount = reader.readBytes(_accountIdLength);
    final assetClass = switch (reader.readU8()) {
      0 => OnchainAssetClass.plain,
      1 => OnchainAssetClass.pegged,
      final value => throw FormatException('未知 AssetClass 枚举: $value'),
    };
    final decimals = reader.readU8();
    if (decimals > 18) {
      throw FormatException('OnchainAssetMeta.decimals 超出链端范围: $decimals');
    }
    final state = _decodeState(reader);
    reader.ensureFinished('OnchainAssetMeta');
    return OnchainAssetMeta(
      assetId: assetId,
      actorCidNumber: actorCidNumber,
      executionAccount: executionAccount,
      assetClass: assetClass,
      decimals: decimals,
      state: state,
    );
  }

  static OnchainAssetIssued decodeAssetIssued(Uint8List scale) {
    final reader = _ScaleReader(scale);
    final assetId = reader.readU32();
    final actorCidNumber = reader.readCidNumber();
    final executionAccount = reader.readBytes(_accountIdLength);
    reader.ensureFinished('AssetIssued');
    return OnchainAssetIssued(
      assetId: assetId,
      actorCidNumber: actorCidNumber,
      executionAccount: executionAccount,
    );
  }

  static OnchainAssetState _decodeState(_ScaleReader reader) {
    return switch (reader.readU8()) {
      0 => const OnchainAssetState.active(),
      1 => const OnchainAssetState.closed(),
      2 => OnchainAssetState.forceClosed(closeBlock: reader.readU32()),
      final value => throw FormatException('未知 AssetState 枚举: $value'),
    };
  }

  static void _ensureAssetId(int assetId) {
    if (assetId < 0 || assetId > 0xFFFFFFFF) {
      throw ArgumentError.value(assetId, 'assetId', '必须在 u32 范围内');
    }
  }
}

class _ScaleReader {
  _ScaleReader(this._bytes);

  final Uint8List _bytes;
  int _offset = 0;

  int readU8() {
    _ensureAvailable(1);
    return _bytes[_offset++];
  }

  int readU32() {
    _ensureAvailable(4);
    final value = ByteData.sublistView(_bytes, _offset, _offset + 4)
        .getUint32(0, Endian.little);
    _offset += 4;
    return value;
  }

  Uint8List readBytes(int length) {
    if (length < 0) {
      throw const FormatException('SCALE 字节长度不能为负数');
    }
    _ensureAvailable(length);
    final value = Uint8List.fromList(_bytes.sublist(_offset, _offset + length));
    _offset += length;
    return value;
  }

  String readCidNumber() {
    final length = readCompactU32();
    if (length == 0 || length > OnchainAssetScaleCodec._maxCidNumberLength) {
      throw FormatException('actor_cid_number 长度非法: $length');
    }
    final value = readBytes(length);
    try {
      return utf8.decode(value);
    } on FormatException {
      throw const FormatException('actor_cid_number 不是合法 UTF-8');
    }
  }

  int readCompactU32() {
    final first = readU8();
    switch (first & 0x03) {
      case 0:
        return first >> 2;
      case 1:
        _ensureAvailable(1);
        final second = _bytes[_offset++];
        return ((first | (second << 8)) >> 2);
      case 2:
        _ensureAvailable(3);
        final encoded = first |
            (_bytes[_offset] << 8) |
            (_bytes[_offset + 1] << 16) |
            (_bytes[_offset + 2] << 24);
        _offset += 3;
        return encoded >>> 2;
      case 3:
        throw const FormatException('SCALE Compact 大整数模式不适用于 u32 长度');
    }
    throw const FormatException('无法解析 SCALE Compact<u32>');
  }

  void ensureFinished(String typeName) {
    if (_offset != _bytes.length) {
      throw FormatException(
        '$typeName SCALE 存在尾随字节: ${_bytes.length - _offset}',
      );
    }
  }

  void _ensureAvailable(int length) {
    if (_offset + length > _bytes.length) {
      throw const FormatException('SCALE 数据被截断');
    }
  }
}

abstract class OnchainAssetQuery {
  /// 列出指定持币人(SS58)持有的所有资产与当前余额关联元数据。
  Future<List<OnchainAssetMeta>> listAssetsHeldBy(String ss58);

  /// 按 asset_id 读取并严格解码 OnchainIssuance.Assets 元数据。
  Future<OnchainAssetMeta?> readAssetById(int assetId);

  /// 读取特定持币人在某资产下的 raw 余额。
  Future<BigInt> readBalance({required int assetId, required String ss58});
}
