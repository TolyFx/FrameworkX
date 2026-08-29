import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:dio/dio.dart';
import 'package:fx_dio/fx_dio.dart';
import 'package:fx_user_core/fx_user_core.dart';
import 'package:fx_user_session/fx_user_session.dart';
import 'package:uuid/uuid.dart';

/// 通过 FrameworkX 统一存储接口上传用户头像。
final class HttpFxAvatarUploadTask implements FxAvatarUploadTask {
  /// 宿主统一请求入口。
  final RequestHost host;

  /// 头像上传接口路径。
  final String endpoint;

  const HttpFxAvatarUploadTask({
    required this.host,
    this.endpoint = '/storage/upload/image',
  });

  @override
  Future<Uri> upload({
    required Uint8List bytes,
    required UserCredential credential,
  }) async {
    if (credential is! BearerCredential) {
      throw StateError('Avatar upload requires bearer authentication.');
    }
    final FormData form = FormData.fromMap(<String, dynamic>{
      'file': MultipartFile.fromBytes(bytes, filename: 'avatar.png'),
      'hash': sha256.convert(bytes).toString(),
      'upload_id': const Uuid().v4(),
    });
    final ApiRet<Map<String, dynamic>> result = await host
        .post<Map<String, dynamic>>(
          endpoint,
          data: form,
          options: Options(
            headers: <String, dynamic>{
              'Authorization': 'Bearer ${credential.accessToken}',
            },
          ),
          convertor: _decodeUploadResult,
        );
    if (!result.success) throw StateError('Avatar upload request failed.');
    final String? rawUrl = result.data['url']?.toString();
    if (rawUrl == null || rawUrl.isEmpty) {
      throw StateError('Avatar upload returned an empty URL.');
    }
    final Uri? uri = Uri.tryParse(rawUrl);
    if (uri == null) throw StateError('Avatar upload returned an invalid URL.');
    return uri.hasScheme || uri.hasAuthority
        ? uri
        : Uri.parse(host.url).resolveUri(uri);
  }
}

Map<String, dynamic> _decodeUploadResult(dynamic raw) {
  final Map<String, dynamic> envelope = raw as Map<String, dynamic>;
  final dynamic data = envelope['data'];
  if (data is! Map) throw const FormatException('Invalid upload response.');
  return Map<String, dynamic>.from(data);
}
