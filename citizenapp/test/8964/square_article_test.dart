import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/pages/square_article_compose_page.dart';

String? _validate({
  String title = '标题标题标题标题标题',
  bool cover = true,
  String body = '正文内容',
}) {
  return articleValidationError(title: title, hasCover: cover, body: body);
}

void main() {
  group('articleValidationError', () {
    test('passes with valid title, cover and body', () {
      expect(_validate(), isNull);
    });

    test('rejects a title shorter than 10 chars', () {
      expect(_validate(title: '短标题'), '标题需 10–50 字');
    });

    test('rejects a title longer than 50 chars', () {
      expect(_validate(title: 'x' * 51), '标题需 10–50 字');
    });

    test('requires a cover image', () {
      expect(_validate(cover: false), '请选择 1 张首图');
    });

    test('requires non-empty body', () {
      expect(_validate(body: '   '), '正文不能为空');
    });

    test('rejects a body over 19890 chars', () {
      expect(_validate(body: 'x' * 19891), '正文不能超过 19890 字');
    });
  });
}
