import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:fx_resource/fx_resource.dart';

void main() {
  test('converts a Documents file to a stable reference and resolves it', () {
    final Directory rootDirectory =
        Directory.systemTemp.createTempSync('fx_resource_');
    final Directory imageDirectory =
        Directory('${rootDirectory.path}/picked_images');
    imageDirectory.createSync();
    final File imageFile = File('${imageDirectory.path}/a.png');
    imageFile.writeAsBytesSync(<int>[0]);
    final FxFileResourceStore store = FxFileResourceStore(rootDirectory);

    final FxResourceRef reference = store.referenceForPath(imageFile.path);

    expect(reference.rawValue, 'resources:picked_images/a.png');
    expect(store.resolvePath(reference.rawValue), imageFile.path);
    expect(store.exists(reference.rawValue), isTrue);

    rootDirectory.deleteSync(recursive: true);
  });

  test('imports an external file into the managed resource root', () async {
    final Directory rootDirectory =
        Directory.systemTemp.createTempSync('fx_resource_root_');
    final Directory sourceDirectory =
        Directory.systemTemp.createTempSync('fx_resource_source_');
    final File sourceFile = File('${sourceDirectory.path}/image.png');
    await sourceFile.writeAsBytes(<int>[1, 2, 3]);
    final FxFileResourceStore store = FxFileResourceStore(rootDirectory);

    final FxManagedResourceRef reference = await store.importFile(
      sourceFile.path,
    );
    final File importedFile = File(store.resolvePath(reference.rawValue));

    expect(reference.rawValue, startsWith('resources:imports/'));
    expect(importedFile.existsSync(), isTrue);
    expect(await importedFile.readAsBytes(), <int>[1, 2, 3]);

    rootDirectory.deleteSync(recursive: true);
    sourceDirectory.deleteSync(recursive: true);
  });
}
