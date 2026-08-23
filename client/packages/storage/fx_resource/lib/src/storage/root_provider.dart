import 'dart:io';

/// 为资源存储提供根目录，宿主可按平台、账号或工作区自由实现。
abstract interface class FxResourceRootProvider {
  /// 异步获取当前资源存储应使用的根目录。
  Future<Directory> loadRootDirectory();
}
