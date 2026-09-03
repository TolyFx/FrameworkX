import 'package:flutter/material.dart';

import '../../../components/auth/login_tabs.dart';
import '../../../components/auth/login_types.dart';

/// 仅展示登录方式与表单主体的桌面弹框。
class LoginViewDialog extends StatelessWidget {
  const LoginViewDialog({
    super.key,
    required this.config,
    required this.method,
    required this.formBody,
    required this.onMethodChanged,
  });

  /// 宿主提供的登录方式与协议配置。
  final FxUserUiConfig config;

  /// 当前选择的登录方式。
  final FxLoginMethod method;

  /// 包含输入、协议和提交按钮的表单内容。
  final Widget formBody;

  /// 切换登录方式时触发的回调。
  final ValueChanged<FxLoginMethod> onMethodChanged;

  @override
  Widget build(BuildContext context) {
    final ThemeData dialogTheme = ThemeData.light(
      useMaterial3: Theme.of(context).useMaterial3,
    );
    return Theme(
      data: dialogTheme,
      child: Dialog(
        backgroundColor: Colors.transparent,
        elevation: 0,
        insetPadding: const EdgeInsets.all(24),
        child: SingleChildScrollView(
          child: Container(
            width: 460,
            padding: const EdgeInsets.symmetric(horizontal: 60, vertical: 52),
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(16),
              boxShadow: <BoxShadow>[
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.08),
                  blurRadius: 24,
                  offset: const Offset(0, 4),
                ),
              ],
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                LoginTabs(
                  methods: config.methods,
                  current: method,
                  onChanged: onMethodChanged,
                  isDesktop: true,
                ),
                const SizedBox(height: 24),
                formBody,
              ],
            ),
          ),
        ),
      ),
    );
  }
}
