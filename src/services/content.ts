import { invoke } from "@tauri-apps/api/core";
import type { CoverChange } from "./catalog";
import type { WatchSnapshot } from "./watch";

export type Category = { id: number; name: string };
export type Content = {
  id: number;
  categoryId: number;
  title: string;
  coverKey: string | null;
  description: string;
  opinion: string;
  releaseYear: number | null;
  watch: WatchSnapshot;
};
export type SeriesContents = { items: Content[]; categories: Category[]; ratingAverage: number | null; updatedAt: string };
type SaveContent = Pick<Content, "title" | "description" | "opinion" | "releaseYear"> & {
  id: number | null;
  seriesId: number;
  category: string;
  cover: CoverChange;
};
export type Placement = "before" | "after";

export const listContents = (seriesId: number) => invoke<SeriesContents>("content_list", { seriesId });
export const saveContent = (input: SaveContent) => invoke<SeriesContents>("content_save", { input });
export const deleteContent = (seriesId: number, id: number) =>
  invoke<SeriesContents>("content_delete", { seriesId, id });
export const reorderContent = (seriesId: number, id: number, targetId: number, placement: Placement) =>
  invoke<SeriesContents>("content_reorder", { seriesId, id, targetId, placement });
