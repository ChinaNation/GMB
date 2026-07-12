import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/pages/square_turnstile_page.dart';

void main() {
  test('Turnstile 页面使用调用方指定的同域 API 根地址', () {
    const page = SquareTurnstilePage(
      baseUrl: 'https://www.crcfrcn.com/api-staging',
    );
    expect(page.baseUrl, 'https://www.crcfrcn.com/api-staging');
  });
}
