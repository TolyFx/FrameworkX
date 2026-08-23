import '../domain/fx_user_session.dart';

/// 用户会话的只读来源。
///
/// 非 UI 模块可依赖该接口，而无需直接依赖 Cubit。
abstract interface class FxUserSessionSource {
  FxUserSession get session;
  Stream<FxUserSession> get sessions;
}
