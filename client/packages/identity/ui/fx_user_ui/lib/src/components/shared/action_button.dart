import 'package:flutter/material.dart';

import 'style.dart';

/// 登录按钮：灰色禁用态、蓝色可用态和三点加载动画。
class ActionButton extends StatelessWidget {
  final bool enabled;
  final bool loading;
  final VoidCallback onPressed;

  const ActionButton({
    super.key,
    required this.enabled,
    required this.loading,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      height: 48,
      child: enabled
          ? ElevatedButton(
              onPressed: loading ? null : onPressed,
              style: ElevatedButton.styleFrom(
                elevation: 0,
                backgroundColor: fxPrimary,
                disabledBackgroundColor: fxPrimary,
                foregroundColor: Colors.white,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(24),
                ),
              ),
              child: loading
                  ? const _BouncingDots()
                  : const Text('登录', style: TextStyle(fontSize: 16)),
            )
          : OutlinedButton(
              onPressed: null,
              style: OutlinedButton.styleFrom(
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(24),
                ),
                side: BorderSide(color: Colors.grey[300]!),
              ),
              child: Text(
                '登录',
                style: TextStyle(fontSize: 16, color: Colors.grey[400]),
              ),
            ),
    );
  }
}

class _BouncingDots extends StatefulWidget {
  const _BouncingDots();

  @override
  State<_BouncingDots> createState() => _BouncingDotsState();
}

class _BouncingDotsState extends State<_BouncingDots>
    with TickerProviderStateMixin {
  late final List<AnimationController> _controllers;
  late final List<Animation<double>> _animations;

  @override
  void initState() {
    super.initState();
    _controllers = List.generate(
      3,
      (_) => AnimationController(
        duration: const Duration(milliseconds: 400),
        vsync: this,
      ),
    );
    _animations = _controllers
        .map(
          (controller) => Tween<double>(begin: 0, end: -8).animate(
            CurvedAnimation(parent: controller, curve: Curves.easeInOut),
          ),
        )
        .toList();
    for (var index = 0; index < _controllers.length; index++) {
      Future<void>.delayed(Duration(milliseconds: index * 150), () {
        if (mounted) _controllers[index].repeat(reverse: true);
      });
    }
  }

  @override
  void dispose() {
    for (final controller in _controllers) {
      controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 2),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(
          3,
          (index) => AnimatedBuilder(
            animation: _animations[index],
            builder: (_, child) => Transform.translate(
              offset: Offset(0, _animations[index].value),
              child: child,
            ),
            child: Container(
              margin: const EdgeInsets.symmetric(horizontal: 2),
              width: 6,
              height: 6,
              decoration: const BoxDecoration(
                color: Colors.white,
                shape: BoxShape.circle,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
