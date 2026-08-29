/// 验证码的业务使用场景。
enum FxVerificationCodeScene {
  /// 登录。
  login('login'),

  /// 找回密码。
  resetPassword('reset_password'),

  /// 绑定或更换邮箱。
  bindEmail('bind_email'),

  /// 绑定或更换手机号。
  bindPhone('bind_phone');

  /// 传给服务端的协议值。
  final String value;

  const FxVerificationCodeScene(this.value);
}

/// 当前用户视角下的账号标识检查结果。
final class FxAccountCheckResult {
  /// 标识是否已存在。
  final bool exists;

  /// 标识是否属于当前账号。
  final bool ownedByCurrentAccount;

  /// 标识是否可供当前账号绑定。
  final bool available;

  const FxAccountCheckResult({
    required this.exists,
    required this.ownedByCurrentAccount,
    required this.available,
  });

  factory FxAccountCheckResult.fromJson(Map<String, dynamic> json) {
    return FxAccountCheckResult(
      exists: json['exists'] == true,
      ownedByCurrentAccount: json['owned_by_current_account'] == true,
      available: json['available'] == true,
    );
  }
}
