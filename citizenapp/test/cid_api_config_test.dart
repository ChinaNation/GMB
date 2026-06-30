import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/cid_api_config.dart';

void main() {
  test('OnChina API config only allows production and USB development paths', () {
    expect(
      CidApiConfig.baseUrlForEnvironment('prod'),
      'https://cid.crcfrcn.com',
    );
    expect(
      CidApiConfig.baseUrlForEnvironment('dev_usb'),
      'http://127.0.0.1:8899',
    );

    expect(
      () => CidApiConfig.baseUrlForEnvironment('http://127.0.0.1:8787'),
      throwsUnsupportedError,
    );
    expect(
      () => CidApiConfig.baseUrlForEnvironment('http://10.0.0.2:8899'),
      throwsUnsupportedError,
    );
    expect(
      () => CidApiConfig.baseUrlForEnvironment(''),
      throwsUnsupportedError,
    );
  });
}
