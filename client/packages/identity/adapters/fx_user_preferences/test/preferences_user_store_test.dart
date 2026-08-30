import 'package:flutter_test/flutter_test.dart';
import 'package:fx_user_core/fx_user_core.dart';
import 'package:fx_user_preferences/fx_user_preferences.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';

void main() {
  late FxPreferencesUserStore store;

  setUp(() {
    SharedPreferencesAsyncPlatform.instance =
        InMemorySharedPreferencesAsync.empty();
    store = FxPreferencesUserStore(
      namespace: 'test',
      preferences: SharedPreferencesAsync(),
    );
  });

  test('读写和清除认证凭据', () async {
    const BearerCredential credential = BearerCredential(
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    );

    await store.write(credential);
    final UserCredential? restored = await store.read();

    expect(restored, isA<BearerCredential>());
    expect((restored! as BearerCredential).accessToken, 'access-token');

    await store.clear();
    expect(await store.read(), isNull);
  });

  test('读写和清除用户资料快照', () async {
    const FxUser user = FxUser(
      id: '1',
      displayName: 'Toly',
      profile: {'signature': 'hello'},
    );

    await store.writeSnapshot(user);
    final FxUser? restored = await store.readSnapshot();

    expect(restored?.id, '1');
    expect(restored?.displayName, 'Toly');
    expect(restored?.profile['signature'], 'hello');

    await store.clearSnapshot();
    expect(await store.readSnapshot(), isNull);
  });
}
