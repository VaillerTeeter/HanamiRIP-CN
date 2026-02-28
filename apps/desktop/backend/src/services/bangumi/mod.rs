/* 文件：mod.rs | 用途：声明 bangumi 子模块并控制可见性 | 关键对象：api、commands、filters、models、translate */
// 本行目的：声明内部 API 实现模块。
mod api;
// 本行目的：公开命令模块供外部调用。
pub mod commands;
// 本行目的：声明过滤与解析辅助模块。
mod filters;
// 本行目的：声明数据模型模块。
mod models;
// 本行目的：声明翻译辅助模块。
mod translate;
