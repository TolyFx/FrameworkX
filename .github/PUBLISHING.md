# Flutter 包发布

FrameworkX 的 `fx_*` 包通过 GitHub Actions 发布到 pub.dev。

## 环境配置

- GitHub Environment：`pub.dev`
- 认证方式：pub.dev Trusted Publishing（GitHub OIDC）
- Workflow 权限：`contents: read`、`id-token: write`
- 不在仓库或本机项目配置中保存 pub.dev 凭据

pub.dev 后台的 Automated Publishing 必须绑定：

- Repository：`TolyFx/FrameworkX`
- Workflow：`publish-package.yml`
- Environment：`pub.dev`

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
