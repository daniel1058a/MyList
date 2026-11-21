import { describe, expect, it } from "vitest";
import { filterCatalog, filterContents, indexCatalog } from "../src/features/library/catalogFilters";
import type { Series } from "../src/services/catalog";
import type { Content } from "../src/services/content";
import { defaultCatalogFilters, type CatalogFilters } from "../src/services/preferences";
import type { WatchStatus } from "../src/services/watch";

function series(id: number, title: string, extra: Partial<Series> = {}): Series {
  return { id, title, coverKey: null, description: "", panoramaKey: null, favorite: false, genreIds: [], contents: [],
    ratingAverage: null, createdAt: "", updatedAt: "", ...extra };
}

const index = indexCatalog([
  series(1, "Órbita", { favorite: true, genreIds: [1, 2], ratingAverage: 8, createdAt: "2024", updatedAt: "2025",
    contents: [{ title: "Película: Éclipse", status: "to_watch", categoryId: 1, ratingTenths: null },
      { title: "Éclipse II", status: "finished", categoryId: 2, ratingTenths: 99 }] }),
  series(2, "Estación", { favorite: true, genreIds: [2], ratingAverage: 9, createdAt: "2025", updatedAt: "2024",
    contents: [{ title: "ÉCLIPSE rojo", status: "watching", categoryId: 1, ratingTenths: 100 }] }),
  series(3, "Álbum", { genreIds: [1], createdAt: "2023", updatedAt: "2026" }),
  series(4, "Orbita", { genreIds: [2], ratingAverage: 8, createdAt: "2024", updatedAt: "2025",
    contents: [{ title: "Éclipse I", status: "finished", categoryId: 1, ratingTenths: null }] }),
]);

const ids = (filters: Partial<CatalogFilters> = {}, favorites = false) =>
  filterCatalog(index, { ...defaultCatalogFilters, ...filters }, favorites).map((item) => item.id);

describe("Fase 4", () => {
  it("matches partial titles and returns a series only once for multiple matching children", () => {
    expect(ids({ query: "EcLiP" })).toEqual([2, 1, 4]);
    expect(ids({ query: " ORB " })).toEqual([1, 4]);
    expect(ids({ query: "pelicula: ecl" })).toEqual([1]);
    expect(ids({ query: "%" })).toEqual([]);
  });

  it("combines independent filters and matches the active state of at least one child", () => {
    expect(ids({ genreIds: [1, 2], genreMode: "all" })).toEqual([1]);
    expect(ids({ query: "eclip", genreIds: [1, 2], genreMode: "any", favoriteOnly: true, status: "watching" })).toEqual([2]);
    expect(ids({ query: "pelicula", genreIds: [1, 2], status: "finished" }, true)).toEqual([1]);
    expect(ids({ status: "to_watch" })).toEqual([1]);
    expect(ids({ query: "álbum", status: "to_watch" })).toEqual([]);
    expect(ids({ genreMode: "any" })).toEqual([3, 2, 1, 4]);
  });

  it("orders every criterion in both directions, with a stable ID tie and empty scores last", () => {
    expect(ids()).toEqual([3, 2, 1, 4]);
    expect(ids({ direction: "desc" })).toEqual([1, 4, 2, 3]);
    expect(ids({ sortBy: "rating" })).toEqual([1, 4, 2, 3]);
    expect(ids({ sortBy: "rating", direction: "desc" })).toEqual([2, 1, 4, 3]);
    expect(ids({ sortBy: "created" })).toEqual([3, 1, 4, 2]);
    expect(ids({ sortBy: "created", direction: "desc" })).toEqual([2, 1, 4, 3]);
    expect(ids({ sortBy: "updated" })).toEqual([2, 1, 4, 3]);
    expect(ids({ sortBy: "updated", direction: "desc" })).toEqual([3, 1, 4, 2]);
  });

  it("filters contents without changing the manual order or the source array", () => {
    const content = (id: number, categoryId: number, status: WatchStatus): Content => ({
      id, categoryId, title: "Contenido " + id, coverKey: null, description: "", opinion: "", releaseYear: null,
      watch: { completedViews: 0, ratingTenths: null, totalEpisodes: null, session: {
        id, status, watchedEpisodes: 0, startedOn: null, finishedOn: null, completed: false, active: true,
      } },
    });
    const items = [content(3, 1, "watching"), content(2, 2, "to_watch"), content(1, 1, "watching")];
    expect(filterContents(items, { categoryId: 1, status: "watching" }).map((item) => item.id)).toEqual([3, 1]);
    expect(filterContents(items, { categoryId: 2, status: "finished" })).toEqual([]);
    expect(items.map((item) => item.id)).toEqual([3, 2, 1]);
  });

  it("matches dashboard category, grade and state against the same child content", () => {
    const matches = (selection: Parameters<typeof filterCatalog>[3], status: WatchStatus | null = null) =>
      filterCatalog(index, { ...defaultCatalogFilters, status }, false, selection).map((item) => item.id);
    expect(matches({ kind: "category", value: 1, label: "Película" }, "finished")).toEqual([4]);
    expect(matches({ kind: "rating", value: 9, label: "9–9,9" })).toEqual([1]);
    expect(matches({ kind: "rating", value: 10, label: "10" })).toEqual([2]);
    expect(matches({ kind: "category", value: 2, label: "Otra" }, "watching")).toEqual([]);
  });
});
