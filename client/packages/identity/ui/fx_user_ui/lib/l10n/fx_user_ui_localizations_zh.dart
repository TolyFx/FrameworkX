// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'fx_user_ui_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class FxUserUiLocalizationsZh extends FxUserUiLocalizations {
  FxUserUiLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get close => '关闭';

  @override
  String get scanUnavailable => '当前宿主未配置扫码登录';

  @override
  String get acceptAgreements => '请先阅读并同意用户协议和隐私政策';

  @override
  String get loginMethodUnavailable => '当前宿主未配置该登录方式';

  @override
  String get emailLogin => '邮箱登录';

  @override
  String get phoneLogin => '手机号登录';

  @override
  String get passwordLogin => '密码登录';

  @override
  String get scanLogin => '扫码登录';

  @override
  String get account => '账号';

  @override
  String get email => '邮箱';

  @override
  String get phone => '手机号';

  @override
  String get verificationCode => '验证码';

  @override
  String get password => '密码';

  @override
  String get emailHint => '请输入邮箱';

  @override
  String get phoneHint => '请输入手机号';

  @override
  String get accountHint => '用户 ID/手机号/邮箱';

  @override
  String get codeHint => '请输入验证码';

  @override
  String get passwordHint => '请输入密码';

  @override
  String get getCode => '获取验证码';

  @override
  String get login => '登录';

  @override
  String get agreementPrefix => '登录即代表您同意';

  @override
  String get userAgreement => '《用户协议》';

  @override
  String get agreementAnd => '和';

  @override
  String get privacyPolicy => '《隐私政策》';

  @override
  String get agreementSuffix => '，未注册账号验证成功后将自动注册';

  @override
  String get otherLoginMethods => '其他登录方式';

  @override
  String get codeLogin => '验证码登录';

  @override
  String get reloadQrCode => '重新加载二维码';

  @override
  String get qrCodeExpired => '二维码已过期，点击刷新';

  @override
  String get scanConfirmed => '已扫码，请在手机上确认';

  @override
  String get scanQrCodeHint => '使用已登录的移动端扫描二维码';
}
