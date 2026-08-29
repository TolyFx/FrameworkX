import 'package:flutter/material.dart';

import '../../l10n/fx_account_localizations.dart';

/// 外部提交账户注销密码。
typedef AccountDeletionSubmitted = Future<void> Function(String password);

/// 外部展示页面产生的非字段提示。
typedef AccountPageMessageRequested = void Function(String message);

/// 只负责风险确认和密码采集的账户注销页面。
class DeleteAccountPage extends StatefulWidget {
  /// 由宿主负责的账户注销提交行为。
  final AccountDeletionSubmitted onSubmit;

  /// 页面非字段提示交给宿主展示。
  final AccountPageMessageRequested? onMessage;

  const DeleteAccountPage({super.key, required this.onSubmit, this.onMessage});

  @override
  State<DeleteAccountPage> createState() => _DeleteAccountPageState();
}

class _DeleteAccountPageState extends State<DeleteAccountPage> {
  /// 用于验证当前用户身份的账号密码。
  final TextEditingController _passwordController = TextEditingController();

  /// 用户是否已主动确认不可恢复风险。
  bool _riskAccepted = false;

  /// 注销请求是否正在提交。
  bool _submitting = false;

  /// 密码输入框当前展示的错误。
  String? _errorText;

  @override
  void dispose() {
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final FxAccountLocalizations l10n = FxAccountLocalizations.of(context)!;
    final bool isDark = Theme.of(context).brightness == Brightness.dark;
    final Color background = isDark ? const Color(0xFF121212) : Colors.white;
    final Color foreground = isDark ? Colors.white : Colors.black;
    return Scaffold(
      backgroundColor: background,
      appBar: AppBar(
        backgroundColor: background,
        surfaceTintColor: background,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: true,
        leading: IconButton(
          onPressed: () => Navigator.of(context).pop(),
          icon: Icon(Icons.arrow_back_ios_new, size: 18, color: foreground),
        ),
        title: Text(
          l10n.deleteAccountTitle,
          style: TextStyle(
            color: foreground,
            fontSize: 17,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
      body: SafeArea(
        top: false,
        child: ListView(
          padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 32),
          children: <Widget>[
            _buildWarningCard(context, l10n),
            const SizedBox(height: 32),
            _buildVerificationCard(context, l10n),
            const SizedBox(height: 40),
            _buildSubmitButton(l10n.deleteConfirm),
            const SizedBox(height: 10),
            Text(
              l10n.deleteIrreversible,
              textAlign: TextAlign.center,
              style: TextStyle(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
                fontSize: 12,
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 构建注销后果说明卡片。
  Widget _buildWarningCard(BuildContext context, FxAccountLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Row(
          children: <Widget>[
            Container(
              width: 36,
              height: 36,
              decoration: const BoxDecoration(
                color: Color(0xFFFFEEE9),
                shape: BoxShape.circle,
              ),
              child: const Icon(
                Icons.warning_amber_rounded,
                color: Color(0xFFE5484D),
                size: 21,
              ),
            ),
            const SizedBox(width: 10),
            Text(
              l10n.deleteNoticeTitle,
              style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
            ),
          ],
        ),
        const SizedBox(height: 16),
        _buildWarningItem(l10n.deleteProfileWarning),
        _buildWarningItem(l10n.deleteCloudWarning),
        _buildWarningItem(l10n.deleteLoginWarning),
        _buildWarningItem(l10n.deleteLocalFileNotice),
      ],
    );
  }

  Widget _buildWarningItem(String text) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          const Padding(
            padding: EdgeInsets.only(top: 7),
            child: SizedBox.square(
              dimension: 4,
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: Color(0xFF98A0AE),
                  shape: BoxShape.circle,
                ),
              ),
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              text,
              style: const TextStyle(fontSize: 14, height: 1.4),
            ),
          ),
        ],
      ),
    );
  }

  /// 构建密码验证和风险确认区域。
  Widget _buildVerificationCard(
    BuildContext context,
    FxAccountLocalizations l10n,
  ) {
    final bool isDark = Theme.of(context).brightness == Brightness.dark;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Text(
          l10n.verifyIdentityTitle,
          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
        ),
        const SizedBox(height: 5),
        Text(
          l10n.deletePasswordHelp,
          style: TextStyle(
            color: Theme.of(context).colorScheme.onSurfaceVariant,
            fontSize: 13,
          ),
        ),
        const SizedBox(height: 14),
        Container(
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: isDark ? Colors.grey.shade800 : Colors.grey.shade200,
              ),
            ),
          ),
          child: TextField(
            controller: _passwordController,
            obscureText: true,
            textInputAction: TextInputAction.done,
            onChanged: _handlePasswordChanged,
            onSubmitted: (_) => _requestFinalConfirmation(),
            decoration: InputDecoration(
              hintText: l10n.currentPasswordHint,
              errorText: _errorText,
              border: InputBorder.none,
              contentPadding: const EdgeInsets.symmetric(vertical: 14),
            ),
          ),
        ),
        const SizedBox(height: 14),
        InkWell(
          onTap: _submitting ? null : () => _handleRiskChanged(!_riskAccepted),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                SizedBox.square(
                  dimension: 22,
                  child: Checkbox(
                    value: _riskAccepted,
                    onChanged: _submitting ? null : _handleRiskChanged,
                    visualDensity: VisualDensity.compact,
                  ),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    l10n.deleteRiskAccepted,
                    style: const TextStyle(fontSize: 13, height: 1.4),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildSubmitButton(String label) {
    return SizedBox(
      height: 48,
      child: FilledButton(
        onPressed: _submitting ? null : _requestFinalConfirmation,
        style: FilledButton.styleFrom(
          backgroundColor: const Color(0xFFE5484D),
          disabledBackgroundColor: const Color(
            0xFFE5484D,
          ).withValues(alpha: 0.45),
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
            : Text(label),
      ),
    );
  }

  void _handlePasswordChanged(String value) {
    if (_errorText != null) setState(() => _errorText = null);
  }

  void _handleRiskChanged(bool? accepted) {
    setState(() => _riskAccepted = accepted ?? false);
  }

  /// 校验当前输入后展示最后一次不可逆操作确认。
  Future<void> _requestFinalConfirmation() async {
    final FxAccountLocalizations l10n = FxAccountLocalizations.of(context)!;
    if (_passwordController.text.isEmpty) {
      setState(() => _errorText = l10n.passwordEmpty);
      return;
    }
    if (!_riskAccepted) {
      _showMessage(l10n.deleteAcceptRiskFirst);
      return;
    }
    final bool confirmed =
        await showDialog<bool>(
          context: context,
          builder: (BuildContext dialogContext) => AlertDialog(
            title: Text(l10n.deleteFinalTitle),
            content: Text(l10n.deleteFinalContent),
            actions: <Widget>[
              TextButton(
                onPressed: () => Navigator.pop(dialogContext, false),
                child: Text(l10n.cancel),
              ),
              TextButton(
                onPressed: () => Navigator.pop(dialogContext, true),
                child: Text(
                  l10n.deleteConfirm,
                  style: const TextStyle(color: Color(0xFFE5484D)),
                ),
              ),
            ],
          ),
        ) ??
        false;
    if (confirmed && mounted) await _submit();
  }

  void _showMessage(String message) {
    final AccountPageMessageRequested? onMessage = widget.onMessage;
    if (onMessage != null) {
      onMessage(message);
      return;
    }
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  /// 将密码交给宿主，页面自身只维护提交中的按钮状态。
  Future<void> _submit() async {
    setState(() {
      _submitting = true;
      _errorText = null;
    });
    try {
      await widget.onSubmit(_passwordController.text);
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }
}
