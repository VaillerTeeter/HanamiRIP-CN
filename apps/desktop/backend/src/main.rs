#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod services;

use tauri::Manager;
use tauri_plugin_dialog::init as dialog_plugin;

fn ensure_data_link(app: &tauri::AppHandle) {
  #[cfg(target_os = "windows")]
  {
    use std::env;
    use std::fs;
    use std::process::Command;

    let data_dir = match app.path().app_data_dir() {
      Ok(dir) => dir.join("hanamirip-cn"),
      Err(err) => {
        eprintln!("failed to resolve app data dir: {err}");
        return;
      }
    };

    if let Err(err) = fs::create_dir_all(&data_dir) {
      eprintln!("failed to create app data dir: {err}");
      return;
    }

    // 获取exe所在目录（真实安装位置）
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

    let link_path = install_dir.join("user-data");

    // 如果已存在，跳过创建
    if link_path.exists() {
      return;
    }

    // 尝试创建 junction
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

fn main() {
  tauri::Builder::default()
    .plugin(dialog_plugin())
    .setup(|app| {
      ensure_data_link(app.handle());
      Ok(())
    })
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
