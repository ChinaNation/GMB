import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/sfid_api_config.dart';

void main() {
  test('SFID API config only allows production and USB development paths', () {
    expect(
      SfidApiConfig.baseUrlForEnvironment('prod'),
      'https://sfid.crcfrcn.com',
    );
    expect(
      SfidApiConfig.baseUrlForEnvironment('dev_usb'),
      'http://127.0.0.1:8899',
    );

    expect(
      () => SfidApiConfig.baseUrlForEnvironment('http://127.0.0.1:8787'),
      throwsUnsupportedError,
    );
    expect(
      () => SfidApiConfig.baseUrlForEnvironment('http://10.0.0.2:8899'),
      throwsUnsupportedError,
    );
    expect(
      () => SfidApiConfig.baseUrlForEnvironment(''),
      throwsUnsupportedError,
    );
  });
}
