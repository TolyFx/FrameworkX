import 'package:fx_user_core/fx_user_core.dart';

/// 可恢复认证凭据的宿主存储。
///
/// 只保存凭据，不保存用户资料或 [FxUserSession] 运行时状态。
abstract interface class AuthCredentialStore {
  Future<UserCredential?> read();
  Future<void> write(UserCredential credential);
  Future<void> clear();
}
