import 'package:flutter/material.dart';

/// 设置密码提交任务。
typedef SetPasswordSubmit = Future<bool> Function(String password);

/// 为尚未拥有密码的账号设置首个密码。
class SetPasswordPage extends StatefulWidget {
  /// 设置密码的业务提交回调。
  final SetPasswordSubmit onSubmit;

  /// 交互消息回调，由宿主接入 Toast 等统一反馈。
  final ValueChanged<String>? onMessage;

  const SetPasswordPage({super.key, required this.onSubmit, this.onMessage});

  @override
  State<SetPasswordPage> createState() => _SetPasswordPageState();
}

class _SetPasswordPageState extends State<SetPasswordPage> {
  /// 密码最小长度。
  static const int _minimumLength = 6;

  /// 新密码输入控制器。
  final TextEditingController _passwordController = TextEditingController();

  /// 确认密码输入控制器。
  final TextEditingController _confirmationController = TextEditingController();

  /// 当前是否正在提交。
  bool _submitting = false;

  /// 两次输入是否已通过本地校验。
  bool get _inputValid {
    final String password = _passwordController.text.trim();
    return password.length >= _minimumLength &&
        password == _confirmationController.text.trim();
  }

  /// 当前是否允许发起提交。
  bool get _canSubmit => _inputValid && !_submitting;

  @override
  void initState() {
    super.initState();
    _passwordController.addListener(_refresh);
    _confirmationController.addListener(_refresh);
  }

  @override
  void dispose() {
    _passwordController.removeListener(_refresh);
    _confirmationController.removeListener(_refresh);
    _passwordController.dispose();
    _confirmationController.dispose();
    super.dispose();
  }

  void _refresh() => setState(() {});

  /// 校验两次密码并提交给宿主。
  Future<void> _submit() async {
    final String password = _passwordController.text.trim();
    final String confirmation = _confirmationController.text.trim();
    if (password.length < _minimumLength) {
      widget.onMessage?.call('密码至少 6 位');
      return;
    }
    if (password != confirmation) {
      widget.onMessage?.call('两次输入的密码不一致');
      return;
    }
    setState(() => _submitting = true);
    try {
      final bool succeeded = await widget.onSubmit(password);
      if (!mounted) return;
      if (!succeeded) {
        widget.onMessage?.call('密码设置失败');
        return;
      }
      Navigator.of(context).pop();
    } catch (_) {
      if (mounted) widget.onMessage?.call('密码设置失败');
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final ThemeData theme = Theme.of(context);
    final bool isDark = theme.brightness == Brightness.dark;
    final Color backgroundColor = isDark
        ? const Color(0xFF121212)
        : Colors.white;
    final Color foregroundColor = isDark ? Colors.white : Colors.black;
    return Scaffold(
      backgroundColor: backgroundColor,
      appBar: AppBar(
        backgroundColor: backgroundColor,
        surfaceTintColor: backgroundColor,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: true,
        leading: IconButton(
          onPressed: () => Navigator.of(context).pop(),
          icon: Icon(
            Icons.arrow_back_ios_new,
            size: 18,
            color: foregroundColor,
          ),
        ),
        title: Text(
          '设置密码',
          style: TextStyle(
            color: foregroundColor,
            fontSize: 17,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
      body: SafeArea(
        top: false,
        child: ListView(
          padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 40),
          children: <Widget>[
            Text(
              '为账号设置一个密码',
              style: TextStyle(
                fontSize: 14,
                color: isDark ? Colors.grey.shade400 : Colors.grey.shade600,
              ),
            ),
            const SizedBox(height: 24),
            _buildInput(
              controller: _passwordController,
              hint: '请输入密码（至少6位）',
              foregroundColor: foregroundColor,
              borderColor: isDark ? Colors.grey.shade800 : Colors.grey.shade200,
              autofocus: true,
            ),
            const SizedBox(height: 16),
            _buildInput(
              controller: _confirmationController,
              hint: '请再次输入密码',
              foregroundColor: foregroundColor,
              borderColor: isDark ? Colors.grey.shade800 : Colors.grey.shade200,
            ),
            const SizedBox(height: 48),
            _buildActionButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildInput({
    required TextEditingController controller,
    required String hint,
    required Color foregroundColor,
    required Color borderColor,
    bool autofocus = false,
  }) {
    return Container(
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: borderColor)),
      ),
      child: TextField(
        controller: controller,
        autofocus: autofocus,
        obscureText: true,
        textInputAction: TextInputAction.next,
        autofillHints: const <String>[AutofillHints.newPassword],
        style: TextStyle(fontSize: 16, color: foregroundColor),
        decoration: InputDecoration(
          hintText: hint,
          border: InputBorder.none,
          contentPadding: const EdgeInsets.symmetric(vertical: 14),
        ),
        onSubmitted: (_) {
          if (_canSubmit) _submit();
        },
      ),
    );
  }

  Widget _buildActionButton() {
    const Color primaryColor = Color(0xFF3B82F6);
    return SizedBox(
      width: double.infinity,
      height: 48,
      child: _inputValid
          ? ElevatedButton(
              onPressed: _submitting ? null : _submit,
              style: ElevatedButton.styleFrom(
                backgroundColor: primaryColor,
                foregroundColor: Colors.white,
                elevation: 0,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(24),
                ),
              ),
              child: _submitting
                  ? const SizedBox.square(
                      dimension: 20,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.white,
                      ),
                    )
                  : const Text('确认', style: TextStyle(fontSize: 16)),
            )
          : OutlinedButton(
              onPressed: null,
              style: OutlinedButton.styleFrom(
                side: BorderSide(color: Colors.grey.shade300),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(24),
                ),
              ),
              child: Text(
                '确认',
                style: TextStyle(fontSize: 16, color: Colors.grey.shade400),
              ),
            ),
    );
  }
}
