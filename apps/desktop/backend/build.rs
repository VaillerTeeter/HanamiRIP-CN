/* 文件：build.rs | 用途：根据目标平台设置 Tauri 打包资源并触发构建 | 关键对象：main、target、TAURI_CONFIG */
/// 函数：main | 输入：无 | 输出：无 | 可能失败：环境变量读取异常但被默认值兜底
// 本行目的：定义构建脚本入口函数。
fn main() {
    // 变量：target | 含义：目标编译三元组的小写字符串 | 类型：String | 作用域：main
    // 本行目的：读取 Tauri/编译器目标三元组并规范为小写，便于平台判断。
    let target = std::env::var("TAURI_ENV_TARGET_TRIPLE").or_else(|_| std::env::var("TARGET")).unwrap_or_default().to_lowercase();

    // 本行目的：判断是否为 Windows 目标以设置对应的资源清单。
    if target.contains("windows") {
        // 本行目的：设置 Tauri 构建时的资源清单环境变量。
        std::env::set_var(
            // 本行目的：指定要写入的环境变量键名。
            "TAURI_CONFIG",
            // 本行目的：为 Windows 目标配置可执行工具资源路径。
            r#"{ "bundle": { "resources": ["../public/tools/ffprobe.exe", "../public/tools/ffmpeg.exe", "../public/tools/mkvmerge.exe", "../public/tools/mkvinfo.exe"] } }"#,
        );
    // 本行目的：判断是否为 Linux 目标以设置对应的资源清单。
    } else if target.contains("linux") {
        // 本行目的：设置 Tauri 构建时的资源清单环境变量。
        std::env::set_var(
            // 本行目的：指定要写入的环境变量键名。
            "TAURI_CONFIG",
            // 本行目的：为 Linux 目标配置可执行工具资源路径。
            r#"{ "bundle": { "resources": ["../public/tools/ffprobe", "../public/tools/ffmpeg", "../public/tools/mkvmerge", "../public/tools/mkvinfo"] } }"#,
        );
    // 本行目的：结束平台分支判断。
    }

    // 本行目的：调用 Tauri 构建辅助逻辑生成必要构建产物。
    tauri_build::build()
// 本行目的：结束构建脚本入口函数。
}
