// 设备子钥绑定（op_tag OP_SIGN_SQUARE_DEVICE_BIND = 0x1C）跨语言金标。
//
// 设备绑定是唯一「客户端 + Worker 双侧各自 SCALE 编码」的流，字段编码必须逐字节
// 一致。该 golden hex 必须与 Worker 端
// cloudflare/test/device_subkey.test.ts 的 DEVICE_BIND_GOLDEN_HEX 完全相同。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/wallet/core/device_subkey.dart';

const _owner = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
final String _pubkey = '04${'ab' * 64}';
const int _issuedAt = 1700000000000;
const _goldenHex = 'e9e25da7159f23e174b3c1cfc214ab41c4ea6fa413844e0e89656e8d24166c31';

void main() {
  test('buildDeviceBindingSigningMessage matches Worker golden (0x1C)', () {
    final message = buildDeviceBindingSigningMessage(_owner, _pubkey, _issuedAt);
    expect(message.length, 32);
    expect(bytesToHex(message), _goldenHex);
  });
}
