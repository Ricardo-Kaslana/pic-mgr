# ADR-0002: SQLite 存储元数据

**日期:** 2026-06-07
**状态:** 已接受

## 背景

图片的标签、备注、评分等元数据需要持久化存储。

## 决策

使用 SQLite (rusqlite + bundled)，所有元数据只写数据库，不修改原始图片文件。

## 理由

- 单文件数据库，零配置
- FTS5 支持全文搜索
- WAL 模式支持并发读写
- `bundled` feature 自带编译好的 SQLite，无需系统依赖

## 后果

- 数据库文件在应用数据目录，与工作区分离
- 工作区可自由移动/重命名（使用相对路径）
- 数据库损坏风险低，但需定期备份
- **标签模型（全局）** — `tags` 表**没有** `workspace_id` 列。整个数据库里只有一张全局标签表，所有工作区共用。决定一个标签**可见性**的是 `image_tags` 成员关系，不是标签定义本身。后端只暴露两条查询路径：`get_tags(workspace_id)`（带 `image_count`，按工作区过滤，驱动侧栏 chip）和 `get_all_tags()`（无计数，无工作区过滤，驱动 MetadataEditor 的建议下拉和去重）。**跨工作区复用是显式设计目标**（在 A 工作区创建的标签可直接挂到 B 工作区的图，无需重新创建）。**标签删除是全局的** —— `image_tags.tag_id` FK 是 `ON DELETE CASCADE`，删除一个标签会一次性清掉所有工作区的 `image_tags` 引用。**孤儿标签**（零 `image_tags` 引用的行）`get_tags` 看不到，但会阻挡同名 `create_tag`，只能由 `delete_workspace` 的第 4 步扫除。详见 AGENTS.md "Tag model (global, not workspace-scoped)" 与 `db/queries.rs` 的 5 个 tag 模型相关单元测试。

---

## 后续修订

- **2026-07-12**: 补写"标签模型（全局）"后果条目，记录跨工作区复用、全局删除、孤儿清理三项决策。原 ADR 仅描述了存储引擎选择，本身没明说全局标签设计；随 v0.3.16 的 `get_all_tags`/`create_tag trim`/右键删除功能上线后，把这条隐含规则从代码分散注释里集中到 ADR 防止以后再被误读。
