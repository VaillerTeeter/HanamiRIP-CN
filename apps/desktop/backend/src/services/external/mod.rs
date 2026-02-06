/*
   外部链接相关功能：
   目前只提供“打开链接”的简单命令。
   前端调用这个命令时，会在系统默认浏览器中打开指定 URL。
*/

// 让前端可调用的命令：打开外部链接。
// 成功返回 Ok(())，失败返回错误字符串。
#[tauri::command]
pub fn open_external_link(url: String) -> Result<(), String> {
  open::that(url).map_err(|err| format!("打开外部链接失败: {err}"))
}
