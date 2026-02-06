/**
 * 判断当前运行环境是否为 Tauri。
 * 浏览器环境下不会存在 Tauri 注入的全局对象。
 */
export const isTauri = () => {
  // 服务端渲染或无 window 环境时直接返回 false。
  if (typeof window === "undefined") return false;
  // 通过 Tauri 注入的全局标识判断。
  const w = window as typeof window & {
    __TAURI__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  };
  return Boolean(w.__TAURI__ || w.__TAURI_INTERNALS__);
};
