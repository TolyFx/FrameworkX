import 'package:flutter/material.dart';
import '../../../components/auth/login_tabs.dart';
import '../../../components/auth/login_types.dart';
import '../../../components/shared/brand_header.dart';

class LoginViewDesktop extends StatelessWidget {
  final FxUserUiConfig config;
  final FxLoginMethod method;
  final Widget formBody;
  final ValueChanged<FxLoginMethod> onMethodChanged;

  const LoginViewDesktop({
    super.key,
    required this.config,
    required this.method,
    required this.formBody,
    required this.onMethodChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        SizedBox(
          width: 280,
          child: ColoredBox(
            color: const Color(0xFFDBEAFE),
            child: Center(child: BrandHeader(config: config, desktop: true)),
          ),
        ),
        Expanded(
          child: Center(
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 48, vertical: 40),
              child: Container(
                width: 460,
                padding: const EdgeInsets.symmetric(
                  horizontal: 60,
                  vertical: 52,
                ),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(16),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: 0.06),
                      blurRadius: 24,
                      offset: const Offset(0, 4),
                    ),
                  ],
                ),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
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
        ),
      ],
    );
  }
}
