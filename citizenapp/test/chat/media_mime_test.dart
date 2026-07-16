import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/media/media_mime.dart';

void main() {
  test('mimeFromFileName 按扩展名(大小写不敏感)', () {
    expect(mimeFromFileName('A.JPG'), 'image/jpeg');
    expect(mimeFromFileName('a.png'), 'image/png');
    expect(mimeFromFileName('a.heic'), 'image/heic');
    expect(mimeFromFileName('a.mp4'), 'video/mp4');
    expect(mimeFromFileName('a.mov'), 'video/quicktime');
    expect(mimeFromFileName('a.mkv'), 'video/x-matroska');
    expect(mimeFromFileName('a.pdf'), 'application/pdf');
    expect(mimeFromFileName('a.unknownext'), 'application/octet-stream');
  });

  test('mediaKindFromMime 按前缀', () {
    expect(mediaKindFromMime('image/png'), ChatMessageKind.image);
    expect(mediaKindFromMime('video/mp4'), ChatMessageKind.video);
    expect(mediaKindFromMime('application/pdf'), ChatMessageKind.file);
    expect(mediaKindFromMime('text/plain'), ChatMessageKind.file);
  });
}
