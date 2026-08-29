import 'package:fx_dio/fx_dio.dart';
import 'package:fx_user_core/fx_user_core.dart';

/// 将 FrameworkX 用户凭据注入统一请求头。
final class FxBearerApiAuth extends ApiAuth {
  /// 当前用户凭据。
  UserCredential? credential;

  @override
  Map<String, dynamic> get buildHeaders {
    final UserCredential? current = credential;
    if (current is BearerCredential) {
      return <String, dynamic>{
        'Authorization': 'Bearer ${current.accessToken}',
      };
    }
    return const <String, dynamic>{};
  }
}
