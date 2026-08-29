import 'package:fx_user_core/fx_user_core.dart';

/// 将旧服务主机上的本地上传头像迁移到当前服务根地址。
FxUser normalizeFxUserUploadUrls(FxUser user, Uri serverUri) {
  final Uri? avatar = normalizeFxUploadUri(user.avatar, serverUri);
  return avatar == user.avatar ? user : user.copyWith(avatar: avatar);
}

/// 仅重写 IPv4 主机下的 `/uploads/` 资源，不影响第三方头像。
Uri? normalizeFxUploadUri(Uri? source, Uri serverUri) {
  if (source == null ||
      source.scheme != 'http' ||
      !source.path.startsWith('/uploads/') ||
      !_isIpv4(source.host)) {
    return source;
  }
  return serverUri.resolveUri(
    Uri(
      path: source.path,
      query: source.hasQuery ? source.query : null,
      fragment: source.hasFragment ? source.fragment : null,
    ),
  );
}

bool _isIpv4(String host) {
  final List<String> parts = host.split('.');
  if (parts.length != 4) return false;
  return parts.every((String part) {
    final int? value = int.tryParse(part);
    return value != null && value >= 0 && value <= 255;
  });
}
