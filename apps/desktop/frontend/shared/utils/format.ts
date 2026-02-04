const weekdayLabels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

export const formatRating = (value?: number | null) => (typeof value === "number" ? value.toFixed(1) : "0.0");

export const formatStars = (value?: number | null) => {
  if (typeof value !== "number") return "☆☆☆☆☆";
  const normalized = Math.max(0, Math.min(10, value));
  const filled = Math.round(normalized / 2);
  return "★".repeat(filled) + "☆".repeat(5 - filled);
};

export const formatAirDate = (value?: string) => {
  if (!value) return "未知";
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return `${value}（${weekdayLabels[parsed.getDay()]}）`;
};

export const formatBytes = (value?: number) => {
  if (value == null || Number.isNaN(value)) return "-";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = value;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
};

export const parseSpeedToBps = (value?: string) => {
  if (!value) return 0;
  const match = value.match(/([\d.]+)\s*([a-zA-Z/]+)?/);
  if (!match) return 0;
  const num = Number.parseFloat(match[1]);
  if (!Number.isFinite(num)) return 0;
  const unitRaw = (match[2] || "B").replace(/\s*/g, "").replace(/\/s/i, "").toUpperCase();
  const unit = unitRaw.endsWith("/S") ? unitRaw.slice(0, -2) : unitRaw;
  const factorMap: Record<string, number> = {
    B: 1,
    KB: 1024,
    KIB: 1024,
    MB: 1024 ** 2,
    MIB: 1024 ** 2,
    GB: 1024 ** 3,
    GIB: 1024 ** 3,
    TB: 1024 ** 4,
    TIB: 1024 ** 4,
  };
  const factor = factorMap[unit] ?? 1;
  return num * factor;
};

export const formatSpeed = (bps: number) => {
  if (!Number.isFinite(bps) || bps <= 0) return "0 B/s";
  const units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
  let size = bps;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
};
