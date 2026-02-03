import { invoke } from "@tauri-apps/api/core";

export const useExternalLink = () => {
  const openExternalLink = async (url?: string | null) => {
    if (!url) return;
    try {
      await invoke("open_external_link", { url });
    } catch (err) {
      console.error("openExternalLink failed", err);
      window.open(url, "_blank");
    }
  };

  return { openExternalLink };
};
