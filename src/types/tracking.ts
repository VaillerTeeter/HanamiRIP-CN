import type { MonthAnime } from "./anime";

export type ItemStatus = { watching: boolean; backlog: boolean; watched: boolean };

export type TrackedItem = MonthAnime & ItemStatus;

export type StatusKey = "watching" | "backlog" | "watched" | null;
