import 'dart:io';

import '../domain/exception.dart';
import '../domain/reference.dart';
import '../migration/ios_sandbox_path_strategy.dart';
import '../migration/legacy_path_strategy.dart';
import 'store.dart';

/// 基于应用 Documents 目录的文件资源存储实现。
final class FxFileResourceStore
    implements FxResourceStore, FxResourceImportStore {
  /// 受管理资源的绝对根目录。
  final Directory _rootDirectory;

  /// 解析旧绝对路径时使用的兼容策略。
  final List<FxLegacyPathStrategy> _legacyPathStrategies;

  /// 同一微秒内导入多个文件时的递增序号。
  int _nextImportSequence = 0;

  /// 以既有资源根目录创建存储，便于测试和特殊宿主接入。
  FxFileResourceStore(
    Directory rootDirectory, {
    List<FxLegacyPathStrategy> legacyPathStrategies =
        const <FxLegacyPathStrategy>[FxIosSandboxPathStrategy()],
  })  : _rootDirectory = rootDirectory,
        _legacyPathStrategies = legacyPathStrategies;

  @override
  String get rootPath => _rootDirectory.path;

  @override
  FxResourceRef referenceForPath(String path) {
    final FxResourceRef parsedReference = FxResourceRef.parse(path);
    if (parsedReference is FxManagedResourceRef ||
        parsedReference is FxNetworkResourceRef) {
      return parsedReference;
    }
    final String normalizedPath =
        _normalizeSystemPath(parsedReference.rawValue);
    final String normalizedRoot = _normalizeSystemPath(rootPath);
    if (normalizedPath == normalizedRoot ||
        !normalizedPath.startsWith('$normalizedRoot/')) {
      return parsedReference;
    }
    final String relativePath =
        normalizedPath.substring(normalizedRoot.length + 1);
    return FxManagedResourceRef(relativePath);
  }

  @override
  Future<FxManagedResourceRef> importFile(String sourcePath) async {
    try {
      final File sourceFile = File(sourcePath);
      if (!await sourceFile.exists()) {
        throw FxResourceException(
          FxResourceCode.storageFailed,
          'Local resource does not exist',
          FileSystemException('Source file not found', sourcePath),
          StackTrace.current,
          sourcePath,
        );
      }
      final String relativePath = _newImportRelativePath(sourcePath);
      final File targetFile = File(_joinRelativePath(relativePath));
      await targetFile.parent.create(recursive: true);
      await sourceFile.copy(targetFile.path);
      return FxManagedResourceRef(relativePath);
    } on FxResourceException {
      rethrow;
    } catch (error, stackTrace) {
      throw FxResourceException(
        FxResourceCode.storageFailed,
        'Failed to import local resource',
        error,
        stackTrace,
        sourcePath,
      );
    }
  }

  @override
  String resolvePath(String reference) {
    try {
      final FxResourceRef parsedReference = FxResourceRef.parse(reference);
      if (parsedReference is FxExternalResourceRef) {
        return parsedReference.path;
      }
      if (parsedReference is! FxManagedResourceRef) {
        final String? legacyRelativePath = _resolveLegacyRelativePath(
          parsedReference.rawValue,
        );
        if (legacyRelativePath != null) {
          final String migratedPath = _joinRelativePath(legacyRelativePath);
          if (File(migratedPath).existsSync()) {
            return migratedPath;
          }
        }
        return parsedReference.rawValue;
      }
      return _joinRelativePath(parsedReference.relativePath);
    } on FxResourceException {
      rethrow;
    } catch (error, stackTrace) {
      throw FxResourceException(
        FxResourceCode.storageFailed,
        'Failed to resolve resource reference',
        error,
        stackTrace,
        reference,
      );
    }
  }

  @override
  bool exists(String reference) {
    try {
      final FxResourceRef parsedReference = FxResourceRef.parse(reference);
      if (parsedReference is FxNetworkResourceRef) {
        return false;
      }
      return File(resolvePath(reference)).existsSync();
    } on FxResourceException {
      rethrow;
    } catch (error, stackTrace) {
      throw FxResourceException(
        FxResourceCode.storageFailed,
        'Failed to check resource existence',
        error,
        stackTrace,
        reference,
      );
    }
  }

  String _normalizeSystemPath(String value) {
    return value.replaceAll('\\', '/').replaceAll(RegExp('/+'), '/');
  }

  String _joinRelativePath(String relativePath) {
    final String systemRelativePath = relativePath.replaceAll(
      '/',
      Platform.pathSeparator,
    );
    return '${_rootDirectory.path}${Platform.pathSeparator}$systemRelativePath';
  }

  /// 生成应用托管资源目录下的唯一导入路径，并保留原文件扩展名。
  String _newImportRelativePath(String sourcePath) {
    final String fileName = sourcePath.split(Platform.pathSeparator).last;
    final int dotIndex = fileName.lastIndexOf('.');
    final String extension = dotIndex > 0 ? fileName.substring(dotIndex) : '';
    final int timestamp = DateTime.now().microsecondsSinceEpoch;
    final int sequence = _nextImportSequence++;
    return 'imports/$timestamp-$sequence$extension';
  }

  /// 为旧绝对路径寻找当前应用内可恢复的相对路径。
  String? _resolveLegacyRelativePath(String path) {
    for (final FxLegacyPathStrategy strategy in _legacyPathStrategies) {
      final String? relativePath = strategy.relativePathOf(path);
      if (relativePath != null) {
        return relativePath;
      }
    }
    return null;
  }
}
