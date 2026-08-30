import 'package:flutter_test/flutter_test.dart';
import 'package:fx_user_http/src/avatar_upload_response_decoder.dart';

void main() {
  test('解析完整上传响应信封', _decodesResponseEnvelope);
  test('解析宿主拦截器已解包的上传响应', _decodesUnwrappedResponse);
  test('拒绝缺少图片地址的响应', _rejectsResponseWithoutUrl);
}

void _decodesResponseEnvelope() {
  final Map<String, dynamic> result = decodeAvatarUploadResponse({
    'code': 0,
    'data': {'url': '/uploads/original/avatar.png', 'file_id': 'avatar-file'},
  });

  expect(result['url'], '/uploads/original/avatar.png');
  expect(result['file_id'], 'avatar-file');
}

void _decodesUnwrappedResponse() {
  final Map<String, dynamic> result = decodeAvatarUploadResponse({
    'url': '/uploads/original/avatar.png',
    'file_id': 'avatar-file',
  });

  expect(result['url'], '/uploads/original/avatar.png');
  expect(result['file_id'], 'avatar-file');
}

void _rejectsResponseWithoutUrl() {
  expect(
    () => decodeAvatarUploadResponse({'file_id': 'avatar-file'}),
    throwsA(isA<FormatException>()),
  );
}
