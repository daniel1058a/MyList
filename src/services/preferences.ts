import { invoke } from "@tauri-apps/api/core";
import type { WatchStatus } from "./watch";
import type { Language } from "../locales/locale";

export type ContentFilters = { categoryId: number | null; status: WatchStatus | null };
export type CatalogFilters = {
  query: string;
  genreIds: number[];
  genreMode: "all" | "any";
  status: WatchStatus | null;
  favoriteOnly: boolean;
  sortBy: "title" | "rating" | "created" | "updated";
  direction: "asc" | "desc";
};

export type VisualPreferences = { theme: "dark" | "light"; coverSize: "small" | "medium" | "large"; language: Language };
export type Preferences = CatalogFilters & VisualPreferences & { contentFilters: Record<string, ContentFilters> };

export const defaultCatalogFilters: CatalogFilters = {
  query: "", genreIds: [], genreMode: "all", status: null, favoriteOnly: false,
  sortBy: "title", direction: "asc",
};
export const defaultContentFilters: ContentFilters = { categoryId: null, status: null };
export const defaultPreferences: Preferences = {
  ...defaultCatalogFilters, contentFilters: {}, theme: "dark", coverSize: "medium", language: "es",
};

export const accessPreferences = (value: Preferences | null) =>
  invoke<Preferences>("catalog_preferences", { value });
