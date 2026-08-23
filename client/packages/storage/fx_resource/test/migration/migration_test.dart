import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:fx_resource/fx_resource.dart';

/// 用于验证宿主可追加自定义历史路径规则的测试策略。
final class _TestLegacyPathStrategy extends FxLegacyPathStrategy {
  const _TestLegacyPathStrategy();

  @override
  String? relativePathOf(String path) {
    const String prefix = 'legacy://';
    return path.startsWith(prefix) ? path.substring(prefix.length) : null;
  }
}

/// 用于验证策略自身异常会被迁移边界包装的测试策略。
final class _FailingLegacyPathStrategy extends FxLegacyPathStrategy {
  const _FailingLegacyPathStrategy();

  @override
  String? relativePathOf(String path) {
    throw StateError('Legacy path strategy failed');
  }
}

void main() {
  test('migrates a legacy iOS sandbox path to a Documents reference', () {
    final Directory rootDirectory =
        Directory.systemTemp.createTempSync('fx_resource_');
    final Directory imageDirectory =
        Directory('${rootDirectory.path}/picked_images');
    imageDirectory.createSync();
    File('${imageDirectory.path}/a.png').writeAsBytesSync(<int>[0]);
    final FxFileResourceStore store = FxFileResourceStore(rootDirectory);
    final FxResourceMigration migration = FxResourceMigration(store);
    const String legacyPath =
        '/Users/me/Library/Developer/CoreSimulator/Devices/1/data/Containers/Data/Application/old/Documents/picked_images/a.png';

    final String migratedPath = migration.migrate(legacyPath);

    expect(migratedPath, 'resources:picked_images/a.png');
    rootDirectory.deleteSync(recursive: true);
  });

  test('allows a host to append custom legacy path strategies', () {
    final Directory rootDirectory =
        Directory.systemTemp.createTempSync('fx_resource_');
    final Directory imageDirectory =
        Directory('${rootDirectory.path}/external_images');
    imageDirectory.createSync();
    File('${imageDirectory.path}/b.png').writeAsBytesSync(<int>[0]);
    final FxFileResourceStore store = FxFileResourceStore(rootDirectory);
    final FxResourceMigration migration = FxResourceMigration(
      store,
      strategies: const <FxLegacyPathStrategy>[_TestLegacyPathStrategy()],
    );

    final String migratedPath =
        migration.migrate('legacy://external_images/b.png');

    expect(migratedPath, 'resources:external_images/b.png');
    rootDirectory.deleteSync(recursive: true);
  });

  test('preserves empty and invalid references without blocking migration', () {
    final Directory rootDirectory =
        Directory.systemTemp.createTempSync('fx_resource_');
    final FxFileResourceStore store = FxFileResourceStore(rootDirectory);
    final FxResourceMigration migration = FxResourceMigration(store);

    expect(migration.migrate(''), '');
    expect(migration.migrate('resources:../outside.png'),
        'resources:../outside.png');

    rootDirectory.deleteSync(recursive: true);
  });

  test('wraps legacy path strategy errors as resource migration errors', () {
    final Directory rootDirectory =
        Directory.systemTemp.createTempSync('fx_resource_');
    final FxFileResourceStore store = FxFileResourceStore(rootDirectory);
    final FxResourceMigration migration = FxResourceMigration(
      store,
      strategies: const <FxLegacyPathStrategy>[_FailingLegacyPathStrategy()],
    );

    expect(
      () => migration.migrate('legacy://broken.png'),
      throwsA(
        isA<FxResourceException>().having(
          (FxResourceException exception) => exception.code,
          'code',
          FxResourceCode.migrationFailed,
        ),
      ),
    );
    rootDirectory.deleteSync(recursive: true);
  });
}
