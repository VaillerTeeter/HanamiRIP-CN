fn main() {
  let target = std::env::var("TAURI_ENV_TARGET_TRIPLE")
    .or_else(|_| std::env::var("TARGET"))
    .unwrap_or_default()
    .to_lowercase();

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

  tauri_build::build()
}
