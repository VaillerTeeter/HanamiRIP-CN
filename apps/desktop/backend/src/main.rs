// 在 Windows release 下隐藏控制台窗口（避免启动时弹出黑框）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 后端服务模块入口（Bangumi/下载/媒体/存储等）。
mod services;

use tauri::Manager;
use tauri_plugin_dialog::init as dialog_plugin;

// 在 Windows 下创建一个指向数据目录的“junction”。
// 目的：让用户在安装目录下看到 user-data，方便查找/迁移。
fn ensure_data_link(app: &tauri::AppHandle) {
  #[cfg(target_os = "windows")]
  {
    use std::env;
    use std::fs;
    use std::process::Command;

    // 应用数据目录（系统推荐的位置）。
    let data_dir = match app.path().app_data_dir() {
      Ok(dir) => dir.join("hanamirip-cn"),
      Err(err) => {
        eprintln!("failed to resolve app data dir: {err}");
        return;
      }
    };

    // 确保数据目录存在。
    if let Err(err) = fs::create_dir_all(&data_dir) {
      eprintln!("failed to create app data dir: {err}");
      return;
    }

    // 获取 exe 所在目录（真实安装位置）。
    let install_dir = match env::current_exe() {
      Ok(exe_path) => exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| exe_path.clone()),
      Err(err) => {
        eprintln!("failed to get current exe path: {err}");
        return;
      }
    };

    // 想在安装目录下创建的链接路径。
    let link_path = install_dir.join("user-data");

    // 如果已存在，跳过创建。
    if link_path.exists() {
      return;
    }

    // 尝试创建 junction（目录链接）。
    let status = if cfg!(target_os = "windows") {
      use std::os::windows::process::CommandExt;
      const CREATE_NO_WINDOW: u32 = 0x08000000;

      Command::new("cmd")
        .args([
          "/C",
          "mklink",
          "/J",
          link_path.to_string_lossy().as_ref(),
          data_dir.to_string_lossy().as_ref(),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
    } else {
      Ok(std::process::ExitStatus::default())
    };

    // 根据执行结果打印日志。
    match status {
      Ok(status) => {
        if status.success() {
          eprintln!("created data link at: {}", link_path.display());
        } else {
          eprintln!("mklink failed: {}", link_path.display());
        }
      }
      Err(err) => {
        eprintln!("failed to execute mklink: {err}");
      }
    }
  }
}

// 程序入口：初始化插件、注册命令、启动应用。
fn main() {
  tauri::Builder::default()
    .plugin(dialog_plugin())
    .setup(|app| {
      ensure_data_link(app.handle());
      Ok(())
    })
    // 注册可被前端调用的命令列表。
    .invoke_handler(tauri::generate_handler![
      services::bangumi::commands::get_season_subjects,
      services::bangumi::commands::get_subject_origin,
      services::bangumi::commands::get_subject_aired_count,
      services::bangumi::commands::get_subject_filters,
      services::bangumi::commands::get_subject_staff,
      services::bangumi::commands::get_subject_characters,
      services::bangumi::commands::get_subject_summary_cn,
      services::bangumi::commands::get_subject_brief,
      services::bangumi::commands::fetch_search_html,
      services::bangumi::commands::get_subject_aliases,
      services::torrent::start_torrent_download,
      services::torrent::get_torrent_status,
      services::torrent::finalize_torrent_download,
      services::media::parse_media_tracks,
      services::media::get_media_file_size,
      services::media::mix_media_tracks,
      services::torrent::pause_torrent,
      services::torrent::resume_torrent,
      services::torrent::delete_torrent,
      services::external::open_external_link,
      services::storage::list_tracked_subjects,
      services::storage::save_tracked_subject
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
