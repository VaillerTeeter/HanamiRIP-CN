# Vue 注释生成指令（Copilot）

## 目标
为指定的 Vue 组件代码添加“傻子都能看懂”的详细中文注释。注释要覆盖业务意图、组件职责、响应式数据、生命周期、事件流、边界条件、错误处理与性能注意点。注释写在代码附近（行内/块注释/文档注释均可），不改变行为。

## 代码范围
- 需要注释的文件列表：
  - "./apps/desktop/frontend/App.vue"
  - "./apps/desktop/frontend/modules/download/pages/DownloadPage.vue"
  - "./apps/desktop/frontend/modules/query/components/StaffModal.vue"
  - "./apps/desktop/frontend/modules/query/pages/QueryPage.vue"
  - "./apps/desktop/frontend/modules/search/components/AliasModal.vue"
  - "./apps/desktop/frontend/modules/search/pages/SearchPage.vue"
  - "./apps/desktop/frontend/modules/tracking/pages/BacklogPage.vue"
  - "./apps/desktop/frontend/modules/tracking/pages/FinishedPage.vue"
  - "./apps/desktop/frontend/modules/tracking/pages/WatchingPage.vue"
  - "./apps/desktop/frontend/modules/tracks/components/MixQueueDetailModal.vue"
  - "./apps/desktop/frontend/modules/tracks/pages/TracksPage.vue"
  - "./apps/desktop/frontend/shared/components/AppTitlebar.vue"

## 输出要求（必须遵守）
1. **极度清晰**：对每个关键步骤、变量、分支、循环、返回值都要解释“为什么这样做”。
2. **从零开始**：假设读者完全不了解 Vue，必要时解释响应式、computed、watch、ref、reactive、生命周期、模板语法、指令与组件通信等概念。
3. **覆盖全面**：至少覆盖以下内容（按需选用，但不得遗漏适用项）：
   - 组件/文件用途与整体职责
   - props/emit 的含义与约束
   - 响应式状态的含义与来源
   - computed/watch 的触发条件与用途
   - 关键事件处理流程（点击、输入、提交等）
   - 异步请求与错误处理
   - 性能与副作用注意点（如频繁渲染、深度 watch）
   - 与外部模块/接口的依赖
4. **不改功能**：只加注释，不改逻辑、不重构、不改格式风格（除非注释必须换行）。
5. **注释风格**：
   - 组件说明、props、emit 使用文档注释 `/** ... */`
   - 复杂逻辑使用块注释 `/* ... */`
   - 行内细节使用 `// ...`
6. **术语统一**：同一概念使用同一中文词汇，不要同义反复。
7. **示例优先**：复杂逻辑可用小例子帮助理解（写在注释中，不改代码）。

## 注释粒度示例（仅作为风格参考）
- 对组件：说明“负责什么 UI/业务”。
- 对数据：说明“代表什么”“为何用响应式”。
- 对方法：说明“做什么”“何时触发”“可能失败的原因”。

## 禁止事项
- 不要删除现有注释
- 不要引入第三方库
- 不要生成与代码无关的内容

## 交付检查清单
- [ ] 关键 props/emit 均有说明
- [ ] 响应式数据与 computed/watch 已解释
- [ ] 异步流程与错误路径已说明
- [ ] 性能注意点已覆盖

## 备注
- 已补充文件列表，如需调整请直接修改上方列表。
- 最后更新：2026-02-06（初始第一版）。
