import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../components/auth/login_form_body.dart';
import '../../../components/auth/login_types.dart';
import '../../../components/auth/scan_login_panel.dart';
import 'login_view_desktop.dart';
import 'login_view_mobile.dart';

class FxLoginPage extends StatefulWidget {
  final FxUserUiConfig config;
  final FxLoginSubmit onLogin;
  final FxVerificationCodeRequest onRequestCode;
  final VoidCallback? onAuthenticated;
  final VoidCallback? onClose;
  final FxLoginErrorListener? onError;
  final Duration codeCooldown;

  const FxLoginPage({
    super.key,
    required this.config,
    required this.onLogin,
    required this.onRequestCode,
    this.onAuthenticated,
    this.onClose,
    this.onError,
    this.codeCooldown = const Duration(seconds: 60),
  });

  @override
  State<FxLoginPage> createState() => _FxLoginPageState();
}

class _FxLoginPageState extends State<FxLoginPage> {
  final _identifier = TextEditingController();
  final _credential = TextEditingController();
  late FxLoginMethod _method;
  bool _agreed = false;
  bool _submitting = false;
  bool _sendingCode = false;
  int _cooldownSeconds = 0;
  Object? _error;
  Timer? _timer;

  @override
  void initState() {
    super.initState();
    _method = widget.config.methods.first;
    _identifier.addListener(_refresh);
    _credential.addListener(_refresh);
  }

  @override
  void dispose() {
    _timer?.cancel();
    _identifier
      ..removeListener(_refresh)
      ..dispose();
    _credential
      ..removeListener(_refresh)
      ..dispose();
    super.dispose();
  }

  void _refresh() => mounted ? setState(() {}) : null;

  void _showToast(String message) => ScaffoldMessenger.of(
    context,
  ).showSnackBar(SnackBar(content: Text(message)));

  @override
  Widget build(BuildContext context) {
    final Widget formBody = _method == FxLoginMethod.scan
        ? _buildScanPanel()
        : LoginFormBody(
            identifierController: _identifier,
            credentialController: _credential,
            method: _method,
            agreed: _agreed,
            submitting: _submitting,
            sendingCode: _sendingCode,
            cooldownSeconds: _cooldownSeconds,
            error: _error,
            config: widget.config,
            onAgreedChanged: () => setState(() => _agreed = !_agreed),
            onRequestCode: _requestCode,
            onSubmit: _submit,
          );

    return AnnotatedRegion<SystemUiOverlayStyle>(
      value: const SystemUiOverlayStyle(
        systemNavigationBarColor: Colors.transparent,
        statusBarColor: Colors.transparent,
        statusBarIconBrightness: Brightness.dark,
      ),
      child: Scaffold(
        backgroundColor: const Color(0xFFF8FAFC),
        body: Stack(
          children: <Widget>[
            LayoutBuilder(
              builder: (context, constraints) => constraints.maxWidth >= 768
                  ? LoginViewDesktop(
                      config: widget.config,
                      method: _method,
                      formBody: formBody,
                      onMethodChanged: _selectMethod,
                    )
                  : LoginViewMobile(
                      config: widget.config,
                      method: _method,
                      formBody: formBody,
                      submitting: _submitting,
                      onMethodChanged: _selectMethod,
                      onToggleMode: _toggleMode,
                      onThirdPartyLogin: _thirdPartyLogin,
                    ),
            ),
            if (widget.onClose != null)
              SafeArea(
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: IconButton(
                    tooltip: '关闭',
                    onPressed: widget.onClose,
                    icon: const Icon(Icons.close_rounded),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildScanPanel() {
    final createSession = widget.config.createScanSession;
    final pollStatus = widget.config.pollScanStatus;
    final onAuthenticated = widget.config.onScanAuthenticated;
    if (createSession == null ||
        pollStatus == null ||
        onAuthenticated == null) {
      return const SizedBox(
        height: 240,
        child: Center(child: Text('当前宿主未配置扫码登录')),
      );
    }
    return ScanLoginPanel(
      createSession: createSession,
      pollStatus: pollStatus,
      onAuthenticated: onAuthenticated,
    );
  }

  Future<void> _requestCode() async {
    if (!_agreed) {
      _showToast('请先阅读并同意用户协议和隐私政策');
      return;
    }
    if (_sendingCode || _cooldownSeconds > 0) return;
    setState(() {
      _sendingCode = true;
      _error = null;
    });
    try {
      final code = await widget.onRequestCode(
        method: _method,
        identifier: _identifier.text.trim(),
      );
      if (code != null) _credential.text = code;
      _startCooldown();
    } catch (error) {
      _handleError(error);
    } finally {
      if (mounted) setState(() => _sendingCode = false);
    }
  }

  Future<void> _submit() async {
    setState(() {
      _submitting = true;
      _error = null;
    });
    try {
      await widget.onLogin(
        method: _method,
        identifier: _identifier.text.trim(),
        credential: _credential.text.trim(),
      );
      widget.onAuthenticated?.call();
    } catch (error) {
      _handleError(error);
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  Future<void> _thirdPartyLogin(FxThirdPartyLogin? login) async {
    if (!_agreed) {
      _showToast('请先阅读并同意用户协议和隐私政策');
      return;
    }
    if (login == null) {
      _showToast('当前宿主未配置该登录方式');
      return;
    }
    try {
      final bool authenticated = await login();
      if (authenticated) widget.onAuthenticated?.call();
    } catch (error) {
      _handleError(error);
    }
  }

  /// 将交互异常交给宿主；未注入处理器时保留内联错误作为兼容行为。
  void _handleError(Object error) {
    if (!mounted) return;
    final FxLoginErrorListener? listener = widget.onError;
    if (listener != null) {
      listener(error);
      return;
    }
    setState(() => _error = error);
  }

  void _selectMethod(FxLoginMethod value) {
    setState(() {
      _method = value;
      _error = null;
      _credential.clear();
    });
  }

  void _toggleMode() {
    final usesCode = _method != FxLoginMethod.password;
    _selectMethod(
      usesCode
          ? FxLoginMethod.password
          : widget.config.methods.contains(FxLoginMethod.phoneCode)
          ? FxLoginMethod.phoneCode
          : FxLoginMethod.emailCode,
    );
  }

  void _startCooldown() {
    _timer?.cancel();
    setState(() => _cooldownSeconds = widget.codeCooldown.inSeconds);
    _timer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (!mounted) return timer.cancel();
      setState(() {
        _cooldownSeconds--;
        if (_cooldownSeconds <= 0) timer.cancel();
      });
    });
  }
}
