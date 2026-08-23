# fx_resource

`fx_resource` 负责 Flutter 应用内的本地资源引用、选择和旧路径迁移。

- `resources:relative/path`：应用管理的稳定本地资源引用。
- `resources:/absolute/path`：外部本地资源引用；本包原样保留。
- `http(s)`：网络地址；本包不下载、不上传。

宿主通过 `FxResourceRootProvider` 决定资源根目录；默认可使用
`FxApplicationDocumentsRootProvider`，也可实现自己的工作区、账号隔离或测试目录。
取得根目录后创建 `FxFileResourceStore` 与 `FxFileResourcePicker`，再通过
`FxResourceScope` 注入组件树。`FxFileResourcePicker` 会在用户选择当次把文件
导入资源根目录的 `imports/`，并返回 `resources:imports/...`；这避免 macOS
沙盒在下次启动后失去对外部文件的读取授权。业务模型仅保存
`FxResourceRef.rawValue`。

同步只传递引用字符串；本包不会传输、替换或删除资源文件。

## 异常

本包接入 `fx_exception` 的统一协议。可捕获 `FxResourceException`，并通过
`FxResourceCode` 区分根目录不可用、引用格式错误、受管理路径不安全、文件选择失败、迁移失败、文件访问失败和资源作用域缺失等边界错误。异常会保留原始
`error`、`stack` 与可用时的 `reference`。旧路径迁移遵循“尽力而为”原则：空引用或不符合当前协议的历史引用会保留原值，不会阻断同步文件的本地物化；未知的迁移策略异常仍会包装为 `migrationFailed`。

运行时错误信息统一使用英文，便于日志检索、跨端展示和宿主自行国际化；下表给出对应的中英文含义。宿主应优先按稳定的 `FxResourceCode` 分支处理，而不是匹配 `message` 文本。

| 错误码 | 数值 | English runtime message | 中文含义 |
| --- | ---: | --- | --- |
| `rootUnavailable` | 1001 | `Unable to access application resource root` | 无法获取或创建应用资源根目录。 |
| `invalidReference` | 1002 | `Resource reference must not be empty` | 资源引用为空或格式无法解析。 |
| `unsafeRelativePath` | 1003 | `Managed resource must use a safe relative path` | 受管理资源路径为空或包含 `..` 等不安全片段。 |
| `pickFailed` | 1004 | `Failed to pick resource` | 平台文件选择器调用失败。 |
| `scopeUnavailable` | 1005 | `FxResourceScope is not available in the widget tree` | 组件树没有注入 `FxResourceScope`。 |
| `migrationFailed` | 1006 | `Failed to migrate legacy resource reference` | 历史路径迁移过程发生未知错误。 |
| `storageFailed` | 1007 | `Failed to resolve resource reference` / `Failed to check resource existence` | 路径解析或本地文件存在性检查失败。 |
