import 'package:file_picker/file_picker.dart';

import '../domain/exception.dart';
import '../domain/reference.dart';
import '../storage/store.dart';
import 'picker.dart';

/// 使用 file_picker 的通用文件选择实现。
final class FxFileResourcePicker implements FxResourcePicker {
  /// 负责托管或转换选中路径的资源存储。
  final FxResourceStore _store;

  /// 创建文件选择器。
  const FxFileResourcePicker(this._store);

  @override
  Future<List<FxResourceRef>> pickImages() {
    return _pick(FileType.image);
  }

  @override
  Future<List<FxResourceRef>> pickFiles() {
    return _pick(FileType.any);
  }

  /// 选择文件后优先导入资源目录；取消选择时返回空列表。
  Future<List<FxResourceRef>> _pick(FileType type) async {
    try {
      final FilePickerResult? result = await FilePicker.platform.pickFiles(
        type: type,
        allowMultiple: true,
      );
      if (result == null) {
        return <FxResourceRef>[];
      }
      final List<FxResourceRef> references = <FxResourceRef>[];
      for (final PlatformFile file in result.files) {
        final String? path = file.path;
        if (path != null && path.isNotEmpty) {
          references.add(await _toReference(path));
        }
      }
      return references;
    } on FxResourceException {
      rethrow;
    } catch (error, stackTrace) {
      throw FxResourceException(
        FxResourceCode.pickFailed,
        'Failed to pick resource',
        error,
        stackTrace,
      );
    }
  }

  /// 优先复制到应用资源目录，保证重启后的本地文件仍可读取。
  Future<FxResourceRef> _toReference(String path) async {
    final FxResourceStore store = _store;
    if (store is FxResourceImportStore) {
      final FxResourceImportStore importStore = store as FxResourceImportStore;
      return importStore.importFile(path);
    }
    return store.referenceForPath(path);
  }
}
