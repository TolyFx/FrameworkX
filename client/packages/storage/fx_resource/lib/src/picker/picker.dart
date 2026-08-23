import '../domain/reference.dart';

/// 面向业务的资源选择抽象，避免业务层绑定具体插件。
abstract interface class FxResourcePicker {
  /// 选择一项或多项图片资源。
  Future<List<FxResourceRef>> pickImages();

  /// 选择一项或多项任意文件资源。
  Future<List<FxResourceRef>> pickFiles();
}
