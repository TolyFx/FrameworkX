import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'login_types.dart';

/// 登录方式切换：iOS 风格滑动分段控件。
/// 移动端固定展示邮箱与（手机号或密码）两项；桌面端展示全部已配置方式。
class LoginTabs extends StatelessWidget {
  final Set<FxLoginMethod> methods;
  final FxLoginMethod current;
  final ValueChanged<FxLoginMethod> onChanged;
  final bool isDesktop;

  const LoginTabs({
    super.key,
    required this.methods,
    required this.current,
    required this.onChanged,
    required this.isDesktop,
  });

  @override
  Widget build(BuildContext context) {
    final visible = isDesktop
        ? methods.toList()
        : methods.contains(FxLoginMethod.phoneCode)
        ? [FxLoginMethod.emailCode, FxLoginMethod.phoneCode]
        : [FxLoginMethod.emailCode, FxLoginMethod.password];
    final selected = visible.contains(current) ? current : visible.first;
    return CupertinoSlidingSegmentedControl<FxLoginMethod>(
      groupValue: selected,
      padding: const EdgeInsets.all(4),
      onValueChanged: (value) {
        if (value != null) onChanged(value);
      },
      children: {
        for (final method in visible)
          method: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  _methodIcon(method),
                  size: 16,
                  color: const Color(0xFF555555),
                ),
                const SizedBox(width: 6),
                Text(
                  _methodName(method, isDesktop: isDesktop),
                  style: const TextStyle(fontSize: 14),
                ),
              ],
            ),
          ),
      },
    );
  }

  IconData _methodIcon(FxLoginMethod method) => switch (method) {
    FxLoginMethod.emailCode => Icons.email_outlined,
    FxLoginMethod.phoneCode => Icons.phone_android,
    FxLoginMethod.password => Icons.lock_outline,
    FxLoginMethod.scan => Icons.qr_code,
  };

  String _methodName(FxLoginMethod method, {required bool isDesktop}) =>
      switch (method) {
        FxLoginMethod.emailCode => '邮箱登录',
        FxLoginMethod.phoneCode => isDesktop ? '手机号' : '手机号登录',
        FxLoginMethod.password => '密码登录',
        FxLoginMethod.scan => '扫码登录',
      };
}
