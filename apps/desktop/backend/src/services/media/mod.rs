/* 文件：mod.rs | 用途：解析媒体轨道并合成输出文件 | 关键对象：parse_media_tracks、mix_media_tracks */
// 本行目的：引入反序列化支持。
use serde::Deserialize;
// 本行目的：引入序列化支持。
use serde::Serialize;
// 本行目的：引入哈希映射类型。
use std::collections::HashMap;
// 本行目的：引入文件系统模块。
use std::fs;
// 本行目的：引入路径类型。
use std::path::Path;
// 本行目的：引入路径缓冲类型。
use std::path::PathBuf;
// 本行目的：引入 Tauri 管理器 trait。
use tauri::Manager;
// 本行目的：引入异步进程命令类型。
use tokio::process::Command;

// 本行目的：为轨道信息结构启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义轨道信息响应结构体。
pub struct TrackInfoResponse {
    // 变量：track_id | 含义：轨道 ID | 类型：String | 作用域：TrackInfoResponse
    // 本行目的：记录轨道标识。
    pub track_id: String,
    // 变量：codec | 含义：编码名称 | 类型：String | 作用域：TrackInfoResponse
    // 本行目的：记录轨道编码。
    pub codec: String,
    // 变量：lang | 含义：语言代码 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录语言代码。
    pub lang: Option<String>,
    // 变量：language_name | 含义：语言名称 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录语言名称。
    pub language_name: Option<String>,
    // 变量：track_name | 含义：轨道名称 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录轨道名称。
    pub track_name: Option<String>,
    // 变量：is_default | 含义：是否默认轨道 | 类型：Option<bool> | 作用域：TrackInfoResponse
    // 本行目的：记录是否为默认轨道。
    pub is_default: Option<bool>,
    // 变量：is_forced | 含义：是否强制轨道 | 类型：Option<bool> | 作用域：TrackInfoResponse
    // 本行目的：记录是否为强制轨道。
    pub is_forced: Option<bool>,
    // 变量：charset | 含义：字符集 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录字符集。
    pub charset: Option<String>,
    // 变量：attributes | 含义：轨道附加属性 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录轨道附加属性。
    pub attributes: Option<String>,
    // 变量：container | 含义：容器类型 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录容器类型。
    pub container: Option<String>,
    // 变量：file_size | 含义：文件大小 | 类型：Option<String> | 作用域：TrackInfoResponse
    // 本行目的：记录文件大小。
    pub file_size: Option<String>,
// 本行目的：结束轨道信息结构体定义。
}

// 本行目的：为轨道解析响应启用序列化。
#[derive(Serialize)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义轨道解析响应结构体。
pub struct TrackParseResponse {
    // 变量：tracks | 含义：轨道列表 | 类型：Vec<TrackInfoResponse> | 作用域：TrackParseResponse
    // 本行目的：记录轨道列表。
    pub tracks: Vec<TrackInfoResponse>,
// 本行目的：结束轨道解析响应结构体定义。
}

// 本行目的：为合成输入结构启用反序列化与克隆。
#[derive(Deserialize, Clone)]
// 本行目的：使用 camelCase 反序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义轨道合成输入结构体。
pub struct MixTrackInput {
    // 变量：path | 含义：轨道文件路径 | 类型：String | 作用域：MixTrackInput
    // 本行目的：记录轨道文件路径。
    pub path: String,
    // 变量：kind | 含义：轨道类型 | 类型：String | 作用域：MixTrackInput
    // 本行目的：记录轨道类型。
    pub kind: String,
    // 变量：track_ids | 含义：轨道 ID 列表 | 类型：Vec<String> | 作用域：MixTrackInput
    // 本行目的：记录轨道 ID 列表。
    pub track_ids: Vec<String>,

    // 本行目的：为缺失语言映射提供默认值。
    #[serde(default)]
    // 变量：track_langs | 含义：轨道语言映射 | 类型：HashMap<String, String> | 作用域：MixTrackInput
    // 本行目的：记录轨道语言映射。
    pub track_langs: HashMap<String, String>,
// 本行目的：结束轨道合成输入结构体定义。
}

// 本行目的：为 ffprobe 格式结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 ffprobe 格式结构体。
struct FFProbeFormat {
    // 变量：format_name | 含义：容器格式名称 | 类型：Option<String> | 作用域：FFProbeFormat
    // 本行目的：记录容器格式名称。
    format_name: Option<String>,
    // 变量：size | 含义：文件大小字符串 | 类型：Option<String> | 作用域：FFProbeFormat
    // 本行目的：记录文件大小字符串。
    size: Option<String>,
// 本行目的：结束 ffprobe 格式结构体定义。
}

// 本行目的：为 ffprobe disposition 结构启用反序列化与克隆。
#[derive(Clone, Deserialize)]
// 本行目的：定义 ffprobe 轨道标记结构体。
struct FFProbeDisposition {
    // 变量：default | 含义：默认标记 | 类型：Option<i32> | 作用域：FFProbeDisposition
    // 本行目的：记录默认标记。
    default: Option<i32>,
    // 变量：forced | 含义：强制标记 | 类型：Option<i32> | 作用域：FFProbeDisposition
    // 本行目的：记录强制标记。
    forced: Option<i32>,
// 本行目的：结束 ffprobe disposition 结构体定义。
}

// 本行目的：为 ffprobe 标签结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 ffprobe 标签结构体。
struct FFProbeStreamTags {
    // 变量：language | 含义：语言代码 | 类型：Option<String> | 作用域：FFProbeStreamTags
    // 本行目的：记录语言代码。
    language: Option<String>,
    // 变量：title | 含义：轨道标题 | 类型：Option<String> | 作用域：FFProbeStreamTags
    // 本行目的：记录轨道标题。
    title: Option<String>,
    // 变量：encoding | 含义：编码信息 | 类型：Option<String> | 作用域：FFProbeStreamTags
    // 本行目的：记录编码信息。
    encoding: Option<String>,
    // 变量：charset | 含义：字符集 | 类型：Option<String> | 作用域：FFProbeStreamTags
    // 本行目的：记录字符集。
    charset: Option<String>,
// 本行目的：结束 ffprobe 标签结构体定义。
}

// 本行目的：为 ffprobe 轨道结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 ffprobe 轨道结构体。
struct FFProbeStream {
    // 变量：index | 含义：轨道索引 | 类型：Option<u32> | 作用域：FFProbeStream
    // 本行目的：记录轨道索引。
    index: Option<u32>,
    // 变量：codec_name | 含义：编码名称 | 类型：Option<String> | 作用域：FFProbeStream
    // 本行目的：记录编码名称。
    codec_name: Option<String>,
    // 变量：codec_type | 含义：轨道类型 | 类型：Option<String> | 作用域：FFProbeStream
    // 本行目的：记录轨道类型。
    codec_type: Option<String>,
    // 变量：width | 含义：视频宽度 | 类型：Option<u32> | 作用域：FFProbeStream
    // 本行目的：记录视频宽度。
    width: Option<u32>,
    // 变量：height | 含义：视频高度 | 类型：Option<u32> | 作用域：FFProbeStream
    // 本行目的：记录视频高度。
    height: Option<u32>,
    // 变量：r_frame_rate | 含义：帧率 | 类型：Option<String> | 作用域：FFProbeStream
    // 本行目的：记录帧率字符串。
    r_frame_rate: Option<String>,
    // 变量：channels | 含义：声道数 | 类型：Option<u32> | 作用域：FFProbeStream
    // 本行目的：记录声道数。
    channels: Option<u32>,
    // 变量：channel_layout | 含义：声道布局 | 类型：Option<String> | 作用域：FFProbeStream
    // 本行目的：记录声道布局。
    channel_layout: Option<String>,
    // 变量：disposition | 含义：轨道标记 | 类型：Option<FFProbeDisposition> | 作用域：FFProbeStream
    // 本行目的：记录轨道标记。
    disposition: Option<FFProbeDisposition>,
    // 变量：tags | 含义：轨道标签 | 类型：Option<FFProbeStreamTags> | 作用域：FFProbeStream
    // 本行目的：记录轨道标签。
    tags: Option<FFProbeStreamTags>,
// 本行目的：结束 ffprobe 轨道结构体定义。
}

// 本行目的：为 ffprobe 输出结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 ffprobe 输出结构体。
struct FFProbeOutput {
    // 变量：streams | 含义：轨道列表 | 类型：Option<Vec<FFProbeStream>> | 作用域：FFProbeOutput
    // 本行目的：记录轨道列表。
    streams: Option<Vec<FFProbeStream>>,
    // 变量：format | 含义：格式信息 | 类型：Option<FFProbeFormat> | 作用域：FFProbeOutput
    // 本行目的：记录格式信息。
    format: Option<FFProbeFormat>,
// 本行目的：结束 ffprobe 输出结构体定义。
}

// 本行目的：为 mkvmerge 容器属性启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 mkvmerge 容器属性结构体。
struct MkvmergeContainerProperties {
    // 变量：file_size | 含义：文件大小 | 类型：Option<u64> | 作用域：MkvmergeContainerProperties
    // 本行目的：记录文件大小。
    file_size: Option<u64>,
// 本行目的：结束 mkvmerge 容器属性结构体定义。
}

// 本行目的：为 mkvmerge 容器结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 mkvmerge 容器结构体。
struct MkvmergeContainer {
    // 变量：r#type | 含义：容器类型 | 类型：Option<String> | 作用域：MkvmergeContainer
    // 本行目的：记录容器类型。
    r#type: Option<String>,
    // 变量：properties | 含义：容器属性 | 类型：Option<MkvmergeContainerProperties> | 作用域：MkvmergeContainer
    // 本行目的：记录容器属性。
    properties: Option<MkvmergeContainerProperties>,
// 本行目的：结束 mkvmerge 容器结构体定义。
}

// 本行目的：为 mkvmerge 轨道属性启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 mkvmerge 轨道属性结构体。
struct MkvmergeTrackProperties {
    // 变量：language | 含义：语言代码 | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录语言代码。
    language: Option<String>,
    // 变量：language_ietf | 含义：IETF 语言代码 | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录 IETF 语言代码。
    language_ietf: Option<String>,
    // 变量：track_name | 含义：轨道名称 | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录轨道名称。
    track_name: Option<String>,
    // 变量：default_track | 含义：默认轨道标记 | 类型：Option<bool> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录默认轨道标记。
    default_track: Option<bool>,
    // 变量：forced_track | 含义：强制轨道标记 | 类型：Option<bool> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录强制轨道标记。
    forced_track: Option<bool>,
    // 变量：codec_name | 含义：编码名称 | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录编码名称。
    codec_name: Option<String>,
    // 变量：codec_id | 含义：编码 ID | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录编码 ID。
    codec_id: Option<String>,
    // 变量：encoding | 含义：编码或字符集 | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录编码信息。
    encoding: Option<String>,
    // 变量：pixel_dimensions | 含义：分辨率 | 类型：Option<String> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录分辨率。
    pixel_dimensions: Option<String>,
    // 变量：audio_channels | 含义：声道数 | 类型：Option<u32> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录声道数。
    audio_channels: Option<u32>,
    // 变量：audio_sampling_frequency | 含义：采样率 | 类型：Option<f64> | 作用域：MkvmergeTrackProperties
    // 本行目的：记录采样率。
    audio_sampling_frequency: Option<f64>,
// 本行目的：结束 mkvmerge 轨道属性结构体定义。
}

// 本行目的：为 mkvmerge 轨道结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 mkvmerge 轨道结构体。
struct MkvmergeTrack {
    // 变量：id | 含义：轨道 ID | 类型：u32 | 作用域：MkvmergeTrack
    // 本行目的：记录轨道 ID。
    id: u32,
    // 变量：r#type | 含义：轨道类型 | 类型：Option<String> | 作用域：MkvmergeTrack
    // 本行目的：记录轨道类型。
    r#type: Option<String>,
    // 变量：codec | 含义：编码名称 | 类型：Option<String> | 作用域：MkvmergeTrack
    // 本行目的：记录编码名称。
    codec: Option<String>,
    // 变量：properties | 含义：轨道属性 | 类型：Option<MkvmergeTrackProperties> | 作用域：MkvmergeTrack
    // 本行目的：记录轨道属性。
    properties: Option<MkvmergeTrackProperties>,
// 本行目的：结束 mkvmerge 轨道结构体定义。
}

// 本行目的：为 mkvmerge 输出结构启用反序列化。
#[derive(Deserialize)]
// 本行目的：定义 mkvmerge 输出结构体。
struct MkvmergeOutput {
    // 变量：container | 含义：容器信息 | 类型：Option<MkvmergeContainer> | 作用域：MkvmergeOutput
    // 本行目的：记录容器信息。
    container: Option<MkvmergeContainer>,
    // 变量：tracks | 含义：轨道列表 | 类型：Option<Vec<MkvmergeTrack>> | 作用域：MkvmergeOutput
    // 本行目的：记录轨道列表。
    tracks: Option<Vec<MkvmergeTrack>>,
// 本行目的：结束 mkvmerge 输出结构体定义。
}

/// 函数：format_bytes_readable | 输入：字节数 | 输出：可读格式字符串 | 可能失败：无
// 本行目的：将字节数转换为人类可读格式。
fn format_bytes_readable(bytes: u64) -> String {
    // 变量：UNITS | 含义：单位列表 | 类型：[&str; 5] | 作用域：format_bytes_readable
    // 本行目的：定义单位列表。
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    // 变量：size | 含义：当前大小浮点值 | 类型：f64 | 作用域：format_bytes_readable
    // 本行目的：初始化大小为字节数。
    let mut size = bytes as f64;
    // 变量：idx | 含义：当前单位索引 | 类型：usize | 作用域：format_bytes_readable
    // 本行目的：初始化单位索引。
    let mut idx = 0;

    // 本行目的：循环缩放到合适单位。
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        // 本行目的：转换为更高单位。
        size /= 1024.0;
        // 本行目的：单位索引加一。
        idx += 1;
    // 本行目的：结束单位转换循环。
    }

    // 本行目的：根据单位选择格式化输出。
    if idx == 0 {
        // 本行目的：无单位转换时直接输出字节数。
        format!("{} {}", bytes, UNITS[idx])
    // 本行目的：处理有单位转换的情况。
    } else {
        // 本行目的：输出保留两位小数的结果。
        format!("{:.2} {}", size, UNITS[idx])
    // 本行目的：结束格式化分支。
    }
// 本行目的：结束字节格式化函数。
}

/// 函数：resolve_tool_path | 输入：App 句柄与工具名 | 输出：工具路径 | 可能失败：找不到工具文件
// 本行目的：解析内置工具的实际路径。
fn resolve_tool_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    // 变量：resource_dir | 含义：资源目录路径 | 类型：PathBuf | 作用域：resolve_tool_path
    // 本行目的：获取资源目录。
    let resource_dir = app.path().resource_dir().map_err(|e| format!("无法获取资源目录: {e}"))?;
    // 变量：candidates | 含义：候选路径列表 | 类型：Vec<PathBuf> | 作用域：resolve_tool_path
    // 本行目的：初始化候选路径列表。
    let mut candidates = Vec::new();

    // 本行目的：在 Windows 下追加 .exe 工具路径。
    if cfg!(target_os = "windows") {
        // 本行目的：追加 Windows 工具路径。
        candidates.push(resource_dir.join("bin").join(format!("{name}.exe")));
    // 本行目的：结束 Windows 判断。
    }

    // 本行目的：追加通用工具路径。
    candidates.push(resource_dir.join("bin").join(name));

    // 本行目的：调试模式下追加开发工具路径。
    if cfg!(debug_assertions) {
        // 变量：dev_bin | 含义：开发环境工具目录 | 类型：PathBuf | 作用域：resolve_tool_path
        // 本行目的：构建开发工具目录路径。
        let dev_bin = PathBuf::from("../public").join("tools");

        // 本行目的：在 Windows 下追加 .exe 开发工具路径。
        if cfg!(target_os = "windows") {
            // 本行目的：追加 Windows 开发工具路径。
            candidates.push(dev_bin.join(format!("{name}.exe")));
        // 本行目的：结束 Windows 判断。
        }

        // 本行目的：追加通用开发工具路径。
        candidates.push(dev_bin.join(name));
    // 本行目的：结束调试模式判断。
    }

    // 本行目的：遍历候选路径并返回第一个存在的路径。
    for path in candidates {
        // 本行目的：检查路径是否存在。
        if path.exists() {
            // 本行目的：返回匹配路径。
            return Ok(path);
        // 本行目的：结束存在性判断。
        }
    // 本行目的：结束候选路径遍历。
    }

    // 本行目的：未找到工具时返回错误。
    Err(format!("未找到内置工具 {name}，请检查打包资源是否包含对应文件"))
// 本行目的：结束工具路径解析函数。
}

/// 函数：map_language_name | 输入：语言代码 | 输出：语言名称 | 可能失败：无
// 本行目的：将语言代码映射为中文名称。
fn map_language_name(code: &str) -> Option<String> {
    // 变量：normalized | 含义：规范化后的语言代码 | 类型：String | 作用域：map_language_name
    // 本行目的：去空白并转换为小写。
    let normalized = code.trim().to_lowercase();

    // 本行目的：处理简体中文前缀。
    if normalized.starts_with("zh-hans") {
        // 本行目的：返回简体中文名称。
        return Some("简体中文".to_string());
    // 本行目的：结束简体中文判断。
    }

    // 本行目的：处理繁体中文前缀。
    if normalized.starts_with("zh-hant") || normalized.starts_with("zh-hk") || normalized.starts_with("zh-mo") {
        // 本行目的：返回繁体中文名称。
        return Some("繁体中文".to_string());
    // 本行目的：结束繁体中文判断。
    }

    // 变量：name | 含义：匹配到的语言名称 | 类型：&str | 作用域：map_language_name
    // 本行目的：按语言代码匹配中文名称。
    let name = match normalized.as_str() {
        // 本行目的：日语代码映射。
        "jpn" | "ja" => "日语",
        // 本行目的：英语代码映射。
        "eng" | "en" => "英语",
        // 本行目的：中文代码映射。
        "chi" | "zho" | "zh" => "中文",
        // 本行目的：简体中文代码映射。
        "chs" | "zh-cn" | "cmn" => "简体中文",
        // 本行目的：繁体中文代码映射。
        "cht" | "zh-tw" => "繁体中文",
        // 本行目的：韩语代码映射。
        "kor" | "ko" => "韩语",
        // 本行目的：法语代码映射。
        "fra" | "fr" => "法语",
        // 本行目的：德语代码映射。
        "deu" | "ger" | "de" => "德语",
        // 本行目的：西班牙语代码映射。
        "spa" | "es" => "西班牙语",
        // 本行目的：未知代码返回空字符串。
        _ => "",
    // 本行目的：结束语言代码匹配。
    };

    // 本行目的：若为空则返回 None。
    if name.is_empty() {
        // 本行目的：返回 None 表示未知语言。
        None
    // 本行目的：处理有效名称。
    } else {
        // 本行目的：返回语言名称字符串。
        Some(name.to_string())
    // 本行目的：结束名称判断。
    }
// 本行目的：结束语言名称映射函数。
}

/// 函数：build_attributes | 输入：ffprobe 轨道 | 输出：属性字符串 | 可能失败：无
// 本行目的：构建轨道的附加属性字符串。
fn build_attributes(stream: &FFProbeStream) -> Option<String> {
    // 本行目的：处理视频轨道属性。
    if stream.codec_type.as_deref() == Some("video") {
        // 变量：parts | 含义：属性片段列表 | 类型：Vec<String> | 作用域：build_attributes
        // 本行目的：初始化属性片段列表。
        let mut parts = Vec::new();

        // 本行目的：若存在宽高则添加分辨率。
        if let (Some(w), Some(h)) = (stream.width, stream.height) {
            // 本行目的：追加分辨率描述。
            parts.push(format!("{}x{}", w, h));
        // 本行目的：结束宽高判断。
        }

        // 本行目的：若存在帧率则添加帧率。
        if let Some(rate) = stream.r_frame_rate.as_ref() {
            // 本行目的：过滤无效帧率。
            if rate != "0/0" {
                // 本行目的：追加帧率描述。
                parts.push(rate.to_string());
            // 本行目的：结束帧率有效性判断。
            }
        // 本行目的：结束帧率判断。
        }

        // 本行目的：根据是否有属性返回结果。
        return if parts.is_empty() {
            // 本行目的：无属性时返回 None。
            None
        // 本行目的：处理有属性的情况。
        } else {
            // 本行目的：合并属性片段返回。
            Some(parts.join(" "))
        // 本行目的：结束属性分支。
        };
    // 本行目的：结束视频轨道判断。
    }

    // 本行目的：处理音频轨道属性。
    if stream.codec_type.as_deref() == Some("audio") {
        // 变量：parts | 含义：属性片段列表 | 类型：Vec<String> | 作用域：build_attributes
        // 本行目的：初始化属性片段列表。
        let mut parts = Vec::new();

        // 本行目的：若存在声道数则添加声道描述。
        if let Some(ch) = stream.channels {
            // 本行目的：追加声道描述。
            parts.push(format!("{}ch", ch));
        // 本行目的：结束声道判断。
        }

        // 本行目的：若存在声道布局则添加布局描述。
        if let Some(layout) = stream.channel_layout.as_ref() {
            // 本行目的：追加声道布局描述。
            parts.push(layout.to_string());
        // 本行目的：结束布局判断。
        }

        // 本行目的：根据是否有属性返回结果。
        return if parts.is_empty() {
            // 本行目的：无属性时返回 None。
            None
        // 本行目的：处理有属性的情况。
        } else {
            // 本行目的：合并属性片段返回。
            Some(parts.join(" "))
        // 本行目的：结束属性分支。
        };
    // 本行目的：结束音频轨道判断。
    }

    // 本行目的：对其他类型返回标题作为属性。
    stream.tags.as_ref().and_then(|t| t.title.clone())
// 本行目的：结束属性构建函数。
}

/// 函数：build_mkvmerge_attributes | 输入：mkvmerge 轨道属性与类型 | 输出：属性字符串 | 可能失败：无
// 本行目的：构建 mkvmerge 轨道属性字符串。
fn build_mkvmerge_attributes(props: &MkvmergeTrackProperties, kind: &str) -> Option<String> {
    // 本行目的：处理视频轨道属性。
    if kind == "video" {
        // 本行目的：返回分辨率属性。
        return props.pixel_dimensions.clone();
    // 本行目的：结束视频判断。
    }

    // 本行目的：处理音频轨道属性。
    if kind == "audio" {
        // 变量：parts | 含义：属性片段列表 | 类型：Vec<String> | 作用域：build_mkvmerge_attributes
        // 本行目的：初始化属性片段列表。
        let mut parts = Vec::new();

        // 本行目的：若存在声道数则添加声道描述。
        if let Some(ch) = props.audio_channels {
            // 本行目的：追加声道描述。
            parts.push(format!("{}ch", ch));
        // 本行目的：结束声道判断。
        }

        // 本行目的：若存在采样率则添加采样率描述。
        if let Some(freq) = props.audio_sampling_frequency {
            // 本行目的：追加采样率描述。
            parts.push(format!("{} Hz", freq.round() as u64));
        // 本行目的：结束采样率判断。
        }

        // 本行目的：根据是否有属性返回结果。
        return if parts.is_empty() {
            // 本行目的：无属性时返回 None。
            None
        // 本行目的：处理有属性的情况。
        } else {
            // 本行目的：合并属性片段返回。
            Some(parts.join(" "))
        // 本行目的：结束属性分支。
        };
    // 本行目的：结束音频判断。
    }

    // 本行目的：其他类型无额外属性。
    None
// 本行目的：结束 mkvmerge 属性构建函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：parse_media_tracks | 输入：App 句柄、文件路径、轨道类型 | 输出：轨道解析响应 | 可能失败：工具调用或解析失败
// 本行目的：解析媒体文件中的轨道信息。
pub async fn parse_media_tracks(app: tauri::AppHandle, path: String, kind: String) -> Result<TrackParseResponse, String> {
    // 变量：kind_lower | 含义：轨道类型小写 | 类型：String | 作用域：parse_media_tracks
    // 本行目的：规范化轨道类型为小写。
    let kind_lower = kind.to_lowercase();
    // 变量：ext | 含义：文件扩展名小写 | 类型：String | 作用域：parse_media_tracks
    // 本行目的：提取并规范化文件扩展名。
    let ext = Path::new(&path).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    // 本行目的：针对 MKV 家族使用 mkvmerge 解析。
    if ["mkv", "mka", "mks"].contains(&ext.as_str()) {
        // 变量：mkvmerge_path | 含义：mkvmerge 工具路径 | 类型：PathBuf | 作用域：parse_media_tracks
        // 本行目的：解析 mkvmerge 工具路径。
        let mkvmerge_path = resolve_tool_path(&app, "mkvmerge")?;
        // 变量：output | 含义：命令输出 | 类型：tokio::process::Output | 作用域：parse_media_tracks
        // 本行目的：调用 mkvmerge 输出 JSON。
        let output = Command::new(mkvmerge_path).args(["-J", &path]).output().await.map_err(|e| format!("调用 mkvmerge 失败: {e}"))?;

        // 本行目的：检查 mkvmerge 是否执行成功。
        if !output.status.success() {
            // 变量：stderr | 含义：错误输出 | 类型：Cow<str> | 作用域：parse_media_tracks
            // 本行目的：读取错误输出。
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 本行目的：返回执行失败错误。
            return Err(format!("mkvmerge 执行失败: {stderr}"));
        // 本行目的：结束执行成功判断。
        }

        // 变量：parsed | 含义：解析后的 mkvmerge 输出 | 类型：MkvmergeOutput | 作用域：parse_media_tracks
        // 本行目的：解析 mkvmerge JSON 输出。
        let parsed: MkvmergeOutput = serde_json::from_slice(&output.stdout).map_err(|e| format!("解析 mkvmerge 输出失败: {e}"))?;
        // 变量：container | 含义：容器类型 | 类型：Option<String> | 作用域：parse_media_tracks
        // 本行目的：提取容器类型。
        let container = parsed.container.as_ref().and_then(|c| c.r#type.clone());
        // 变量：file_size | 含义：文件大小 | 类型：Option<String> | 作用域：parse_media_tracks
        // 本行目的：提取并格式化文件大小。
        let file_size = parsed.container.as_ref().and_then(|c| c.properties.as_ref()).and_then(|p| p.file_size).map(format_bytes_readable);
        // 变量：tracks | 含义：轨道列表 | 类型：Vec<TrackInfoResponse> | 作用域：parse_media_tracks
        // 本行目的：过滤并映射轨道列表。
        let tracks = parsed
            // 本行目的：读取轨道列表。
            .tracks
            // 本行目的：若为空则返回默认列表。
            .unwrap_or_default()
            // 本行目的：转换为迭代器。
            .into_iter()
            // 本行目的：筛选指定类型轨道。
            .filter(|track| {
                // 变量：track_type | 含义：轨道类型字符串 | 类型：&str | 作用域：parse_media_tracks
                // 本行目的：提取轨道类型。
                let track_type = track.r#type.as_deref().unwrap_or("");
                // 本行目的：字幕类型需兼容两种标记。
                if kind_lower == "subtitle" {
                    // 本行目的：返回字幕类型匹配结果。
                    return track_type == "subtitles" || track_type == "subtitle";
                // 本行目的：结束字幕判断。
                }
                // 本行目的：匹配其他轨道类型。
                track_type == kind_lower
            // 本行目的：结束过滤闭包。
            })
            // 本行目的：映射为响应结构。
            .map(|track| {
                // 变量：props | 含义：轨道属性 | 类型：MkvmergeTrackProperties | 作用域：parse_media_tracks
                // 本行目的：为缺失属性提供默认结构。
                let props = track.properties.unwrap_or(MkvmergeTrackProperties {
                    // 本行目的：默认语言为空。
                    language: None,
                    // 本行目的：默认 IETF 语言为空。
                    language_ietf: None,
                    // 本行目的：默认轨道名为空。
                    track_name: None,
                    // 本行目的：默认默认标记为空。
                    default_track: None,
                    // 本行目的：默认强制标记为空。
                    forced_track: None,
                    // 本行目的：默认编码名为空。
                    codec_name: None,
                    // 本行目的：默认编码 ID 为空。
                    codec_id: None,
                    // 本行目的：默认编码信息为空。
                    encoding: None,
                    // 本行目的：默认分辨率为空。
                    pixel_dimensions: None,
                    // 本行目的：默认声道数为空。
                    audio_channels: None,
                    // 本行目的：默认采样率为空。
                    audio_sampling_frequency: None,
                // 本行目的：结束默认属性构建。
                });
                // 变量：lang | 含义：语言代码 | 类型：Option<String> | 作用域：parse_media_tracks
                // 本行目的：优先选择 IETF 语言代码。
                let lang = props.language_ietf.clone().or(props.language.clone());
                // 变量：language_name | 含义：语言名称 | 类型：Option<String> | 作用域：parse_media_tracks
                // 本行目的：将语言代码映射为中文名称。
                let language_name = lang.as_deref().and_then(map_language_name);
                // 变量：codec | 含义：编码名称 | 类型：String | 作用域：parse_media_tracks
                // 本行目的：选择可用的编码名称。
                let codec = props.codec_name.clone().or(track.codec.clone()).or(props.codec_id.clone()).unwrap_or_else(|| "unknown".to_string());
                // 本行目的：构建轨道响应对象。
                TrackInfoResponse {
                    // 本行目的：写入轨道 ID。
                    track_id: track.id.to_string(),
                    // 本行目的：写入编码名称。
                    codec,
                    // 本行目的：写入语言代码。
                    lang,
                    // 本行目的：写入语言名称。
                    language_name,
                    // 本行目的：写入轨道名称。
                    track_name: props.track_name.clone(),
                    // 本行目的：写入默认标记。
                    is_default: props.default_track,
                    // 本行目的：写入强制标记。
                    is_forced: props.forced_track,
                    // 本行目的：写入字符集。
                    charset: props.encoding.clone(),
                    // 本行目的：写入属性描述。
                    attributes: build_mkvmerge_attributes(&props, &kind_lower),
                    // 本行目的：写入容器类型。
                    container: container.clone(),
                    // 本行目的：写入文件大小。
                    file_size: file_size.clone(),
                // 本行目的：结束轨道响应构建。
                }
            // 本行目的：结束映射闭包。
            })
            // 本行目的：收集为向量。
            .collect();

        // 本行目的：返回轨道解析响应。
        return Ok(TrackParseResponse {
            // 本行目的：写入轨道列表。
            tracks,
        // 本行目的：结束响应构建。
        });
    // 本行目的：结束 mkvmerge 处理分支。
    }

    // 变量：ffprobe_path | 含义：ffprobe 工具路径 | 类型：PathBuf | 作用域：parse_media_tracks
    // 本行目的：解析 ffprobe 工具路径。
    let ffprobe_path = resolve_tool_path(&app, "ffprobe")?;
    // 变量：output | 含义：命令输出 | 类型：tokio::process::Output | 作用域：parse_media_tracks
    // 本行目的：调用 ffprobe 输出 JSON。
    let output = Command::new(ffprobe_path)
        // 本行目的：设置 ffprobe 参数。
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams", &path])
        // 本行目的：执行命令。
        .output()
        // 本行目的：等待命令执行。
        .await
        // 本行目的：转换错误为字符串。
        .map_err(|e| format!("调用 ffprobe 失败: {e}"))?;

    // 本行目的：检查 ffprobe 是否执行成功。
    if !output.status.success() {
        // 变量：stderr | 含义：错误输出 | 类型：Cow<str> | 作用域：parse_media_tracks
        // 本行目的：读取错误输出。
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 本行目的：返回执行失败错误。
        return Err(format!("ffprobe 执行失败: {stderr}"));
    // 本行目的：结束执行成功判断。
    }

    // 变量：parsed | 含义：解析后的 ffprobe 输出 | 类型：FFProbeOutput | 作用域：parse_media_tracks
    // 本行目的：解析 ffprobe JSON 输出。
    let parsed: FFProbeOutput = serde_json::from_slice(&output.stdout).map_err(|e| format!("解析 ffprobe 输出失败: {e}"))?;
    // 变量：container | 含义：容器类型 | 类型：Option<String> | 作用域：parse_media_tracks
    // 本行目的：提取容器类型。
    let container = parsed.format.as_ref().and_then(|f| f.format_name.clone());
    // 变量：file_size | 含义：文件大小 | 类型：Option<String> | 作用域：parse_media_tracks
    // 本行目的：提取并格式化文件大小。
    let file_size = parsed.format.as_ref().and_then(|f| f.size.as_ref()).and_then(|s| s.parse::<u64>().ok()).map(format_bytes_readable);
    // 变量：streams | 含义：轨道列表 | 类型：Vec<FFProbeStream> | 作用域：parse_media_tracks
    // 本行目的：获取轨道列表。
    let streams = parsed.streams.unwrap_or_default();
    // 变量：tracks | 含义：轨道列表 | 类型：Vec<TrackInfoResponse> | 作用域：parse_media_tracks
    // 本行目的：过滤并映射轨道列表。
    let tracks = streams
        // 本行目的：转换为迭代器。
        .into_iter()
        // 本行目的：筛选指定类型轨道。
        .filter(|stream| stream.codec_type.as_deref() == Some(&kind_lower))
        // 本行目的：映射为响应结构。
        .map(|stream| {
            // 变量：lang | 含义：语言代码 | 类型：Option<String> | 作用域：parse_media_tracks
            // 本行目的：提取语言代码。
            let lang = stream.tags.as_ref().and_then(|t| t.language.clone());
            // 变量：language_name | 含义：语言名称 | 类型：Option<String> | 作用域：parse_media_tracks
            // 本行目的：将语言代码映射为中文名称。
            let language_name = lang.as_deref().and_then(map_language_name);
            // 变量：track_name | 含义：轨道名称 | 类型：Option<String> | 作用域：parse_media_tracks
            // 本行目的：提取轨道名称。
            let track_name = stream.tags.as_ref().and_then(|t| t.title.clone());
            // 变量：charset | 含义：字符集 | 类型：Option<String> | 作用域：parse_media_tracks
            // 本行目的：提取字符集或编码信息。
            let charset = stream.tags.as_ref().and_then(|t| t.charset.clone().or_else(|| t.encoding.clone()));
            // 变量：codec | 含义：编码名称 | 类型：String | 作用域：parse_media_tracks
            // 本行目的：获取编码名称或使用 unknown。
            let codec = stream.codec_name.clone().unwrap_or_else(|| "unknown".to_string());
            // 变量：disposition | 含义：轨道标记 | 类型：FFProbeDisposition | 作用域：parse_media_tracks
            // 本行目的：提供默认标记值。
            let disposition = stream.disposition.clone().unwrap_or(FFProbeDisposition {
                // 本行目的：默认标记为空。
                default: None,
                // 本行目的：强制标记为空。
                forced: None,
            // 本行目的：结束默认标记构建。
            });

            // 本行目的：构建轨道响应对象。
            TrackInfoResponse {
                // 本行目的：写入轨道 ID。
                track_id: stream.index.unwrap_or(0).to_string(),
                // 本行目的：写入编码名称。
                codec,
                // 本行目的：写入语言代码。
                lang,
                // 本行目的：写入语言名称。
                language_name,
                // 本行目的：写入轨道名称。
                track_name,
                // 本行目的：写入默认标记。
                is_default: disposition.default.map(|v| v == 1),
                // 本行目的：写入强制标记。
                is_forced: disposition.forced.map(|v| v == 1),
                // 本行目的：写入字符集。
                charset,
                // 本行目的：写入属性描述。
                attributes: build_attributes(&stream),
                // 本行目的：写入容器类型。
                container: container.clone(),
                // 本行目的：写入文件大小。
                file_size: file_size.clone(),
            // 本行目的：结束轨道响应构建。
            }
        // 本行目的：结束映射闭包。
        })
        // 本行目的：收集为向量。
        .collect();

    // 本行目的：返回轨道解析响应。
    Ok(TrackParseResponse {
        // 本行目的：写入轨道列表。
        tracks,
    // 本行目的：结束响应构建。
    })
// 本行目的：结束轨道解析命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_media_file_size | 输入：文件路径 | 输出：可读大小 | 可能失败：读取元数据失败
// 本行目的：获取媒体文件大小并格式化。
pub async fn get_media_file_size(path: String) -> Result<Option<String>, String> {
    // 变量：meta | 含义：文件元数据 | 类型：std::fs::Metadata | 作用域：get_media_file_size
    // 本行目的：读取文件元数据。
    let meta = fs::metadata(&path).map_err(|e| format!("读取文件大小失败: {e}"))?;
    // 本行目的：返回格式化后的文件大小。
    Ok(Some(format_bytes_readable(meta.len())))
// 本行目的：结束文件大小命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：mix_media_tracks | 输入：App 句柄、轨道输入列表、输出路径 | 输出：输出路径 | 可能失败：工具调用或文件操作失败
// 本行目的：合成音视频与字幕轨道为输出文件。
pub async fn mix_media_tracks(app: tauri::AppHandle, inputs: Vec<MixTrackInput>, output_path: String) -> Result<String, String> {
    // 本行目的：校验输入列表不能为空。
    if inputs.is_empty() {
        // 本行目的：返回输入为空的错误。
        return Err("未提供可合成的轨道".to_string());
    // 本行目的：结束输入检查。
    }

    // 变量：output | 含义：输出路径 | 类型：PathBuf | 作用域：mix_media_tracks
    // 本行目的：初始化输出路径。
    let mut output = PathBuf::from(&output_path);

    // 本行目的：若没有扩展名则默认使用 mkv。
    if output.extension().is_none() {
        // 本行目的：设置输出文件扩展名。
        output.set_extension("mkv");
    // 本行目的：结束扩展名判断。
    }

    // 本行目的：确保输出目录存在。
    if let Some(parent) = output.parent() {
        // 本行目的：若父目录不存在则创建。
        if !parent.exists() {
            // 本行目的：创建输出目录。
            fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败: {e}"))?;
        // 本行目的：结束父目录存在判断。
        }
    // 本行目的：结束父目录判断。
    }

    // 变量：mkvmerge_path | 含义：mkvmerge 工具路径 | 类型：PathBuf | 作用域：mix_media_tracks
    // 本行目的：解析 mkvmerge 工具路径。
    let mkvmerge_path = resolve_tool_path(&app, "mkvmerge")?;

    // 本行目的：定义默认轨道语言规则。
    fn lang_for_kind(kind: &str) -> &'static str {
        // 本行目的：根据轨道类型选择语言代码。
        match kind {
            // 本行目的：视频轨道默认日语。
            "video" => "ja",
            // 本行目的：音频轨道默认日语。
            "audio" => "ja",
            // 本行目的：字幕轨道默认简体中文。
            "subtitle" => "zh-Hans",
            // 本行目的：其他轨道默认未指定。
            _ => "und",
        // 本行目的：结束类型匹配。
        }
    // 本行目的：结束语言规则函数。
    }

    // 本行目的：定义命令行参数格式化函数。
    fn format_arg(arg: &str) -> String {
        // 本行目的：若含空格或引号则进行转义。
        if arg.contains(' ') || arg.contains('\t') || arg.contains('"') {
            // 本行目的：返回带引号的参数。
            format!("\"{}\"", arg.replace('"', "\\\""))
        // 本行目的：处理无需转义的情况。
        } else {
            // 本行目的：直接返回原参数。
            arg.to_string()
        // 本行目的：结束参数格式化分支。
        }
    // 本行目的：结束参数格式化函数。
    }

    // 本行目的：定义命令行拼接函数。
    fn build_cmdline(args: &[String]) -> String {
        // 变量：cmdline | 含义：命令行参数列表 | 类型：Vec<String> | 作用域：build_cmdline
        // 本行目的：初始化命令行列表。
        let mut cmdline = Vec::new();
        // 本行目的：添加命令名。
        cmdline.push(format_arg("mkvmerge"));

        // 本行目的：追加格式化后的参数。
        for arg in args {
            // 本行目的：追加单个参数。
            cmdline.push(format_arg(arg));
        // 本行目的：结束参数遍历。
        }

        // 本行目的：拼接命令行字符串返回。
        cmdline.join(" ")
    // 本行目的：结束命令行拼接函数。
    }

    // 本行目的：定义执行 mkvmerge 的辅助函数。
    async fn run_mkvmerge(mkvmerge_path: &PathBuf, args: &[String]) -> Result<(), String> {
        // 变量：output_exec | 含义：命令执行结果 | 类型：tokio::process::Output | 作用域：run_mkvmerge
        // 本行目的：执行 mkvmerge 命令。
        let output_exec = Command::new(mkvmerge_path).args(args.iter()).output().await.map_err(|e| format!("调用 mkvmerge 失败: {e}"))?;

        // 本行目的：检查命令执行结果。
        if !output_exec.status.success() {
            // 变量：stderr | 含义：错误输出 | 类型：Cow<str> | 作用域：run_mkvmerge
            // 本行目的：读取错误输出。
            let stderr = String::from_utf8_lossy(&output_exec.stderr);
            // 变量：stdout | 含义：标准输出 | 类型：Cow<str> | 作用域：run_mkvmerge
            // 本行目的：读取标准输出。
            let stdout = String::from_utf8_lossy(&output_exec.stdout);

            // 本行目的：返回包含命令行的错误信息。
            return Err(format!(
                // 本行目的：格式化错误信息。
                "mkvmerge 执行失败 (code {:?}): {} {}\n命令: {}",
                // 本行目的：写入退出码。
                output_exec.status.code(),
                // 本行目的：写入标准输出。
                stdout.trim(),
                // 本行目的：写入错误输出。
                stderr.trim(),
                // 本行目的：写入命令行。
                build_cmdline(args)
            // 本行目的：结束错误格式化。
            ));
        // 本行目的：结束执行成功判断。
        }
        // 本行目的：执行成功返回 Ok。
        Ok(())
    // 本行目的：结束 mkvmerge 执行函数。
    }

    // 变量：kind_inputs | 含义：按类型分组的输入 | 类型：HashMap<String, MixTrackInput> | 作用域：mix_media_tracks
    // 本行目的：初始化轨道输入映射。
    let mut kind_inputs: HashMap<String, MixTrackInput> = HashMap::new();

    // 本行目的：遍历输入并进行校验与归并。
    for input in inputs {
        // 变量：path | 含义：轨道文件路径切片 | 类型：&str | 作用域：mix_media_tracks
        // 本行目的：去除路径首尾空白。
        let path = input.path.trim();

        // 本行目的：检查路径是否为空。
        if path.is_empty() {
            // 本行目的：返回路径为空错误。
            return Err("轨道文件路径为空".to_string());
        // 本行目的：结束路径空判断。
        }

        // 本行目的：检查轨道文件是否存在。
        if !Path::new(path).exists() {
            // 本行目的：返回文件不存在错误。
            return Err(format!("轨道文件不存在: {path}"));
        // 本行目的：结束文件存在判断。
        }

        // 变量：track_ids | 含义：清洗后的轨道 ID 列表 | 类型：Vec<String> | 作用域：mix_media_tracks
        // 本行目的：清洗轨道 ID 列表。
        let track_ids: Vec<String> = input.track_ids.into_iter().map(|id| id.trim().to_string()).filter(|id| !id.is_empty()).collect();

        // 本行目的：若无轨道 ID 则跳过。
        if track_ids.is_empty() {
            // 本行目的：继续处理下一个输入。
            continue;
        // 本行目的：结束轨道 ID 判断。
        }

        // 变量：kind_lower | 含义：轨道类型小写 | 类型：String | 作用域：mix_media_tracks
        // 本行目的：规范化轨道类型为小写。
        let kind_lower = input.kind.to_lowercase();
        // 变量：entry | 含义：轨道输入条目 | 类型：&mut MixTrackInput | 作用域：mix_media_tracks
        // 本行目的：按类型创建或获取输入条目。
        let entry = kind_inputs.entry(kind_lower.clone()).or_insert(MixTrackInput {
            // 本行目的：写入路径。
            path: path.to_string(),
            // 本行目的：写入类型。
            kind: kind_lower.clone(),
            // 本行目的：初始化轨道 ID 列表。
            track_ids: Vec::new(),
            // 本行目的：初始化语言映射。
            track_langs: HashMap::new(),
        // 本行目的：结束条目构建。
        });

        // 本行目的：同类型路径不一致则报错。
        if entry.path != path {
            // 本行目的：返回路径冲突错误。
            return Err(format!("同一类型只支持一个文件：{}", kind_lower));
        // 本行目的：结束路径一致性判断。
        }

        // 本行目的：合并轨道 ID 列表。
        for track_id in track_ids {
            // 本行目的：避免重复轨道 ID。
            if !entry.track_ids.iter().any(|id| id == &track_id) {
                // 本行目的：追加轨道 ID。
                entry.track_ids.push(track_id);
            // 本行目的：结束重复检查。
            }
        // 本行目的：结束轨道 ID 遍历。
        }

        // 本行目的：合并轨道语言映射。
        for (track_id, lang) in input.track_langs {
            // 本行目的：插入轨道语言映射。
            entry.track_langs.insert(track_id, lang);
        // 本行目的：结束语言映射遍历。
        }
    // 本行目的：结束输入遍历。
    }

    // 本行目的：必须至少包含视频轨道。
    if !kind_inputs.contains_key("video") {
        // 本行目的：返回缺少视频轨道错误。
        return Err("请先检测并选择至少一个视频轨道".to_string());
    // 本行目的：结束视频轨道检查。
    }

    // 变量：temp_root | 含义：临时目录根路径 | 类型：PathBuf | 作用域：mix_media_tracks
    // 本行目的：构建临时目录路径。
    let temp_root = app
        // 本行目的：获取路径工具。
        .path()
        // 本行目的：获取应用数据目录。
        .app_data_dir()
        // 本行目的：转换错误为字符串。
        .map_err(|e| format!("无法获取数据目录: {e}"))?
        // 本行目的：拼接应用子目录。
        .join("hanamirip-cn")
        // 本行目的：拼接混流临时目录。
        .join("mix-temp")
        // 本行目的：使用时间戳创建唯一目录。
        .join(chrono::Utc::now().timestamp_millis().to_string());

    // 本行目的：创建临时目录。
    fs::create_dir_all(&temp_root).map_err(|e| format!("创建临时目录失败: {e}"))?;

    // 变量：temp_files | 含义：临时文件列表 | 类型：Vec<PathBuf> | 作用域：mix_media_tracks
    // 本行目的：初始化临时文件列表。
    let mut temp_files: Vec<PathBuf> = Vec::new();

    // 本行目的：定义生成临时轨道文件的函数。
    async fn build_temp(kind: &str, input: &MixTrackInput, temp_root: &PathBuf, mkvmerge_path: &PathBuf) -> Result<PathBuf, String> {
        // 变量：ext | 含义：输出扩展名 | 类型：&str | 作用域：build_temp
        // 本行目的：根据轨道类型决定扩展名。
        let ext = match kind {
            // 本行目的：视频轨道扩展名。
            "video" => "mkv",
            // 本行目的：音频轨道扩展名。
            "audio" => "mka",
            // 本行目的：字幕轨道扩展名。
            "subtitle" => "mks",
            // 本行目的：其他轨道默认扩展名。
            _ => "mkv",
        // 本行目的：结束扩展名选择。
        };

        // 变量：temp_path | 含义：临时输出路径 | 类型：PathBuf | 作用域：build_temp
        // 本行目的：构建临时输出路径。
        let temp_path = temp_root.join(format!("{kind}.{ext}"));
        // 变量：args | 含义：mkvmerge 参数列表 | 类型：Vec<String> | 作用域：build_temp
        // 本行目的：初始化参数列表。
        let mut args: Vec<String> = Vec::new();

        // 本行目的：设置输出参数。
        args.push("-o".to_string());
        // 本行目的：设置输出路径参数。
        args.push(temp_path.to_string_lossy().to_string());

        // 本行目的：根据轨道类型设置轨道筛选参数。
        match kind {
            // 本行目的：视频轨道参数配置。
            "video" => {
                // 本行目的：设置视频轨道 ID。
                args.push("--video-tracks".to_string());
                // 本行目的：写入轨道 ID 列表。
                args.push(input.track_ids.join(","));
                // 本行目的：禁用音频轨道。
                args.push("--audio-tracks".to_string());
                // 本行目的：设置为无音频。
                args.push("-1".to_string());
                // 本行目的：禁用字幕轨道。
                args.push("--subtitle-tracks".to_string());
                // 本行目的：设置为无字幕。
                args.push("-1".to_string());
            // 本行目的：结束视频轨道配置。
            }
            // 本行目的：音频轨道参数配置。
            "audio" => {
                // 本行目的：设置音频轨道 ID。
                args.push("--audio-tracks".to_string());
                // 本行目的：写入轨道 ID 列表。
                args.push(input.track_ids.join(","));
                // 本行目的：禁用视频轨道。
                args.push("--video-tracks".to_string());
                // 本行目的：设置为无视频。
                args.push("-1".to_string());
                // 本行目的：禁用字幕轨道。
                args.push("--subtitle-tracks".to_string());
                // 本行目的：设置为无字幕。
                args.push("-1".to_string());
            // 本行目的：结束音频轨道配置。
            }
            // 本行目的：字幕轨道参数配置。
            "subtitle" => {
                // 本行目的：设置字幕轨道 ID。
                args.push("--subtitle-tracks".to_string());
                // 本行目的：写入轨道 ID 列表。
                args.push(input.track_ids.join(","));
                // 本行目的：禁用视频轨道。
                args.push("--video-tracks".to_string());
                // 本行目的：设置为无视频。
                args.push("-1".to_string());
                // 本行目的：禁用音频轨道。
                args.push("--audio-tracks".to_string());
                // 本行目的：设置为无音频。
                args.push("-1".to_string());
            // 本行目的：结束字幕轨道配置。
            }
            // 本行目的：其他类型不设置额外参数。
            _ => {}
        // 本行目的：结束轨道类型匹配。
        }

        // 变量：lang | 含义：默认语言代码 | 类型：&str | 作用域：build_temp
        // 本行目的：获取默认语言代码。
        let lang = lang_for_kind(kind);

        // 本行目的：为每个轨道设置名称与语言。
        for track_id in &input.track_ids {
            // 本行目的：设置轨道名称参数。
            args.push("--track-name".to_string());
            // 本行目的：指定轨道名称为空。
            args.push(format!("{track_id}:"));
            // 本行目的：设置默认轨道标记。
            args.push("--default-track-flag".to_string());
            // 本行目的：标记为默认。
            args.push(format!("{track_id}:yes"));
            // 本行目的：设置强制显示标记。
            args.push("--forced-display-flag".to_string());
            // 本行目的：标记为非强制。
            args.push(format!("{track_id}:no"));
            // 本行目的：设置语言参数。
            args.push("--language".to_string());
            // 变量：custom_lang | 含义：自定义语言 | 类型：Option<&str> | 作用域：build_temp
            // 本行目的：读取自定义语言。
            let custom_lang = input.track_langs.get(track_id).map(|v| v.as_str());
            // 变量：final_lang | 含义：最终语言代码 | 类型：&str | 作用域：build_temp
            // 本行目的：选择自定义语言或默认语言。
            let final_lang = custom_lang.unwrap_or(lang);
            // 本行目的：写入轨道语言参数。
            args.push(format!("{track_id}:{final_lang}"));
        // 本行目的：结束轨道参数循环。
        }

        // 本行目的：追加输入文件路径。
        args.push(input.path.clone());
        // 本行目的：执行 mkvmerge 生成临时文件。
        run_mkvmerge(mkvmerge_path, &args).await?;
        // 本行目的：返回临时文件路径。
        Ok::<PathBuf, String>(temp_path)
    // 本行目的：结束临时文件构建函数。
    }

    // 变量：video_temp | 含义：视频临时文件路径 | 类型：Option<PathBuf> | 作用域：mix_media_tracks
    // 本行目的：初始化视频临时路径。
    let mut video_temp = None;
    // 变量：audio_temp | 含义：音频临时文件路径 | 类型：Option<PathBuf> | 作用域：mix_media_tracks
    // 本行目的：初始化音频临时路径。
    let mut audio_temp = None;
    // 变量：subtitle_temp | 含义：字幕临时文件路径 | 类型：Option<PathBuf> | 作用域：mix_media_tracks
    // 本行目的：初始化字幕临时路径。
    let mut subtitle_temp = None;

    // 本行目的：构建视频临时文件。
    if let Some(input) = kind_inputs.get("video") {
        // 变量：path | 含义：临时文件路径 | 类型：PathBuf | 作用域：mix_media_tracks
        // 本行目的：生成视频临时文件。
        let path = build_temp("video", input, &temp_root, &mkvmerge_path).await?;
        // 本行目的：记录临时文件用于清理。
        temp_files.push(path.clone());
        // 本行目的：保存视频临时路径。
        video_temp = Some(path);
    // 本行目的：结束视频临时文件构建。
    }

    // 本行目的：构建音频临时文件。
    if let Some(input) = kind_inputs.get("audio") {
        // 变量：path | 含义：临时文件路径 | 类型：PathBuf | 作用域：mix_media_tracks
        // 本行目的：生成音频临时文件。
        let path = build_temp("audio", input, &temp_root, &mkvmerge_path).await?;
        // 本行目的：记录临时文件用于清理。
        temp_files.push(path.clone());
        // 本行目的：保存音频临时路径。
        audio_temp = Some(path);
    // 本行目的：结束音频临时文件构建。
    }

    // 本行目的：构建字幕临时文件。
    if let Some(input) = kind_inputs.get("subtitle") {
        // 变量：path | 含义：临时文件路径 | 类型：PathBuf | 作用域：mix_media_tracks
        // 本行目的：生成字幕临时文件。
        let path = build_temp("subtitle", input, &temp_root, &mkvmerge_path).await?;
        // 本行目的：记录临时文件用于清理。
        temp_files.push(path.clone());
        // 本行目的：保存字幕临时路径。
        subtitle_temp = Some(path);
    // 本行目的：结束字幕临时文件构建。
    }

    // 变量：merge_args | 含义：合成命令参数列表 | 类型：Vec<String> | 作用域：mix_media_tracks
    // 本行目的：初始化合成参数列表。
    let mut merge_args: Vec<String> = Vec::new();
    // 本行目的：设置输出参数。
    merge_args.push("-o".to_string());
    // 本行目的：写入输出路径。
    merge_args.push(output.to_string_lossy().to_string());

    // 本行目的：追加视频临时文件。
    if let Some(path) = video_temp.as_ref() {
        // 本行目的：写入视频临时路径。
        merge_args.push(path.to_string_lossy().to_string());
    // 本行目的：结束视频临时路径判断。
    }

    // 本行目的：追加音频临时文件。
    if let Some(path) = audio_temp.as_ref() {
        // 本行目的：写入音频临时路径。
        merge_args.push(path.to_string_lossy().to_string());
    // 本行目的：结束音频临时路径判断。
    }

    // 本行目的：追加字幕临时文件。
    if let Some(path) = subtitle_temp.as_ref() {
        // 本行目的：写入字幕临时路径。
        merge_args.push(path.to_string_lossy().to_string());
    // 本行目的：结束字幕临时路径判断。
    }

    // 本行目的：执行最终合成命令。
    run_mkvmerge(&mkvmerge_path, &merge_args).await?;

    // 本行目的：清理临时文件。
    for path in temp_files {
        // 本行目的：删除临时文件，忽略错误。
        let _ = fs::remove_file(path);
    // 本行目的：结束临时文件清理遍历。
    }

    // 本行目的：返回输出路径字符串。
    Ok(output.to_string_lossy().to_string())
// 本行目的：结束轨道合成命令。
}
