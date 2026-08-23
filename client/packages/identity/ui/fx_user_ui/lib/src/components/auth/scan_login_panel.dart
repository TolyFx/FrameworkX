import 'dart:async';

import 'package:flutter/material.dart';
import 'package:fx_user_core/fx_user_core.dart';
import 'package:qr_flutter/qr_flutter.dart';

class ScanLoginPanel extends StatefulWidget {
  final Future<FxScanSession> Function() createSession;
  final Future<FxScanStatus> Function(String token) pollStatus;
  final Future<void> Function(String credential) onAuthenticated;

  const ScanLoginPanel({
    super.key,
    required this.createSession,
    required this.pollStatus,
    required this.onAuthenticated,
  });

  @override
  State<ScanLoginPanel> createState() => _ScanLoginPanelState();
}

class _ScanLoginPanelState extends State<ScanLoginPanel> {
  FxScanSession? session;
  String status = 'loading';
  Timer? timer;

  @override
  void initState() {
    super.initState();
    _create();
  }

  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  Future<void> _create() async {
    timer?.cancel();
    setState(() => status = 'loading');
    try {
      final value = await widget.createSession();
      if (!mounted) return;
      setState(() {
        session = value;
        status = 'pending';
      });
      timer = Timer.periodic(const Duration(seconds: 2), (_) => _poll());
    } catch (_) {
      if (mounted) setState(() => status = 'error');
    }
  }

  Future<void> _poll() async {
    final current = session;
    if (current == null) return;
    try {
      final result = await widget.pollStatus(current.token);
      if (!mounted) return;
      setState(() => status = result.status);
      if (result.status == 'confirmed' && result.credential != null) {
        timer?.cancel();
        await widget.onAuthenticated(result.credential!);
      } else if (result.status == 'expired') {
        timer?.cancel();
      }
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final current = session;
    if (status == 'loading') {
      return const SizedBox(
        height: 240,
        child: Center(child: CircularProgressIndicator()),
      );
    }
    if (status == 'error' || current == null) {
      return SizedBox(
        height: 240,
        child: Center(
          child: TextButton.icon(
            onPressed: _create,
            icon: const Icon(Icons.refresh),
            label: const Text('重新加载二维码'),
          ),
        ),
      );
    }
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Stack(
          alignment: Alignment.center,
          children: [
            QrImageView(data: current.content, size: 220),
            if (status == 'expired')
              ColoredBox(
                color: Colors.white.withValues(alpha: .92),
                child: SizedBox(
                  width: 220,
                  height: 220,
                  child: Center(
                    child: TextButton(
                      onPressed: _create,
                      child: const Text('二维码已过期，点击刷新'),
                    ),
                  ),
                ),
              ),
            if (status == 'scanned')
              const ColoredBox(
                color: Color(0xEEFFFFFF),
                child: SizedBox(
                  width: 220,
                  height: 220,
                  child: Center(child: Text('已扫码，请在手机上确认')),
                ),
              ),
          ],
        ),
        const SizedBox(height: 14),
        const Text('使用已登录的移动端扫描二维码', style: TextStyle(color: Colors.grey)),
      ],
    );
  }
}
