/*
	服务模块入口：
	在这里统一导出各个功能模块，供 main.rs 注册命令时使用。
*/

// Bangumi 相关功能（条目、季节、翻译等）。
pub mod bangumi;
// 打开外部链接的能力。
pub mod external;
// 媒体轨道解析与混流。
pub mod media;
// 本地追番数据存储。
pub mod storage;
// 种子下载相关功能。
pub mod torrent;
