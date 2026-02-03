#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bangumi;
mod external;
mod media;
mod storage;
mod torrent;

use tauri_plugin_dialog::init as dialog_plugin;

fn main() {
  tauri::Builder::default()
    .plugin(dialog_plugin())
    .invoke_handler(tauri::generate_handler![
      bangumi::commands::get_season_subjects,
      bangumi::commands::get_subject_origin,
      bangumi::commands::get_subject_aired_count,
      bangumi::commands::get_subject_filters,
      bangumi::commands::get_subject_staff,
      bangumi::commands::get_subject_characters,
      bangumi::commands::get_subject_summary_cn,
      bangumi::commands::get_subject_brief,
      bangumi::commands::fetch_search_html,
      bangumi::commands::get_subject_aliases,
      torrent::start_torrent_download,
      torrent::get_torrent_status,
      torrent::finalize_torrent_download,
      media::parse_media_tracks,
      media::get_media_file_size,
      media::mix_media_tracks,
      torrent::pause_torrent,
      torrent::resume_torrent,
      torrent::delete_torrent,
      external::open_external_link,
      storage::list_tracked_subjects,
      storage::save_tracked_subject
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
