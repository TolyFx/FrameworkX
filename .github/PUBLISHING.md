# Flutter 包发布

FrameworkX 的 `fx_*` 包通过 GitHub Actions 发布到 pub.dev。

## 环境配置

- 认证方式：GitHub Actions Secret `PUB_CREDENTIALS_B64`
- 本地凭据备份：`/Volumes/Toly1T/File/config/env/PUB_CREDENTIALS_B64`
- Workflow 权限：`contents: read`
- 凭据只保存在本机私有目录和 GitHub Secret 中，不提交到仓库

配置或更新仓库 Secret：

```bash
gh secret set PUB_CREDENTIALS_B64 \
  --repo TolyFx/FrameworkX \
  < /Volumes/Toly1T/File/config/env/PUB_CREDENTIALS_B64
```

本地文件权限应保持为 `600`。工作流运行时会临时还原
`~/.config/dart/pub-credentials.json`，发布结束后立即删除。

## 发布标签

单包发布：

```text
fx_account-v0.1.1
```

批量发布：

```text
publish/fx_account@0.1.1/fx_user_ui@0.0.2
```

标签版本必须与目标包 `pubspec.yaml` 中的版本一致。工作流会依次校验必要文件、执行测试、
检查发布归档并发布到 pub.dev。

首次配置或修复工作流后，可以从 Actions 手动运行，并在 `release_tag` 中填写同样的标签。
