import 'package:flutter_test/flutter_test.dart';
import 'package:fx_resource/fx_resource.dart';

void main() {
  test('parses a managed resource reference', () {
    final FxResourceRef reference =
        FxResourceRef.parse('resources:picked_images/a.png');

    expect(reference, isA<FxManagedResourceRef>());
    expect(reference.rawValue, 'resources:picked_images/a.png');
  });

  test('parses an external local resource reference', () {
    final FxResourceRef reference =
        FxResourceRef.parse('resources:/Users/me/a.png');

    expect(reference, isA<FxExternalResourceRef>());
    expect(reference.rawValue, 'resources:/Users/me/a.png');
  });

  test('rejects a managed resource path containing parent traversal', () {
    expect(
      () => FxManagedResourceRef('../private/a.png'),
      throwsA(isA<FxResourceException>()),
    );
  });

  test('returns a structured format error for an empty reference', () {
    expect(
      () => FxResourceRef.parse(''),
      throwsA(
        isA<FxResourceException>().having(
          (FxResourceException exception) => exception.code,
          'code',
          FxResourceCode.invalidReference,
        ),
      ),
    );
  });
}
