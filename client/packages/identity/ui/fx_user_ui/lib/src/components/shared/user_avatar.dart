import 'package:flutter/material.dart';

/// 用户公共头像渲染组件。
///
/// 统一处理 identicon、远程地址和基于 [baseUri] 的相对地址；地址不可用或
/// 加载失败时回退到占位图。
class FxUserAvatar extends StatelessWidget {
  final String? avatar;
  final double size;
  final double borderRadius;
  final Uri? baseUri;

  const FxUserAvatar({
    super.key,
    this.avatar,
    this.size = 40,
    this.borderRadius = 6,
    this.baseUri,
  });

  @override
  Widget build(BuildContext context) {
    final value = avatar ?? '';
    if (value.startsWith('identicon:')) {
      return ClipRRect(
        borderRadius: BorderRadius.circular(borderRadius),
        child: CustomPaint(
          size: Size.square(size),
          painter: _IdenticonPainter(value.substring('identicon:'.length)),
        ),
      );
    }
    final parsed = Uri.tryParse(value);
    if (parsed != null) {
      final Uri? imageUri = _resolveAvatarUri(parsed);
      if (imageUri != null) {
        return ClipRRect(
          borderRadius: BorderRadius.circular(borderRadius),
          child: Image.network(
            imageUri.toString(),
            width: size,
            height: size,
            fit: BoxFit.cover,
            errorBuilder: (_, _, _) => _placeholder(),
          ),
        );
      }
    }
    return _placeholder();
  }

  Uri? _resolveAvatarUri(Uri parsed) {
    if (parsed.isAbsolute) {
      if (!['http', 'https'].contains(parsed.scheme)) return null;
      return parsed;
    }
    if (baseUri == null) return null;
    if (!parsed.hasAuthority && parsed.path.isNotEmpty) {
      return baseUri!.resolveUri(parsed);
    }
    return null;
  }

  Widget _placeholder() => Container(
    width: size,
    height: size,
    decoration: BoxDecoration(
      color: const Color(0xFFE5EEFB),
      borderRadius: BorderRadius.circular(borderRadius),
    ),
    child: Icon(Icons.person, color: const Color(0xFF3B82F6), size: size * .55),
  );
}

class _IdenticonPainter extends CustomPainter {
  final String seed;

  const _IdenticonPainter(this.seed);

  @override
  void paint(Canvas canvas, Size size) {
    final hash = _hash(seed);
    final hue = (hash[0] + hash[1] * 256) % 360;
    final foreground = HSLColor.fromAHSL(1, hue.toDouble(), .6, .5).toColor();
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = const Color(0xFFEEEEEE),
    );
    final padding = size.width * .15;
    final cell = (size.width - padding * 2) / 5;
    final paint = Paint()..color = foreground;
    for (var row = 0; row < 5; row++) {
      for (var column = 0; column < 3; column++) {
        final bitIndex = row * 3 + column;
        final bit = (hash[2 + bitIndex ~/ 8] >> (bitIndex % 8)) & 1;
        if (bit == 0) continue;
        for (final current in {column, 4 - column}) {
          canvas.drawRect(
            Rect.fromLTWH(
              padding + current * cell,
              padding + row * cell,
              cell,
              cell,
            ),
            paint,
          );
        }
      }
    }
  }

  List<int> _hash(String value) {
    final result = List<int>.filled(16, 0);
    var hash = 5381;
    for (final code in value.codeUnits) {
      hash = ((hash << 5) + hash + code) & 0xFFFFFFFF;
    }
    for (var index = 0; index < result.length; index++) {
      result[index] = (hash >> (index * 2)) & 0xFF;
      hash = ((hash << 3) + hash + index + 1) & 0xFFFFFFFF;
    }
    return result;
  }

  @override
  bool shouldRepaint(_IdenticonPainter oldDelegate) => oldDelegate.seed != seed;
}
