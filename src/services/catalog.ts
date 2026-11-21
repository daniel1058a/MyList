import { invoke } from "@tauri-apps/api/core";
import { es } from "../locales/es";
import type { WatchStatus } from "./watch";

export type Genre = { id: number; name: string };
export type CatalogContent = { title: string; status: WatchStatus; categoryId: number; ratingTenths: number | null };
export type Series = {
  id: number;
  title: string;
  coverKey: string | null;
  description: string;
  panoramaKey: string | null;
  favorite: boolean;
  genreIds: number[];
  ratingAverage: number | null;
  createdAt: string;
  updatedAt: string;
  contents: CatalogContent[];
};
export type Catalog = { series: Series[]; genres: Genre[] };

export type CoverChange = { action: "keep" | "remove" } | { action: "replace"; data: string };
type SaveSeries = {
  id: number | null;
  title: string;
  genres: string[];
  description: string;
  cover: CoverChange;
  panorama: CoverChange;
  allowDuplicate: boolean;
};

export const listCatalog = () => invoke<Catalog>("catalog_list");
export const saveSeries = (input: SaveSeries) => invoke<Catalog>("series_save", { input });
export const setFavorite = (id: number, favorite: boolean) =>
  invoke<Catalog>("series_favorite", { id, favorite });
export const deleteSeries = (id: number) => invoke<Catalog>("series_delete", { id });

const { errors } = es.library;
export type CatalogError = keyof typeof errors;

export function catalogError(error: unknown): CatalogError {
  if (typeof error === "object" && error !== null && "code" in error
    && typeof error.code === "string" && Object.hasOwn(errors, error.code)) {
    return error.code as CatalogError;
  }
  return "catalog_unavailable";
}
