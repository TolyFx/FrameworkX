# FrameworkX 数据库迁移

`postgres/` 是 FrameworkX 服务端 PostgreSQL 迁移的唯一公开入口。

迁移按文件名前缀顺序执行；已执行文件不可修改，只能追加新迁移。SDK crate 持有模型与
Adapter，根迁移目录负责聚合 SDK 内部建表顺序，消费方不依赖 crate 内部目录结构。

FrameworkX 与消费方各自维护独立编号空间，均可从 `001` 开始。宿主迁移工具必须用
`source + filename` 记录执行历史，不能要求产品迁移顺延 SDK 的内部编号。
