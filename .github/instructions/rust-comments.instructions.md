# Rust 注释生成指令（Copilot）

## 目标
为指定的 Rust 代码添加“傻子都能看懂”的详细中文注释。注释要覆盖业务意图、数据结构含义、边界条件、错误处理、并发/性能注意点、unsafe 原因与安全性说明。注释写在代码附近（行内/块注释/文档注释均可），不改变行为。

## 代码范围
- 需要注释的文件列表：
   - "./apps/desktop/backend/build.rs"
   - "./apps/desktop/backend/src/main.rs"
   - "./apps/desktop/backend/src/services/bangumi/api.rs"
   - "./apps/desktop/backend/src/services/bangumi/commands.rs"
   - "./apps/desktop/backend/src/services/bangumi/filters.rs"
   - "./apps/desktop/backend/src/services/bangumi/mod.rs"
   - "./apps/desktop/backend/src/services/bangumi/models.rs"
   - "./apps/desktop/backend/src/services/bangumi/translate.rs"
   - "./apps/desktop/backend/src/services/external/mod.rs"
   - "./apps/desktop/backend/src/services/media/mod.rs"
   - "./apps/desktop/backend/src/services/mod.rs"
   - "./apps/desktop/backend/src/services/storage/mod.rs"
   - "./apps/desktop/backend/src/services/torrent/mod.rs"
   - "./crates/baidu_verify/src/lib.rs"

## 输出要求（必须遵守）
1. **极度清晰**：对每个关键步骤、变量、分支、循环、返回值都要解释“为什么这样做”。
2. **从零开始**：假设读者完全不了解 Rust，必要时解释 Rust 概念（所有权、借用、生命周期、迭代器、match、Result、Option、trait、泛型、宏等）。
3. **覆盖全面**：至少覆盖以下内容（按需选用，但不得遗漏适用项）：
   - 模块/文件级用途
   - 结构体/枚举/trait 的设计目的与字段语义
   - 函数/方法的输入、输出、边界条件
   - 每个分支的意图和“为什么要这么分支”
   - 错误处理路径、错误来源、恢复策略
   - I/O 或网络交互的前置条件/后置保证
   - 并发安全、锁、线程、异步的关键点
   - 性能考虑（如拷贝、分配、迭代器/集合选择）
   - `unsafe` 的理由、保障不出错的前提
4. **不改功能**：只加注释，不改逻辑、不重构、不改格式风格（除非注释必须换行）。
5. **注释风格**：
   - 业务含义/整体说明用块注释 `/* ... */`
   - 对外 API 或重要结构体用文档注释 `/// ...`
   - 行内细节用 `// ...`
6. **术语统一**：同一概念使用同一中文词汇（如“所有权”“借用”“生命周期”），不要同义反复。
7. **示例优先**：复杂逻辑可用小例子帮助理解（写在注释中，不改代码）。

## 注释粒度示例（仅作为风格参考）
- 对函数：说明“做什么”“输入是什么”“输出是什么”“可能失败的原因”。
- 对变量：说明“代表什么”“为何选此类型”。
- 对分支：说明“什么时候进来”“为什么要特殊处理”。

## 禁止事项
- 不要删除现有注释
- 不要引入第三方库
- 不要生成与代码无关的内容

## 交付检查清单
- [ ] 关键类型/函数均有说明
- [ ] 错误路径有解释
- [ ] 重要的 Rust 概念已讲清
- [ ] `unsafe` 有安全前提说明（如存在）

## 备注
- 已补充文件列表，如需调整请直接修改上方列表。
- 最后更新：2026-02-06（初始第一版）。
