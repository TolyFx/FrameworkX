import 'package:flutter/material.dart';

import '../shared/labeled_input.dart';
import '../shared/style.dart';
import 'login_types.dart';

/// 账号标识 + 凭证（验证码或密码）两行输入。
/// 验证码模式在第二行尾部提供「获取验证码」按钮，含发送中与冷却态。
class LoginFormFields extends StatelessWidget {
  final TextEditingController identifierController;
  final TextEditingController credentialController;
  final FxLoginMethod method;
  final bool sendingCode;
  final int cooldownSeconds;
  final VoidCallback onRequestCode;

  const LoginFormFields({
    super.key,
    required this.identifierController,
    required this.credentialController,
    required this.method,
    required this.sendingCode,
    required this.cooldownSeconds,
    required this.onRequestCode,
  });

  bool get _usesCode => method != FxLoginMethod.password;
  bool get _codeBusy => sendingCode || cooldownSeconds > 0;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        LabeledInput(
          label: switch (method) {
            FxLoginMethod.emailCode => '邮箱',
            FxLoginMethod.phoneCode => '+86',
            FxLoginMethod.password => '账号',
            FxLoginMethod.scan => '账号',
          },
          child: TextField(
            controller: identifierController,
            keyboardType: method == FxLoginMethod.phoneCode
                ? TextInputType.phone
                : TextInputType.emailAddress,
            style: const TextStyle(fontSize: 16),
            decoration: InputDecoration(
              hintText: method == FxLoginMethod.emailCode
                  ? '请输入邮箱'
                  : method == FxLoginMethod.phoneCode
                  ? '请输入手机号'
                  : '用户ID/手机号/邮箱',
              filled: false,
              border: InputBorder.none,
              contentPadding: const EdgeInsets.symmetric(vertical: 14),
            ),
          ),
        ),
        const SizedBox(height: 16),
        LabeledInput(
          label: _usesCode ? '验证码' : '密码',
          trailing: _usesCode
              ? GestureDetector(
                  onTap: _codeBusy ? null : onRequestCode,
                  child: Text(
                    cooldownSeconds > 0 ? '${cooldownSeconds}s' : '获取验证码',
                    style: TextStyle(
                      fontSize: 14,
                      color: _codeBusy ? Colors.grey : fxPrimary,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                )
              : null,
          child: TextField(
            controller: credentialController,
            obscureText: !_usesCode,
            keyboardType: _usesCode
                ? TextInputType.number
                : TextInputType.visiblePassword,
            maxLength: _usesCode ? 6 : null,
            style: const TextStyle(fontSize: 16),
            decoration: InputDecoration(
              hintText: _usesCode ? '请输入验证码' : '请输入密码',
              filled: false,
              border: InputBorder.none,
              counterText: '',
              contentPadding: const EdgeInsets.symmetric(vertical: 14),
            ),
          ),
        ),
      ],
    );
  }
}
