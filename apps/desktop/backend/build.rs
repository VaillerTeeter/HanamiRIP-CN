/*
   构建脚本：
   在打包时为不同平台注入 Tauri 的资源配置，
   让 ffprobe/ffmpeg/mkvmerge 等工具被正确打包进应用。
*/

fn main() {
  // 读取目标平台三元组（TAURI_ENV_TARGET_TRIPLE 或 TARGET）。
  let target = std::env::var("TAURI_ENV_TARGET_TRIPLE")
    .or_else(|_| std::env::var("TARGET"))
    .unwrap_or_default()
    .to_lowercase();

  // 不同平台使用不同的工具文件名（Windows 带 .exe）。
  if target.contains("windows") {
    std::env::set_var(
      "TAURI_CONFIG",
      r#"{ "bundle": { "resources": ["../public/tools/ffprobe.exe", "../public/tools/ffmpeg.exe", "../public/tools/mkvmerge.exe", "../public/tools/mkvinfo.exe"] } }"#,
    );
  } else if target.contains("linux") {
    std::env::set_var(
      "TAURI_CONFIG",
      r#"{ "bundle": { "resources": ["../public/tools/ffprobe", "../public/tools/ffmpeg", "../public/tools/mkvmerge", "../public/tools/mkvinfo"] } }"#,
    );
  }

  // 继续执行 Tauri 的标准构建流程。
  tauri_build::build()
}
