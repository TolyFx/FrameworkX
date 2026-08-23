import 'package:flutter/material.dart';
import '../shared/action_button.dart';
import 'agreement_row.dart';
import 'login_form_fields.dart';
import 'login_types.dart';

/// 登录表单主体：字段 + 协议 + 错误 + 提交。
class LoginFormBody extends StatelessWidget {
  final TextEditingController identifierController;
  final TextEditingController credentialController;
  final FxLoginMethod method;
  final bool agreed;
  final bool submitting;
  final bool sendingCode;
  final int cooldownSeconds;
  final Object? error;
  final FxUserUiConfig config;
  final VoidCallback onAgreedChanged;
  final VoidCallback onRequestCode;
  final VoidCallback onSubmit;

  const LoginFormBody({
    super.key,
    required this.identifierController,
    required this.credentialController,
    required this.method,
    required this.agreed,
    required this.submitting,
    required this.sendingCode,
    required this.cooldownSeconds,
    required this.error,
    required this.config,
    required this.onAgreedChanged,
    required this.onRequestCode,
    required this.onSubmit,
  });

  bool get _canSubmit =>
      agreed &&
      !submitting &&
      identifierController.text.trim().isNotEmpty &&
      credentialController.text.trim().isNotEmpty;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        LoginFormFields(
          identifierController: identifierController,
          credentialController: credentialController,
          method: method,
          sendingCode: sendingCode,
          cooldownSeconds: cooldownSeconds,
          onRequestCode: onRequestCode,
        ),
        const SizedBox(height: 36),
        AgreementRow(
          checked: agreed,
          text: config.agreementText,
          onTap: onAgreedChanged,
          onUserAgreement: config.onUserAgreement,
          onPrivacyPolicy: config.onPrivacyPolicy,
        ),
        if (error != null) ...[
          const SizedBox(height: 16),
          Text(
            config.errorText(error!),
            textAlign: TextAlign.left,
            style: const TextStyle(color: Colors.red, fontSize: 13),
          ),
        ],
        const SizedBox(height: 32),
        ActionButton(
          enabled: _canSubmit,
          loading: submitting,
          onPressed: onSubmit,
        ),
      ],
    );
  }
}
