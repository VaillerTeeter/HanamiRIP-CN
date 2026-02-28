/* 文件：mod.rs | 用途：管理追番数据的本地存储 | 关键对象：TrackedSubject、db_path、load_tracked */
// 本行目的：引入日期相关 trait。
use chrono::Datelike;
// 本行目的：引入本地时间获取器。
use chrono::Local;
// 本行目的：引入反序列化支持。
use serde::Deserialize;
// 本行目的：引入序列化支持。
use serde::Serialize;
// 本行目的：引入哈希映射类型。
use std::collections::HashMap;
// 本行目的：引入环境变量读取模块。
use std::env;
// 本行目的：引入文件系统模块。
use std::fs;
// 本行目的：引入路径缓冲类型。
use std::path::PathBuf;
// 本行目的：引入 Tauri 管理器 trait。
use tauri::Manager;

// 变量：TRACK_DB_FILE | 含义：追番数据文件名 | 类型：&str | 作用域：模块级
// 本行目的：定义追番数据文件名常量。
const TRACK_DB_FILE: &str = "watchlist.json";

// 本行目的：为追番数据结构启用序列化/反序列化与克隆。
#[derive(Serialize, Deserialize, Clone)]
// 本行目的：使用 camelCase 序列化字段名。
#[serde(rename_all = "camelCase")]
// 本行目的：定义追番条目结构体。
pub struct TrackedSubject {
    // 变量：id | 含义：条目 ID | 类型：u32 | 作用域：TrackedSubject
    // 本行目的：记录条目标识。
    pub id: u32,
    // 变量：name | 含义：原始名称 | 类型：String | 作用域：TrackedSubject
    // 本行目的：记录原始名称。
    pub name: String,
    // 变量：name_cn | 含义：中文名称 | 类型：String | 作用域：TrackedSubject
    // 本行目的：记录中文名称。
    pub name_cn: String,
    // 变量：image | 含义：图片 URL | 类型：String | 作用域：TrackedSubject
    // 本行目的：记录图片地址。
    pub image: String,
    // 变量：url | 含义：条目链接 | 类型：String | 作用域：TrackedSubject
    // 本行目的：记录条目链接。
    pub url: String,
    // 变量：watching | 含义：是否在看 | 类型：bool | 作用域：TrackedSubject
    // 本行目的：记录在看状态。
    pub watching: bool,
    // 变量：backlog | 含义：是否想看 | 类型：bool | 作用域：TrackedSubject
    // 本行目的：记录想看状态。
    pub backlog: bool,
    // 变量：watched | 含义：是否看完 | 类型：bool | 作用域：TrackedSubject
    // 本行目的：记录看完状态。
    pub watched: bool,
    // 变量：date | 含义：日期 | 类型：String | 作用域：TrackedSubject
    // 本行目的：记录日期。
    pub date: String,
    // 变量：rating | 含义：评分 | 类型：Option<f64> | 作用域：TrackedSubject
    // 本行目的：记录评分。
    pub rating: Option<f64>,
    // 变量：summary | 含义：简介 | 类型：String | 作用域：TrackedSubject
    // 本行目的：记录简介。
    pub summary: String,

    // 本行目的：为缺失别名提供默认值。
    #[serde(default)]
    // 变量：aliases | 含义：别名列表 | 类型：Option<Vec<String>> | 作用域：TrackedSubject
    // 本行目的：记录别名列表。
    pub aliases: Option<Vec<String>>,

    // 本行目的：为缺失已播集数提供默认值。
    #[serde(default)]
    // 变量：aired_count | 含义：已播集数 | 类型：Option<u32> | 作用域：TrackedSubject
    // 本行目的：记录已播集数。
    pub aired_count: Option<u32>,

    // 本行目的：为缺失总集数提供默认值。
    #[serde(default)]
    // 变量：total_count | 含义：总集数 | 类型：Option<u32> | 作用域：TrackedSubject
    // 本行目的：记录总集数。
    pub total_count: Option<u32>,
// 本行目的：结束追番条目结构体定义。
}

/// 函数：db_path | 输入：App 句柄引用 | 输出：追番数据文件路径 | 可能失败：无法解析或创建目录
// 本行目的：获取追番数据文件路径。
fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 变量：dir | 含义：应用数据目录 | 类型：PathBuf | 作用域：db_path
    // 本行目的：解析应用数据目录并追加子目录。
    let dir = app.path().app_data_dir().map_err(|e| format!("无法获取数据目录: {e}"))?.join("hanamirip-cn");
    // 变量：primary | 含义：主追番数据文件路径 | 类型：PathBuf | 作用域：db_path
    // 本行目的：构造主追番数据文件路径。
    let primary = dir.join(TRACK_DB_FILE);

    // 本行目的：当主文件不存在时尝试读取旧路径。
    if !primary.exists() {
        // 本行目的：尝试读取旧版 LocalAppData 路径。
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            // 变量：legacy | 含义：旧版追番数据路径 | 类型：PathBuf | 作用域：db_path
            // 本行目的：拼接旧版追番数据文件路径。
            let legacy = PathBuf::from(local_app_data).join("HanamiRIP CN").join("user-data").join(TRACK_DB_FILE);

            // 本行目的：如果旧路径存在则返回。
            if legacy.exists() {
                // 本行目的：返回旧版数据文件路径。
                return Ok(legacy);
            // 本行目的：结束旧路径存在判断。
            }
        // 本行目的：结束环境变量读取判断。
        }
    // 本行目的：结束主文件存在判断。
    }

    // 本行目的：确保数据目录存在。
    fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;

    // 本行目的：返回主追番数据路径。
    Ok(primary)
// 本行目的：结束路径解析函数。
}

/// 函数：load_tracked | 输入：App 句柄引用 | 输出：追番数据映射 | 可能失败：读取或解析失败
// 本行目的：从本地文件加载追番数据。
fn load_tracked(app: &tauri::AppHandle) -> Result<HashMap<u32, TrackedSubject>, String> {
    // 变量：path | 含义：数据文件路径 | 类型：PathBuf | 作用域：load_tracked
    // 本行目的：获取追番数据路径。
    let path = db_path(app)?;

    // 本行目的：若数据文件不存在则返回空映射。
    if !path.exists() {
        // 本行目的：返回空映射。
        return Ok(HashMap::new());
    // 本行目的：结束文件存在判断。
    }

    // 变量：content | 含义：文件内容字符串 | 类型：String | 作用域：load_tracked
    // 本行目的：读取追番数据文件内容。
    let content = fs::read_to_string(&path).map_err(|e| format!("读取追番数据失败: {e}"))?;

    // 本行目的：若内容为空则返回空映射。
    if content.trim().is_empty() {
        // 本行目的：返回空映射。
        return Ok(HashMap::new());
    // 本行目的：结束空内容判断。
    }

    // 变量：parsed | 含义：解析后的追番列表 | 类型：Vec<TrackedSubject> | 作用域：load_tracked
    // 本行目的：解析 JSON 为追番列表。
    let parsed: Vec<TrackedSubject> = serde_json::from_str(&content).map_err(|e| format!("解析追番数据失败: {e}"))?;
    // 变量：map | 含义：追番映射 | 类型：HashMap<u32, TrackedSubject> | 作用域：load_tracked
    // 本行目的：初始化追番映射。
    let mut map = HashMap::new();

    // 本行目的：将列表转换为映射。
    for item in parsed {
        // 本行目的：以条目 ID 为键写入映射。
        map.insert(item.id, item);
    // 本行目的：结束列表遍历。
    }

    // 本行目的：返回追番映射。
    Ok(map)
// 本行目的：结束追番加载函数。
}

/// 函数：persist_tracked | 输入：App 句柄引用与追番映射 | 输出：空结果或错误 | 可能失败：序列化或写入失败
// 本行目的：将追番数据写回本地文件。
fn persist_tracked(app: &tauri::AppHandle, data: &HashMap<u32, TrackedSubject>) -> Result<(), String> {
    // 变量：path | 含义：数据文件路径 | 类型：PathBuf | 作用域：persist_tracked
    // 本行目的：获取追番数据路径。
    let path = db_path(app)?;
    // 变量：list | 含义：追番列表 | 类型：Vec<TrackedSubject> | 作用域：persist_tracked
    // 本行目的：将映射转换为列表。
    let list: Vec<_> = data.values().cloned().collect();
    // 变量：payload | 含义：序列化后的 JSON 字符串 | 类型：String | 作用域：persist_tracked
    // 本行目的：将追番列表序列化为 JSON。
    let payload = serde_json::to_string_pretty(&list).map_err(|e| format!("序列化追番数据失败: {e}"))?;

    // 本行目的：写入数据文件并返回结果。
    fs::write(&path, payload).map_err(|e| format!("写入追番数据失败: {e}"))
// 本行目的：结束追番持久化函数。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：list_tracked_subjects | 输入：App 句柄 | 输出：追番列表 | 可能失败：读取或解析失败
// 本行目的：列出所有追番条目。
pub fn list_tracked_subjects(app: tauri::AppHandle) -> Result<Vec<TrackedSubject>, String> {
    // 变量：data | 含义：追番映射 | 类型：HashMap<u32, TrackedSubject> | 作用域：list_tracked_subjects
    // 本行目的：加载追番数据映射。
    let data = load_tracked(&app)?;
    // 本行目的：返回追番列表。
    Ok(data.values().cloned().collect())
// 本行目的：结束追番列表命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：save_tracked_subject | 输入：App 句柄与条目 | 输出：追番列表 | 可能失败：读取或写入失败
// 本行目的：保存或删除追番条目。
pub fn save_tracked_subject(app: tauri::AppHandle, subject: TrackedSubject) -> Result<Vec<TrackedSubject>, String> {
    // 变量：data | 含义：追番映射 | 类型：HashMap<u32, TrackedSubject> | 作用域：save_tracked_subject
    // 本行目的：加载追番数据映射。
    let mut data = load_tracked(&app)?;

    // 本行目的：当条目未被标记任何状态时删除。
    if !subject.watching && !subject.backlog && !subject.watched {
        // 本行目的：移除对应条目。
        data.remove(&subject.id);
    // 本行目的：处理需要保存的情况。
    } else {
        // 本行目的：插入或更新条目。
        data.insert(subject.id, subject);
    // 本行目的：结束保存判断。
    }

    // 本行目的：持久化追番数据。
    persist_tracked(&app, &data)?;

    // 本行目的：返回最新追番列表。
    Ok(data.values().cloned().collect())
// 本行目的：结束保存命令。
}

// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：get_local_weekday | 输入：无 | 输出：本地星期数 | 可能失败：无
// 本行目的：获取本地星期数（周日为 0）。
pub fn get_local_weekday() -> Result<u32, String> {
    // 变量：now | 含义：当前本地时间 | 类型：DateTime<Local> | 作用域：get_local_weekday
    // 本行目的：获取当前本地时间。
    let now = Local::now();
    // 本行目的：返回星期数。
    Ok(now.weekday().num_days_from_sunday())
// 本行目的：结束星期查询命令。
}
