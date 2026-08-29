import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:fx_account/fx_account.dart';

void main() {
  testWidgets('账号管理页统一展示宿主资料与安全项', (WidgetTester tester) async {
    bool passwordOpened = false;
    await tester.pumpWidget(
      MaterialApp(
        home: AccountManagementPage(
          data: AccountManagementData(
            title: '账号管理',
            avatar: const SizedBox.square(dimension: 48),
            username: 'Toly',
            signature: '',
            userId: '10001',
            userIdLabel: '应用 ID',
            contactItems: const <AccountManagementItem>[
              AccountManagementItem(label: '邮箱', value: 'a@example.com'),
            ],
            hasPassword: true,
            onChangePassword: () async => passwordOpened = true,
            onLogout: () async {},
          ),
        ),
      ),
    );

    expect(find.text('Toly'), findsOneWidget);
    expect(find.text('未设置'), findsOneWidget);
    expect(find.text('a@example.com'), findsOneWidget);
    await tester.tap(find.text('修改密码'));
    expect(passwordOpened, isTrue);
  });

  testWidgets('无密码账号由公共页选择设置密码入口', (WidgetTester tester) async {
    bool passwordOpened = false;
    await tester.pumpWidget(
      MaterialApp(
        home: AccountManagementPage(
          data: AccountManagementData(
            title: '账号管理',
            avatar: const SizedBox.square(dimension: 48),
            username: 'Toly',
            signature: '',
            userId: '10001',
            userIdLabel: '应用 ID',
            hasPassword: false,
            onSetPassword: () async => passwordOpened = true,
            onLogout: () async {},
          ),
        ),
      ),
    );

    expect(find.text('设置密码'), findsOneWidget);
    expect(find.text('修改密码'), findsNothing);
    await tester.tap(find.text('设置密码'));
    expect(passwordOpened, isTrue);
  });
}
