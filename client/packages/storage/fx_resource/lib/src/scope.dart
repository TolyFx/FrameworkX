import 'package:flutter/widgets.dart';

import 'domain/exception.dart';
import 'picker/picker.dart';
import 'storage/store.dart';

/// 向 Flutter 组件树注入资源存储与选择器，避免依赖全局静态状态。
class FxResourceScope extends InheritedWidget {
  /// 当前应用的资源存储。
  final FxResourceStore store;

  /// 当前应用的资源选择器。
  final FxResourcePicker picker;

  /// 创建资源依赖作用域。
  const FxResourceScope({
    super.key,
    required this.store,
    required this.picker,
    required super.child,
  });

  /// 从组件树获取资源依赖，缺失时直接抛出明确错误。
  static FxResourceScope of(BuildContext context) {
    final FxResourceScope? scope =
        context.dependOnInheritedWidgetOfExactType<FxResourceScope>();
    if (scope == null) {
      throw FxResourceException(
        FxResourceCode.scopeUnavailable,
        'FxResourceScope is not available in the widget tree',
        null,
        StackTrace.current,
      );
    }
    return scope;
  }

  @override
  bool updateShouldNotify(FxResourceScope oldWidget) {
    return !identical(store, oldWidget.store) ||
        !identical(picker, oldWidget.picker);
  }
}
