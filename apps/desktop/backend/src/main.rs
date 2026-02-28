/* 文件：main.rs | 用途：初始化 Tauri 应用并注册后端命令 | 关键对象：ensure_data_link、main、services */
// 本行目的：在非调试构建时设置 Windows 子系统为 GUI 模式。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 本行目的：声明服务模块以暴露后端功能。
mod services;

// 本行目的：引入 Tauri 管理器 trait 以访问应用句柄功能。
use tauri::Manager;
// 本行目的：引入对话框插件初始化函数并改名方便调用。
use tauri_plugin_dialog::init as dialog_plugin;

/// 函数：ensure_data_link | 输入：app 句柄引用 | 输出：无 | 可能失败：路径解析或文件系统操作失败
// 本行目的：定义数据目录符号链接的创建逻辑。
fn ensure_data_link(app: &tauri::AppHandle) {
    // 本行目的：仅在 Windows 平台编译并执行以下逻辑。
    #[cfg(target_os = "windows")]
    // 本行目的：开始 Windows 平台的作用域块。
    {
        // 本行目的：引入环境变量相关函数。
        use std::env;
        // 本行目的：引入文件系统操作模块。
        use std::fs;
        // 本行目的：引入进程执行模块。
        use std::process::Command;

        // 变量：data_dir | 含义：应用数据目录路径 | 类型：PathBuf | 作用域：ensure_data_link
        // 本行目的：解析并拼接应用数据目录。
        let data_dir = match app.path().app_data_dir() {
            // 本行目的：成功时追加应用专用目录名。
            Ok(dir) => dir.join("hanamirip-cn"),
            // 本行目的：失败时记录错误并提前返回。
            Err(err) => {
                // 本行目的：输出无法解析数据目录的错误信息。
                eprintln!("failed to resolve app data dir: {err}");
                // 本行目的：终止函数执行以避免后续使用无效路径。
                return;
            }
        // 本行目的：结束 match 表达式。
        };

        // 本行目的：尝试创建数据目录的全部父目录。
        if let Err(err) = fs::create_dir_all(&data_dir) {
            // 本行目的：输出创建目录失败的错误信息。
            eprintln!("failed to create app data dir: {err}");
            // 本行目的：终止函数执行以避免后续操作失败。
            return;
        }

        // 变量：install_dir | 含义：当前可执行文件所在目录 | 类型：PathBuf | 作用域：ensure_data_link
        // 本行目的：获取当前可执行文件的安装目录。
        let install_dir = match env::current_exe() {
            // 本行目的：成功时取父目录作为安装目录。
            Ok(exe_path) => exe_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| exe_path.clone()),
            // 本行目的：失败时记录错误并提前返回。
            Err(err) => {
                // 本行目的：输出无法获取可执行路径的错误信息。
                eprintln!("failed to get current exe path: {err}");
                // 本行目的：终止函数执行以避免后续使用无效路径。
                return;
            }
        // 本行目的：结束 match 表达式。
        };

        // 变量：link_path | 含义：安装目录下的 user-data 链接路径 | 类型：PathBuf | 作用域：ensure_data_link
        // 本行目的：构造用户数据链接的目标路径。
        let link_path = install_dir.join("user-data");

        // 本行目的：如果链接已存在则无需重复创建。
        if link_path.exists() {
            // 本行目的：提前返回以避免覆盖已有链接。
            return;
        }

        // 变量：status | 含义：链接命令的执行结果 | 类型：Result<ExitStatus, std::io::Error> | 作用域：ensure_data_link
        // 本行目的：根据平台执行创建符号链接的命令。
        let status = if cfg!(target_os = "windows") {
            // 本行目的：引入 Windows 进程创建标志支持。
            use std::os::windows::process::CommandExt;
            // 变量：CREATE_NO_WINDOW | 含义：隐藏控制台窗口的标志位 | 类型：u32 | 作用域：ensure_data_link
            // 本行目的：定义命令行无窗口执行的常量。
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            // 本行目的：构建并执行 mklink 创建目录联接。
            Command::new("cmd")
                // 本行目的：传入 mklink 命令及参数。
                .args(["/C", "mklink", "/J", link_path.to_string_lossy().as_ref(), data_dir.to_string_lossy().as_ref()])
                // 本行目的：设置进程不创建控制台窗口。
                .creation_flags(CREATE_NO_WINDOW)
                // 本行目的：执行命令并返回状态。
                .status()
        // 本行目的：非 Windows 平台返回默认状态以保持类型一致。
        } else {
            // 本行目的：构造一个默认的成功状态占位。
            Ok(std::process::ExitStatus::default())
        // 本行目的：结束平台分支选择。
        };

        // 本行目的：根据命令执行结果输出日志信息。
        match status {
            // 本行目的：处理命令成功执行的情况。
            Ok(status) => {
                // 本行目的：判断命令退出码是否成功。
                if status.success() {
                    // 本行目的：输出创建链接成功的日志。
                    eprintln!("created data link at: {}", link_path.display());
                // 本行目的：处理命令执行失败的情况。
                } else {
                    // 本行目的：输出创建链接失败的日志。
                    eprintln!("mklink failed: {}", link_path.display());
                // 本行目的：结束退出码判断。
                }
            // 本行目的：处理命令执行出现错误的情况。
            }
            // 本行目的：输出命令无法执行的错误信息。
            Err(err) => {
                // 本行目的：记录执行 mklink 失败的错误细节。
                eprintln!("failed to execute mklink: {err}");
            // 本行目的：结束错误分支。
            }
        // 本行目的：结束 match 处理。
        }
    // 本行目的：结束 Windows 平台作用域块。
    }
// 本行目的：结束数据链接创建函数。
}

/// 函数：main | 输入：无 | 输出：无 | 可能失败：Tauri 运行初始化失败
// 本行目的：定义应用入口并注册命令处理器。
fn main() {
    // 本行目的：创建默认的 Tauri 构建器。
    tauri::Builder::default()
        // 本行目的：注册对话框插件。
        .plugin(dialog_plugin())
        // 本行目的：在应用启动时执行自定义初始化逻辑。
        .setup(|app| {
            // 本行目的：确保用户数据目录链接存在。
            ensure_data_link(app.handle());
            // 本行目的：返回成功结果以继续启动流程。
            Ok(())
        // 本行目的：结束 setup 闭包。
        })
        // 本行目的：注册可被前端调用的命令集合。
        .invoke_handler(tauri::generate_handler![
            // 本行目的：注册季度番剧列表命令。
            services::bangumi::commands::get_season_subjects,
            // 本行目的：注册获取原作信息命令。
            services::bangumi::commands::get_subject_origin,
            // 本行目的：注册已播集数统计命令。
            services::bangumi::commands::get_subject_aired_count,
            // 本行目的：注册筛选标签计算命令。
            services::bangumi::commands::get_subject_filters,
            // 本行目的：注册制作人员列表命令。
            services::bangumi::commands::get_subject_staff,
            // 本行目的：注册角色列表命令。
            services::bangumi::commands::get_subject_characters,
            // 本行目的：注册简介翻译命令。
            services::bangumi::commands::get_subject_summary_cn,
            // 本行目的：注册条目简要信息命令。
            services::bangumi::commands::get_subject_brief,
            // 本行目的：注册搜索站点 HTML 抓取命令。
            services::bangumi::commands::fetch_search_html,
            // 本行目的：注册别名列表命令。
            services::bangumi::commands::get_subject_aliases,
            // 本行目的：注册启动下载任务命令。
            services::torrent::start_torrent_download,
            // 本行目的：注册读取种子元数据命令。
            services::torrent::get_torrent_metadata,
            // 本行目的：注册列出待处理下载命令。
            services::torrent::list_pending_downloads,
            // 本行目的：注册恢复下载任务命令。
            services::torrent::resume_torrent_download,
            // 本行目的：注册丢弃下载任务命令。
            services::torrent::discard_torrent_download,
            // 本行目的：注册获取下载状态命令。
            services::torrent::get_torrent_status,
            // 本行目的：注册完成下载收尾命令。
            services::torrent::finalize_torrent_download,
            // 本行目的：注册解析媒体轨道命令。
            services::media::parse_media_tracks,
            // 本行目的：注册获取媒体文件大小命令。
            services::media::get_media_file_size,
            // 本行目的：注册合成轨道命令。
            services::media::mix_media_tracks,
            // 本行目的：注册暂停下载命令。
            services::torrent::pause_torrent,
            // 本行目的：注册继续下载命令。
            services::torrent::resume_torrent,
            // 本行目的：注册删除下载命令。
            services::torrent::delete_torrent,
            // 本行目的：注册打开外部链接命令。
            services::external::open_external_link,
            // 本行目的：注册列出追番记录命令。
            services::storage::list_tracked_subjects,
            // 本行目的：注册保存追番记录命令。
            services::storage::save_tracked_subject,
            // 本行目的：注册获取本地星期命令。
            services::storage::get_local_weekday
        // 本行目的：结束命令注册列表。
        ])
        // 本行目的：运行 Tauri 应用并使用生成的上下文。
        .run(tauri::generate_context!())
        // 本行目的：在应用启动失败时直接抛出错误。
        .expect("error while running tauri application");
// 本行目的：结束应用入口函数。
}
