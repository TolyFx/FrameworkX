import 'dart:io';

import 'package:path_provider/path_provider.dart';

import '../domain/exception.dart';
import 'root_provider.dart';

/// 使用 Flutter 应用 Documents 目录作为资源根目录的默认实现。
final class FxApplicationDocumentsRootProvider
    implements FxResourceRootProvider {
  /// 创建默认 Documents 根目录提供者。
  const FxApplicationDocumentsRootProvider();

  @override
  Future<Directory> loadRootDirectory() async {
    try {
      final Directory documentsDirectory =
          await getApplicationDocumentsDirectory();
      if (!await documentsDirectory.exists()) {
        await documentsDirectory.create(recursive: true);
      }
      return documentsDirectory;
    } on FxResourceException {
      rethrow;
    } catch (error, stackTrace) {
      throw FxResourceException(
        FxResourceCode.rootUnavailable,
        'Unable to access application resource root',
        error,
        stackTrace,
      );
    }
  }
}
