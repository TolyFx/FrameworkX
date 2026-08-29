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
      child: enabled || loading
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
                  ? const SizedBox.square(
                      dimension: 18,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.white,
                      ),
                    )
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
