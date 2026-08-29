import 'package:flutter/material.dart';

/// 账号管理页面点击事件。
typedef AccountManagementAction = Future<void> Function();

/// 账号管理页中的可配置资料或安全项。
class AccountManagementItem {
  /// 左侧字段名称。
  final String label;

  /// 右侧展示文本；为空时仅展示操作入口。
  final String value;

  /// 点击行为。
  final AccountManagementAction? onTap;

  /// 字段名称颜色。
  final Color? labelColor;

  /// 展示文本颜色。
  final Color? valueColor;

  /// 是否使用占位文本样式。
  final bool placeholder;

  const AccountManagementItem({
    required this.label,
    this.value = '',
    this.onTap,
    this.labelColor,
    this.valueColor,
    this.placeholder = false,
  });
}

/// 账号管理页展示的数据与宿主能力。
class AccountManagementData {
  /// 页面标题。
  final String title;

  /// 用户头像。
  final Widget avatar;

  /// 用户名。
  final String username;

  /// 个性签名。
  final String signature;

  /// 用户标识。
  final String userId;

  /// 用户标识名称。
  final String userIdLabel;

  /// 空签名占位文本。
  final String emptySignatureLabel;

  /// 点击头像的行为。
  final AccountManagementAction? onAvatarTap;

  /// 点击用户名的行为。
  final AccountManagementAction? onUsernameTap;

  /// 点击个性签名的行为。
  final AccountManagementAction? onSignatureTap;

  /// 复制用户标识的行为。
  final AccountManagementAction? onCopyUserId;

  /// 注销账号的行为。
  final AccountManagementAction? onDeleteAccount;

  /// 位于用户标识之后的联系方式等资料项。
  final List<AccountManagementItem> contactItems;

  /// 修改密码等安全操作项。
  final List<AccountManagementItem> securityItems;

  /// 当前账号是否已设置密码。
  final bool hasPassword;

  /// 尚未拥有密码时的设置入口。
  final AccountManagementAction? onSetPassword;

  /// 已拥有密码时的修改入口。
  final AccountManagementAction? onChangePassword;

  /// 退出登录的行为。
  final AccountManagementAction onLogout;

  const AccountManagementData({
    required this.title,
    required this.avatar,
    required this.username,
    required this.signature,
    required this.userId,
    required this.userIdLabel,
    required this.onLogout,
    this.emptySignatureLabel = '未设置',
    this.onAvatarTap,
    this.onUsernameTap,
    this.onSignatureTap,
    this.onCopyUserId,
    this.onDeleteAccount,
    this.contactItems = const <AccountManagementItem>[],
    this.securityItems = const <AccountManagementItem>[],
    this.hasPassword = false,
    this.onSetPassword,
    this.onChangePassword,
  });
}

/// ViewX 风格的公共账号管理页。
class AccountManagementPage extends StatelessWidget {
  /// 页面数据与交互能力。
  final AccountManagementData data;

  const AccountManagementPage({super.key, required this.data});

  @override
  Widget build(BuildContext context) {
    final bool isDark = Theme.of(context).brightness == Brightness.dark;
    final Color backgroundColor = isDark
        ? Colors.black
        : const Color(0xffF5F5F5);
    final Color textColor = isDark ? Colors.white : const Color(0xff333333);
    final Color appBarColor = isDark ? const Color(0xff121318) : Colors.white;
    final Color tileColor = isDark ? const Color(0xff121318) : Colors.white;
    return Scaffold(
      backgroundColor: backgroundColor,
      appBar: AppBar(
        backgroundColor: appBarColor,
        surfaceTintColor: appBarColor,
        elevation: 0.5,
        centerTitle: true,
        leading: IconButton(
          onPressed: () => Navigator.of(context).pop(),
          icon: Icon(Icons.arrow_back_ios, size: 18, color: textColor),
        ),
        title: Text(
          data.title,
          style: TextStyle(
            color: textColor,
            fontSize: 17,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
      body: DefaultTextStyle(
        style: TextStyle(fontSize: 16, color: textColor),
        child: ListView(
          children: <Widget>[
            const SizedBox(height: 8),
            _AccountGroup(
              color: tileColor,
              children: <Widget>[
                _AccountRow(
                  label: '头像',
                  value: data.avatar,
                  onTap: data.onAvatarTap,
                  verticalPadding: 12,
                ),
                _AccountRow(
                  label: '用户名',
                  value: _AccountValue(text: data.username),
                  onTap: data.onUsernameTap,
                ),
                _AccountRow(
                  label: '个性签名',
                  value: _AccountValue(
                    text: data.signature.isEmpty
                        ? data.emptySignatureLabel
                        : data.signature,
                    placeholder: data.signature.isEmpty,
                  ),
                  onTap: data.onSignatureTap,
                ),
              ],
            ),
            const SizedBox(height: 8),
            _AccountGroup(
              color: tileColor,
              children: <Widget>[
                _AccountRow(
                  label: data.userIdLabel,
                  value: _AccountValue(text: data.userId),
                  trailing: data.onCopyUserId == null
                      ? null
                      : IconButton(
                          visualDensity: VisualDensity.compact,
                          onPressed: data.onCopyUserId,
                          icon: const Icon(
                            Icons.copy,
                            size: 18,
                            color: Color(0xffBDBDBD),
                          ),
                        ),
                ),
                ...data.contactItems.map(_buildConfiguredRow),
              ],
            ),
            if (_passwordAction != null ||
                data.securityItems.isNotEmpty ||
                data.onDeleteAccount != null) ...<Widget>[
              const SizedBox(height: 8),
              _AccountGroup(
                color: tileColor,
                children: <Widget>[
                  if (_passwordAction != null)
                    _AccountRow(
                      label: data.hasPassword ? '修改密码' : '设置密码',
                      value: const SizedBox.shrink(),
                      onTap: _passwordAction,
                    ),
                  ...data.securityItems.map(_buildConfiguredRow),
                  if (data.onDeleteAccount != null)
                    _AccountRow(
                      label: '注销账号',
                      labelColor: Colors.red,
                      value: const SizedBox.shrink(),
                      onTap: data.onDeleteAccount,
                    ),
                ],
              ),
            ],
            const SizedBox(height: 24),
            InkWell(
              onTap: data.onLogout,
              child: Container(
                height: 54,
                color: tileColor,
                alignment: Alignment.center,
                child: const Text(
                  '退出登录',
                  style: TextStyle(fontSize: 16, color: Colors.red),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 根据公共账号状态选择密码操作，宿主不再重复判断。
  AccountManagementAction? get _passwordAction =>
      data.hasPassword ? data.onChangePassword : data.onSetPassword;

  /// 将宿主配置转换为统一样式的账号资料行。
  Widget _buildConfiguredRow(AccountManagementItem item) {
    return _AccountRow(
      label: item.label,
      labelColor: item.labelColor,
      value: _AccountValue(
        text: item.value,
        placeholder: item.placeholder,
        color: item.valueColor,
      ),
      onTap: item.onTap,
    );
  }
}

class _AccountGroup extends StatelessWidget {
  /// 分组背景色。
  final Color color;

  /// 分组内的资料行。
  final List<Widget> children;

  const _AccountGroup({required this.color, required this.children});

  @override
  Widget build(BuildContext context) {
    final List<Widget> separated = <Widget>[];
    for (int index = 0; index < children.length; index++) {
      separated.add(children[index]);
      if (index < children.length - 1) {
        separated.add(
          const Divider(
            height: 0.5,
            thickness: 0.5,
            indent: 16,
            color: Color(0xffEEEEEE),
          ),
        );
      }
    }
    return ColoredBox(
      color: color,
      child: Column(children: separated),
    );
  }
}

class _AccountRow extends StatelessWidget {
  /// 左侧字段名称。
  final String label;

  /// 右侧字段内容。
  final Widget value;

  /// 点击行为。
  final AccountManagementAction? onTap;

  /// 自定义尾部组件。
  final Widget? trailing;

  /// 行的垂直内边距。
  final double verticalPadding;

  /// 左侧字段名称颜色。
  final Color? labelColor;

  const _AccountRow({
    required this.label,
    required this.value,
    this.onTap,
    this.trailing,
    this.verticalPadding = 16,
    this.labelColor,
  });

  @override
  Widget build(BuildContext context) {
    final Widget? end =
        trailing ??
        (onTap == null
            ? null
            : const Icon(
                Icons.chevron_right,
                size: 20,
                color: Color(0xffBDBDBD),
              ));
    return InkWell(
      onTap: onTap,
      child: Padding(
        padding: EdgeInsets.symmetric(
          horizontal: 16,
          vertical: verticalPadding,
        ),
        child: Row(
          children: <Widget>[
            SizedBox(
              width: 104,
              child: Text(
                label,
                style: const TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ).copyWith(color: labelColor),
              ),
            ),
            Expanded(child: value),
            if (end != null) ...<Widget>[const SizedBox(width: 4), end],
          ],
        ),
      ),
    );
  }
}

class _AccountValue extends StatelessWidget {
  /// 展示文本。
  final String text;

  /// 是否使用未设置占位色。
  final bool placeholder;

  /// 自定义展示文本颜色。
  final Color? color;

  const _AccountValue({
    required this.text,
    this.placeholder = false,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Text(
      text,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      textAlign: TextAlign.right,
      style: TextStyle(
        color: color ?? (placeholder ? Colors.grey[400] : Colors.grey[600]),
        fontSize: 16,
      ),
    );
  }
}
