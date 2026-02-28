/* 文件：mod.rs | 用途：提供打开外部链接的命令 | 关键对象：open_external_link */
// 本行目的：标记为可被 Tauri 调用的命令。
#[tauri::command]
/// 函数：open_external_link | 输入：外部链接 URL | 输出：空结果或错误 | 可能失败：系统无法打开链接
// 本行目的：打开外部链接并返回结果。
pub fn open_external_link(url: String) -> Result<(), String> {
    // 变量：url | 含义：待打开的外部链接 | 类型：String | 作用域：open_external_link
    // 本行目的：调用系统打开链接并转换错误信息。
    open::that(url).map_err(|err| format!("打开外部链接失败: {err}"))
// 本行目的：结束命令函数。
}
