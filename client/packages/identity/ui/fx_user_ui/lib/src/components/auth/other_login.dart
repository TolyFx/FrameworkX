import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

/// 其他登录方式：分隔线标题 + 第三方入口与「验证码/密码登录」切换。
class OtherLoginRow extends StatelessWidget {
  final bool loading;
  final bool showGithub;
  final bool showApple;
  final bool showPasswordToggle;
  final bool isCodeMode;
  final VoidCallback onGithub;
  final VoidCallback onApple;
  final VoidCallback onToggleMode;

  const OtherLoginRow({
    super.key,
    required this.loading,
    required this.showGithub,
    required this.showApple,
    required this.showPasswordToggle,
    required this.isCodeMode,
    required this.onGithub,
    required this.onApple,
    required this.onToggleMode,
  });

  @override
  Widget build(BuildContext context) {
    if (!showGithub && !showApple && !showPasswordToggle) {
      return const SizedBox.shrink();
    }
    return Column(
      children: [
        Row(
          children: [
            const Expanded(child: Divider(color: Color(0xFFE0E0E0))),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Text(
                '其他登录方式',
                style: TextStyle(fontSize: 12, color: Colors.grey[500]),
              ),
            ),
            const Expanded(child: Divider(color: Color(0xFFE0E0E0))),
          ],
        ),
        const SizedBox(height: 20),
        Row(mainAxisAlignment: MainAxisAlignment.center, children: _items()),
      ],
    );
  }

  List<Widget> _items() {
    final items = <Widget>[
      if (showApple &&
          !kIsWeb &&
          (defaultTargetPlatform == TargetPlatform.iOS ||
              defaultTargetPlatform == TargetPlatform.macOS))
        _OtherLoginItem.apple(onTap: loading ? null : onApple),
      if (showGithub) _OtherLoginItem.github(onTap: loading ? null : onGithub),
      if (showPasswordToggle)
        _OtherLoginItem(
          icon: Icons.lock_outline,
          label: isCodeMode ? '密码登录' : '验证码登录',
          onTap: loading ? null : onToggleMode,
        ),
    ];
    return [
      for (var index = 0; index < items.length; index++) ...[
        if (index > 0) const SizedBox(width: 24),
        items[index],
      ],
    ];
  }
}

/// 单个第三方登录入口：GitHub 用 SVG，Apple 用黑色圆角图标。
class _OtherLoginItem extends StatelessWidget {
  final IconData icon;
  final String label;
  final bool dark;
  final bool github;
  final VoidCallback? onTap;

  const _OtherLoginItem({
    required this.icon,
    required this.label,
    required this.onTap,
  }) : dark = false,
       github = false;

  const _OtherLoginItem.github({required this.onTap})
    : icon = Icons.code,
      label = 'GitHub',
      dark = false,
      github = true;

  const _OtherLoginItem.apple({required this.onTap})
    : icon = Icons.apple,
      label = 'Apple',
      dark = true,
      github = false;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Column(
        children: [
          if (github)
            SvgPicture.asset(
              'assets/icons/github.svg',
              package: 'fx_user_ui',
              width: 48,
              height: 48,
            )
          else
            Container(
              width: 48,
              height: 48,
              padding: dark ? const EdgeInsets.all(2) : EdgeInsets.zero,
              decoration: BoxDecoration(
                color: dark ? Colors.transparent : const Color(0xFFF5F5F5),
                borderRadius: BorderRadius.circular(24),
              ),
              child: Container(
                decoration: BoxDecoration(
                  color: dark ? Colors.black : Colors.transparent,
                  borderRadius: BorderRadius.circular(22),
                ),
                child: Icon(
                  icon,
                  size: 22,
                  color: dark ? Colors.white : const Color(0xFF555555),
                ),
              ),
            ),
          const SizedBox(height: 6),
          Text(
            label,
            style: const TextStyle(fontSize: 12, color: Color(0xFF999999)),
          ),
        ],
      ),
    );
  }
}
