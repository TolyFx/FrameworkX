/// 将头像上传接口返回的相对资源地址拼接到包含 API 前缀的服务地址。
Uri resolveAvatarUploadUri(Uri resource, String serverUrl) {
  if (resource.hasScheme || resource.hasAuthority) {
    return resource;
  }

  final Uri server = Uri.parse(serverUrl);
  final String serverPath = _trimTrailingSlash(server.path);
  final String resourcePath = _trimLeadingSlash(resource.path);
  final String joinedPath = '$serverPath/$resourcePath';
  return server.replace(
    path: joinedPath,
    query: resource.hasQuery ? resource.query : null,
    fragment: resource.hasFragment ? resource.fragment : null,
  );
}

String _trimLeadingSlash(String value) {
  return value.startsWith('/') ? value.substring(1) : value;
}

String _trimTrailingSlash(String value) {
  return value.endsWith('/') ? value.substring(0, value.length - 1) : value;
}
