import 'fx_identity.dart';

/// 面向应用模块的当前用户会话。
///
/// 会话仅是运行时状态；认证凭据和请求异常均不会在这里表达或持久化。
sealed class FxUserSession {
  const FxUserSession();
}

/// 应用正在恢复或建立认证会话。
final class FxAuthing extends FxUserSession {
  const FxAuthing();
}

/// 当前运行时没有已认证用户。
final class FxGuest extends FxUserSession {
  const FxGuest();
}

/// 当前运行时存在已认证用户。
final class FxAuthed extends FxUserSession {
  final FxIdentity user;

  const FxAuthed(this.user);
}
