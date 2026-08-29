import 'package:flutter/material.dart';
import '../../../components/auth/login_tabs.dart';
import '../../../components/auth/login_types.dart';
import '../../../components/auth/other_login.dart';
import '../../../components/shared/brand_header.dart';

class LoginViewMobile extends StatelessWidget {
  /// 正常状态完整展示登录内容所需的最小高度。
  static const double _minimumStaticHeight = 760;

  final FxUserUiConfig config;
  final FxLoginMethod method;
  final Widget formBody;
  final bool submitting;
  final ValueChanged<FxLoginMethod> onMethodChanged;
  final VoidCallback onToggleMode;
  final Future<void> Function(FxThirdPartyLogin? login) onThirdPartyLogin;

  const LoginViewMobile({
    super.key,
    required this.config,
    required this.method,
    required this.formBody,
    required this.submitting,
    required this.onMethodChanged,
    required this.onToggleMode,
    required this.onThirdPartyLogin,
  });

  bool get _usesCode => method != FxLoginMethod.password;

  @override
  Widget build(BuildContext context) {
    return SafeArea(child: LayoutBuilder(builder: _buildContent));
  }

  /// 正常高度禁止拖动；仅在键盘弹出或小屏空间不足时开放滚动。
  Widget _buildContent(BuildContext context, BoxConstraints constraints) {
    final bool keyboardVisible = MediaQuery.viewInsetsOf(context).bottom > 0;
    final bool needsScrolling =
        keyboardVisible || constraints.maxHeight < _minimumStaticHeight;
    return SingleChildScrollView(
      physics: needsScrolling
          ? const ClampingScrollPhysics()
          : const NeverScrollableScrollPhysics(),
      padding: const EdgeInsets.symmetric(horizontal: 46),
      child: Column(
        children: <Widget>[
          const SizedBox(height: 48),
          BrandHeader(config: config),
          const SizedBox(height: 52),
          LoginTabs(
            methods: config.methods,
            current: method,
            onChanged: onMethodChanged,
            isDesktop: false,
          ),
          const SizedBox(height: 16),
          formBody,
          const SizedBox(height: 48),
          OtherLoginRow(
            loading: submitting,
            showGithub: config.showGithub,
            showApple: config.showApple,
            showPasswordToggle: config.methods.contains(
              FxLoginMethod.phoneCode,
            ),
            isCodeMode: _usesCode,
            onGithub: () => onThirdPartyLogin(config.onGithubLogin),
            onApple: () => onThirdPartyLogin(config.onAppleLogin),
            onToggleMode: onToggleMode,
          ),
          const SizedBox(height: 40),
        ],
      ),
    );
  }
}
