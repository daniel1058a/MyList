import { invoke } from "@tauri-apps/api/core";
import type { SeriesContents } from "./content";

export type WatchStatus = "to_watch" | "watching" | "finished";
type Progress = { totalEpisodes: number | null; watchedEpisodes: number };

export type WatchSession = {
  id: number;
  watchedEpisodes: number;
  status: WatchStatus;
  startedOn: string | null;
  finishedOn: string | null;
  completed: boolean;
  active: boolean;
};

export type WatchSnapshot = {
  session: WatchSession;
  totalEpisodes: number | null;
  ratingTenths: number | null;
  completedViews: number;
};

export type WatchAction =
  | { kind: "step"; delta: -1 | 1 }
  | { kind: "set_status"; status: WatchStatus; resetProgress: boolean }
  | { kind: "edit"; progress: Progress | null; ratingTenths: number | null; startedOn: string | null; finishedOn: string | null }
  | { kind: "rewatch" };

export const updateWatch = (seriesId: number, contentId: number, sessionId: number, action: WatchAction) =>
  invoke<SeriesContents>("watch_update", { input: { seriesId, contentId, sessionId, action } });

export const watchHistory = (seriesId: number, contentId: number) =>
  invoke<WatchSession[]>("watch_history", { seriesId, contentId });
