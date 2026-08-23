import 'package:flutter/material.dart';

import '../auth/login_types.dart';

/// 品牌头部：Logo、标题、副标题。移动端与桌面端共用，桌面端可加白色底框。
class BrandHeader extends StatelessWidget {
  final FxUserUiConfig config;
  final bool desktop;

  const BrandHeader({super.key, required this.config, this.desktop = false});

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (config.logo != null)
          Container(
            width: 72,
            height: 72,
            padding: desktop ? const EdgeInsets.all(10) : EdgeInsets.zero,
            decoration: desktop
                ? BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(16),
                  )
                : null,
            child: config.logo,
          ),
        const SizedBox(height: 8),
        Text(
          config.title,
          style: const TextStyle(
            fontSize: 30,
            fontWeight: FontWeight.w900,
            letterSpacing: 2,
          ),
        ),
        if (config.subtitle.isNotEmpty) ...[
          const SizedBox(height: 6),
          Text(
            config.subtitle,
            textAlign: TextAlign.center,
            style: const TextStyle(
              fontSize: 14,
              color: Color(0xFF64748B),
              letterSpacing: 4,
            ),
          ),
        ],
      ],
    );
  }
}
