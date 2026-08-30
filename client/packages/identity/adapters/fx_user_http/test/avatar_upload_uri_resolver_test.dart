import 'package:flutter_test/flutter_test.dart';
import 'package:fx_user_http/src/avatar_upload_uri_resolver.dart';

void main() {
  test('上传资源地址保留服务端 API 前缀', _keepsApiNest);
  test('无前导斜杠的上传资源地址保留 API 前缀', _keepsApiNestWithoutSlash);
  test('完整外部头像地址保持不变', _keepsAbsoluteUri);
}

void _keepsApiNest() {
  final Uri result = resolveAvatarUploadUri(
    Uri.parse('/uploads/original/2026/08/avatar.png'),
    'https://fx.toly1994.com/unit',
  );

  expect(
    result.toString(),
    'https://fx.toly1994.com/unit/uploads/original/2026/08/avatar.png',
  );
}

void _keepsApiNestWithoutSlash() {
  final Uri result = resolveAvatarUploadUri(
    Uri.parse('uploads/original/2026/08/avatar.png'),
    'https://fx.toly1994.com/unit/',
  );

  expect(
    result.toString(),
    'https://fx.toly1994.com/unit/uploads/original/2026/08/avatar.png',
  );
}

void _keepsAbsoluteUri() {
  final Uri source = Uri.parse('https://avatars.githubusercontent.com/u/1');

  expect(
    resolveAvatarUploadUri(source, 'https://fx.toly1994.com/unit'),
    source,
  );
}
