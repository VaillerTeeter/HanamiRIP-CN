/* 文件：mod.rs | 用途：汇总暴露后端服务模块 | 关键对象：bangumi、external、media、storage、torrent */
// 本行目的：公开 bangumi 服务模块。
pub mod bangumi;
// 本行目的：公开外部链接服务模块。
pub mod external;
// 本行目的：公开媒体处理服务模块。
pub mod media;
// 本行目的：公开本地存储服务模块。
pub mod storage;
// 本行目的：公开种子下载服务模块。
pub mod torrent;
