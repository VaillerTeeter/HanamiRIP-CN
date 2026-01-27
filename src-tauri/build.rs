fn main() {
  let target = std::env::var("TAURI_ENV_TARGET_TRIPLE")
    .or_else(|_| std::env::var("TARGET"))
    .unwrap_or_default()
    .to_lowercase();

  if target.contains("windows") {
    std::env::set_var(
      "TAURI_CONFIG",
      r#"{ "bundle": { "resources": ["bin/ffprobe.exe", "bin/ffmpeg.exe", "bin/mkvmerge.exe", "bin/mkvinfo.exe"] } }"#,
    );
  } else if target.contains("linux") {
    std::env::set_var(
      "TAURI_CONFIG",
      r#"{ "bundle": { "resources": ["bin/ffprobe", "bin/ffmpeg", "bin/mkvmerge", "bin/mkvinfo"] } }"#,
    );
  }

  tauri_build::build()
}
