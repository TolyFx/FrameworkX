import 'package:flutter_test/flutter_test.dart';
import 'package:fx_user_ui/fx_user_ui.dart';

void main() {
  test('default UI remains product neutral', () {
    final config = FxUserUiConfig();
    expect(config.methods, contains(FxLoginMethod.emailCode));
    expect(config.methods, contains(FxLoginMethod.password));
    expect(config.methods, hasLength(2));
    expect(config.title, 'WELCOME');
  });
}
