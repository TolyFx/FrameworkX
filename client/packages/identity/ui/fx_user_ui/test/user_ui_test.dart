import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:fx_user_ui/fx_user_ui.dart';

void main() {
  test('default UI remains product neutral', () {
    final FxUserUiConfig config = FxUserUiConfig();
    expect(config.methods, contains(FxLoginMethod.emailCode));
    expect(config.methods, contains(FxLoginMethod.password));
    expect(config.methods, hasLength(2));
    expect(config.title, 'WELCOME');
  });

  testWidgets('移动登录页空间充足时禁止竖向拖动', (WidgetTester tester) async {
    _useMobileViewport(tester);
    await tester.pumpWidget(_buildLoginApp(FxUserUiConfig()));

    final SingleChildScrollView scrollView = tester.widget(
      find.byType(SingleChildScrollView),
    );

    expect(scrollView.physics, isA<NeverScrollableScrollPhysics>());
  });

  testWidgets('第三方登录等待期间展示 loading', (WidgetTester tester) async {
    _useMobileViewport(tester);
    final Completer<bool> pending = Completer<bool>();
    final FxUserUiConfig config = FxUserUiConfig(
      showGithub: true,
      showApple: false,
      onGithubLogin: () => pending.future,
    );
    await tester.pumpWidget(_buildLoginApp(config));
    await tester.tap(find.textContaining('登录即代表'));
    await tester.ensureVisible(find.text('GitHub'));
    await tester.tap(find.text('GitHub'));
    await tester.pump();

    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    pending.complete(false);
    await tester.pumpAndSettle();
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });
}

void _useMobileViewport(WidgetTester tester) {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = const Size(390, 844);
  addTearDown(tester.view.resetDevicePixelRatio);
  addTearDown(tester.view.resetPhysicalSize);
}

Widget _buildLoginApp(FxUserUiConfig config) {
  return MaterialApp(
    home: FxLoginPage(
      config: config,
      onLogin:
          ({
            required FxLoginMethod method,
            required String identifier,
            required String credential,
          }) async {},
      onRequestCode:
          ({required FxLoginMethod method, required String identifier}) async =>
              null,
    ),
  );
}
