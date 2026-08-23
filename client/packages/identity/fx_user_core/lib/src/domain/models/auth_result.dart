import '../entities/user.dart';
import 'credential.dart';

final class AuthResult {
  /// 登录后用于建立应用会话的用户资料。
  ///
  /// 认证接口必须在同一响应中提供该资料，避免客户端再请求一次资料接口。
  final FxUser user;

  /// 本次认证获得的访问凭据。
  final UserCredential credential;

  const AuthResult({required this.credential, required this.user});
}
