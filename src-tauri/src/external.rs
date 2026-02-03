#[tauri::command]
pub fn open_external_link(url: String) -> Result<(), String> {
  open::that(url).map_err(|err| format!("打开外部链接失败: {err}"))
}