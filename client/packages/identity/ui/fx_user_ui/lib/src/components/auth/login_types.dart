import 'package:flutter/widgets.dart';
import 'package:fx_user_core/fx_user_core.dart';

/// 登录方式：邮箱验证码、手机号验证码、账号密码。
enum FxLoginMethod { emailCode, phoneCode, password, scan }

/// 登录提交回调：宿主根据方式、账号标识与凭证完成登录。
typedef FxLoginSubmit =
    Future<void> Function({
      required FxLoginMethod method,
      required String identifier,
      required String credential,
    });

/// 验证码请求回调：宿主发送验证码，可选返回验证码（用于测试自动回填）。
typedef FxVerificationCodeRequest =
    Future<String?> Function({
      required FxLoginMethod method,
      required String identifier,
    });

/// 登录界面可配置项：标题、副标题、Logo、支持的登录方式、协议文案等。
class FxUserUiConfig {
  final String title;
  final String subtitle;
  final Widget? logo;
  final Set<FxLoginMethod> methods;
  final String agreementText;
  final VoidCallback? onUserAgreement;
  final VoidCallback? onPrivacyPolicy;
  final String Function(Object error) errorText;
  final Future<void> Function()? onGithubLogin;
  final Future<void> Function()? onAppleLogin;
  final bool showGithub;
  final bool showApple;
  final Future<FxScanSession> Function()? createScanSession;
  final Future<FxScanStatus> Function(String token)? pollScanStatus;
  final Future<void> Function(String credential)? onScanAuthenticated;

  FxUserUiConfig({
    this.title = 'WELCOME',
    this.subtitle = '',
    this.logo,
    this.methods = const {FxLoginMethod.emailCode, FxLoginMethod.password},
    this.agreementText = '登录即代表您同意《用户协议》和《隐私政策》，未注册绑定的手机号验证成功后将自动注册',
    this.onUserAgreement,
    this.onPrivacyPolicy,
    this.onGithubLogin,
    this.onAppleLogin,
    this.showGithub = true,
    this.showApple = true,
    this.createScanSession,
    this.pollScanStatus,
    this.onScanAuthenticated,
    String Function(Object error)? errorText,
  }) : assert(methods.isNotEmpty),
       errorText = errorText ?? ((error) => error.toString());
}
