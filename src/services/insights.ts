import { invoke } from "@tauri-apps/api/core";
import type { WatchStatus } from "./watch";

export type Count = { label: string; count: number };
export type TermCount = Count & { id: number };
export type Statistics = {
  seriesCount: number;
  contentCount: number;
  favorites: number;
  activeEpisodes: number;
  pastEpisodes: number;
  totalEpisodes: number;
  completedViews: number;
  average: number | null;
  statuses: Count[];
  genres: TermCount[];
  categories: TermCount[];
  ranking: { id: number; title: string; average: number; coverKey: string | null }[];
  ratings: Count[];
};

export type ContentSelection =
  | { kind: "status"; value: WatchStatus; label: string }
  | { kind: "category" | "rating"; value: number; label: string };

export type DashboardSelection = ContentSelection | { kind: "genre"; value: number; label: string };

export type ContinueItem = {
  id: number;
  seriesId: number;
  seriesTitle: string;
  title: string;
  category: string;
  coverKey: string | null;
  sessionId: number;
  watchedEpisodes: number;
  totalEpisodes: number | null;
  ratingTenths: number | null;
};

export const readStatistics = () => invoke<Statistics>("library_statistics");
export const readContinueWatching = () => invoke<ContinueItem[]>("continue_watching");
