将内置的 FFmpeg/MKVToolNix 工具放在此目录中：
- Windows: ffmpeg.exe、ffprobe.exe、mkvmerge.exe、mkvinfo.exe
- Linux: ffmpeg、ffprobe、mkvmerge、mkvinfo

脚本会自动下载：
- Windows：运行 scripts/setup-env.ps1
- Ubuntu：运行 scripts/setup-env.sh

这些文件会在构建时随应用打包，并用于轨道解析/分离/合成。
