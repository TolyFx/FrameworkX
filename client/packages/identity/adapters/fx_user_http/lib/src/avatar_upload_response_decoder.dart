/// 兼容解析完整响应信封与宿主拦截器已经解包的头像上传数据。
Map<String, dynamic> decodeAvatarUploadResponse(dynamic raw) {
  if (raw is! Map) {
    throw const FormatException('Invalid upload response.');
  }

  final dynamic nestedData = raw['data'];
  final Map<dynamic, dynamic> payload = nestedData is Map ? nestedData : raw;
  final dynamic rawUrl = payload['url'];
  if (rawUrl == null || rawUrl.toString().isEmpty) {
    throw const FormatException('Invalid upload response.');
  }
  return Map<String, dynamic>.from(payload);
}
