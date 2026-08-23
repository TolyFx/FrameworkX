import 'exception.dart';

/// 资源引用的基础协议；它只表达位置，不承担文件传输职责。
sealed class FxResourceRef {
  /// 创建资源引用。
  const FxResourceRef(this.rawValue);

  /// 可持久化、可同步的原始引用字符串。
  final String rawValue;

  /// 按引用前缀解析资源类型。
  factory FxResourceRef.parse(String rawValue) {
    if (rawValue.trim().isEmpty) {
      throw FxResourceException(
        FxResourceCode.invalidReference,
        'Resource reference must not be empty',
        ArgumentError.value(rawValue, 'rawValue'),
        StackTrace.current,
        rawValue,
      );
    }
    if (rawValue.startsWith(FxManagedResourceRef.prefix)) {
      final String relativePath = rawValue.substring(
        FxManagedResourceRef.prefix.length,
      );
      if (relativePath.startsWith('/')) {
        return FxExternalResourceRef(relativePath);
      }
      return FxManagedResourceRef(relativePath);
    }
    if (rawValue.startsWith('http://') || rawValue.startsWith('https://')) {
      return FxNetworkResourceRef(rawValue);
    }
    return FxLocalResourceRef(rawValue);
  }

  /// 是否为网络资源。
  bool get isNetwork => this is FxNetworkResourceRef;
}

/// 应用自身管理的本地资源，使用 `resources:` 相对引用。
final class FxManagedResourceRef extends FxResourceRef {
  /// 应用管理资源的稳定前缀。
  static const String prefix = 'resources:';

  /// 相对于应用资源根目录的路径，统一使用 `/` 分隔符。
  final String relativePath;

  /// 创建受管理资源引用。
  FxManagedResourceRef(String relativePath)
      : relativePath = _normalizeRelativePath(relativePath),
        super('$prefix${_normalizeRelativePath(relativePath)}');

  static String _normalizeRelativePath(String value) {
    final String normalizedValue = value.replaceAll('\\', '/');
    final List<String> segments = normalizedValue
        .split('/')
        .where((String segment) => segment.isNotEmpty && segment != '.')
        .toList(growable: false);
    if (segments.isEmpty || segments.any((String segment) => segment == '..')) {
      throw FxResourceException(
        FxResourceCode.unsafeRelativePath,
        'Managed resource must use a safe relative path',
        ArgumentError.value(value, 'relativePath'),
        StackTrace.current,
        value,
      );
    }
    return segments.join('/');
  }
}

/// 外部本地资源，原样保存绝对路径或宿主自定义本地路径。
final class FxLocalResourceRef extends FxResourceRef {
  /// 创建本地资源引用。
  const FxLocalResourceRef(super.rawValue);
}

/// 显式使用 `resources:/` 前缀标记的外部绝对本地资源。
final class FxExternalResourceRef extends FxResourceRef {
  /// 外部资源的绝对本地路径。
  final String path;

  /// 创建外部资源引用。
  FxExternalResourceRef(this.path)
      : super('${FxManagedResourceRef.prefix}$path');
}

/// 远端网络资源。
final class FxNetworkResourceRef extends FxResourceRef {
  /// 创建网络资源引用。
  const FxNetworkResourceRef(super.rawValue);
}
