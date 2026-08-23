# Dart / Flutter SDK Packages

此目录存放可以被多个 Flutter 应用复用的独立 package，并按领域而不是技术类型归档。

```text
packages/
├── foundation/
│   ├── fx_exception/
│   └── fx_dio/
├── identity/
│   ├── fx_user_core/
│   ├── fx_user_session/
│   ├── adapters/
│   │   └── fx_user_http/
│   └── ui/
│       ├── fx_user_ui/
│       └── fx_account/
└── storage/
    ├── fx_resource/
    ├── fx_storage_core/
    ├── fx_storage_queue/
    └── adapters/
        ├── fx_storage_http/
        └── fx_storage_sqflite/
```

规则：

- 领域目录只负责归类，不是可发布 package。
- package 不得依赖 ViewX、`fn_*` 业务模块、宿主路由或宿主状态容器。
- core package 保持纯 Dart；Flutter、Dio、SQLite 等技术依赖留在 UI 或 adapter。
- 跨包只使用 `lib/{package_name}.dart` 公开出口，不 import 其他包的 `src/`。
- 本地开发由根 Pub Workspace 解析，禁止提交指向其他仓库的 `dependency_overrides`。
- 尚未迁移的 package 不提前创建空目录。

