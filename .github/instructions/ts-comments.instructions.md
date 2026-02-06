# TypeScript 注释生成指令（Copilot）

## 目标
为指定的 TypeScript 代码添加“傻子都能看懂”的详细中文注释。注释要覆盖业务意图、关键逻辑、类型约束、边界条件、错误处理与性能注意点。注释写在代码附近（行内/块注释/文档注释均可），不改变行为。

## 代码范围
- 需要注释的文件列表：
   - "./apps/desktop/frontend/env.d.ts"
   - "./apps/desktop/frontend/main.ts"
   - "./apps/desktop/frontend/modules/download/composables/useDownloadPage.ts"
   - "./apps/desktop/frontend/modules/download/types/download.ts"
   - "./apps/desktop/frontend/modules/query/composables/useQueryPage.ts"
   - "./apps/desktop/frontend/modules/search/composables/useSearchPage.ts"
   - "./apps/desktop/frontend/modules/search/types/search.ts"
   - "./apps/desktop/frontend/modules/tracking/composables/useTracking.ts"
   - "./apps/desktop/frontend/modules/tracking/types/anime.ts"
   - "./apps/desktop/frontend/modules/tracking/types/tracking.ts"
   - "./apps/desktop/frontend/modules/tracks/composables/useTracksPage.ts"
   - "./apps/desktop/frontend/modules/tracks/types/tracks.ts"
   - "./apps/desktop/frontend/shared/composables/useExternalLink.ts"
   - "./apps/desktop/frontend/shared/composables/useWindowControls.ts"
   - "./apps/desktop/frontend/shared/types/page.ts"
   - "./apps/desktop/frontend/shared/utils/format.ts"
   - "./apps/desktop/frontend/shared/utils/tauri.ts"
   - "./apps/desktop/vite.config.ts"

## 输出要求（必须遵守）
1. **极度清晰**：对每个关键步骤、变量、分支、循环、返回值都要解释“为什么这样做”。
2. **从零开始**：假设读者完全不了解 TS/JS，必要时解释类型、接口、泛型、Promise、async/await、模块、装饰器等概念。
3. **覆盖全面**：至少覆盖以下内容（按需选用，但不得遗漏适用项）：
   - 文件/模块用途
   - 类型/接口/枚举的设计目的与字段语义
   - 函数/方法的输入、输出、边界条件
   - 关键分支的意图与触发条件
   - 错误处理与异常情况
   - 异步流程、并发与副作用说明
   - 性能与内存相关注意点
   - 与外部接口/环境的依赖（API、存储、DOM、文件等）
4. **不改功能**：只加注释，不改逻辑、不重构、不改格式风格（除非注释必须换行）。
5. **注释风格**：
   - 模块/类型/公共函数使用文档注释 `/** ... */`
   - 复杂逻辑使用块注释 `/* ... */`
   - 行内细节使用 `// ...`
6. **术语统一**：同一概念使用同一中文词汇，不要同义反复。
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
- [ ] 重要的 TS 概念已讲清
- [ ] 异步/副作用已说明

## 备注
- 已补充文件列表，如需调整请直接修改上方列表。
- 最后更新：2026-02-06（初始第一版）。
