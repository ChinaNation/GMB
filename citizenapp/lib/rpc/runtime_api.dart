// 通用运行时 API(state_call)封装(ADR-028 P3)——非立法专属,任何 runtime API 可复用。
//
// 中文注释:轻节点经 JSON-RPC `state_call('<Trait>_<method>', argsHex, blockHash)` 调运行时
// API。统一**钉 finalized 块哈希**(复用 ChainRpc.fetchFinalizedBlock,ADR-018 读一致),
// 返回结果 SCALE 字节交各业务 codec 镜像解码。

import 'dart:typed_data';

import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';

class RuntimeApi {
  RuntimeApi({ChainRpc? chainRpc}) : _chainRpc = chainRpc ?? ChainRpc();

  final ChainRpc _chainRpc;

  /// 调运行时 API,钉 finalized 块。返回结果 SCALE 字节(null=空/未读到)。
  Future<Uint8List?> call(String apiMethod, Uint8List args) async {
    final finalized = await _chainRpc.fetchFinalizedBlock();
    final blockHashHex = '0x${_hex(finalized.blockHash)}';
    final argsHex = '0x${_hex(args)}';
    final resultHex = await SmoldotClientManager.instance
        .request('state_call', [apiMethod, argsHex, blockHashHex]) as String?;
    if (resultHex == null) return null;
    final clean =
        resultHex.startsWith('0x') ? resultHex.substring(2) : resultHex;
    if (clean.isEmpty) return null;
    return _decodeHex(clean);
  }

  /// 当前 finalized 块号(块号→日期换算用)。
  Future<int> finalizedBlockNumber() async =>
      (await _chainRpc.fetchFinalizedBlock()).blockNumber;

  String _hex(Uint8List b) =>
      b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

  Uint8List _decodeHex(String h) => Uint8List.fromList([
        for (var i = 0; i + 1 < h.length; i += 2)
          int.parse(h.substring(i, i + 2), radix: 16),
      ]);
}
