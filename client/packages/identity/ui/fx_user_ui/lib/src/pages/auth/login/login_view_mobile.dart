import 'package:flutter/material.dart';
import '../../../components/auth/login_tabs.dart';
import '../../../components/auth/login_types.dart';
import '../../../components/auth/other_login.dart';
import '../../../components/shared/brand_header.dart';

class LoginViewMobile extends StatelessWidget {
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
    return SafeArea(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 46),
        child: Column(
          children: [
            const SizedBox(height: 80),
            BrandHeader(config: config),
            const SizedBox(height: 60),
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
      ),
    );
  }
}
