/*
  外部链接工具：优先走 Tauri 后端打开链接，失败时退回浏览器打开。
*/
import { invoke } from "@tauri-apps/api/core";

/**
 * 提供统一的“打开外部链接”方法。
 */
export const useExternalLink = () => {
  // 实际执行的打开逻辑。
  const openExternalLink = async (url?: string | null) => {
    // 空地址直接忽略，避免错误。
    if (!url) return;
    try {
      // 优先走 Tauri 命令（更符合桌面应用行为）。
      await invoke("open_external_link", { url });
    } catch (err) {
      // 兜底：浏览器新标签打开。
      console.error("openExternalLink failed", err);
      window.open(url, "_blank");
    }
  };

  return { openExternalLink };
};
