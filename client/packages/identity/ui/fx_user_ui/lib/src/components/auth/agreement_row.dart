import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../shared/style.dart';

/// 协议勾选行：点击整行切换勾选态。
class AgreementRow extends StatelessWidget {
  final bool checked;
  final String text;
  final VoidCallback onTap;
  final VoidCallback? onUserAgreement;
  final VoidCallback? onPrivacyPolicy;

  const AgreementRow({
    super.key,
    required this.checked,
    required this.text,
    required this.onTap,
    this.onUserAgreement,
    this.onPrivacyPolicy,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 14,
            height: 14,
            margin: const EdgeInsets.only(top: 2),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(3),
              color: checked ? fxPrimary : Colors.transparent,
              border: Border.all(
                color: checked ? fxPrimary : Colors.grey[400]!,
                width: 1.2,
              ),
            ),
            child: checked
                ? const Icon(Icons.check, size: 10, color: Colors.white)
                : null,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text.rich(
              TextSpan(
                style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                children: [
                  const TextSpan(text: '登录即代表您同意'),
                  _link('《用户协议》', onUserAgreement),
                  const TextSpan(text: '和'),
                  _link('《隐私政策》', onPrivacyPolicy),
                  const TextSpan(text: '，未注册绑定的手机号验证成功后将自动注册'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  TextSpan _link(String label, VoidCallback? onTap) => TextSpan(
    text: label,
    style: const TextStyle(color: fxPrimary),
    recognizer: onTap == null ? null : (TapGestureRecognizer()..onTap = onTap),
  );
}
