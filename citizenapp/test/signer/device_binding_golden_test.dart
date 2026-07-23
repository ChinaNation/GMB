// 设备子钥绑定（op_tag OP_SIGN_SQUARE_DEVICE_BIND = 0x1C）跨语言金标。
//
// 设备绑定是唯一「客户端 + Worker 双侧各自 SCALE 编码」的流，字段编码必须逐字节
// 一致。该 golden hex 必须与 Worker 端
// cloudflare/test/device_subkey.test.ts 的 DEVICE_BIND_GOLDEN_HEX 完全相同。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

const _accountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';
final String _publicKey = '04${'ab' * 64}';
const int _issuedAt = 1700000000000;
const _goldenHex =
    '0089e293c8ef5c4d7bb5820e18dcb0bdac4eb374eaf6675c1bc2e53e50c3b960';

void main() {
  test('buildDeviceBindingSigningMessage matches Worker golden (0x1C)', () {
    final message =
        buildDeviceBindingSigningMessage(_accountId, _publicKey, _issuedAt);
    expect(message.length, 32);
    expect(bytesToHex(message), _goldenHex);
  });
}
