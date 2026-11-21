import type { CatalogContent, Series } from "../../services/catalog";
import type { Content } from "../../services/content";
import type { ContentSelection, DashboardSelection } from "../../services/insights";
import type { CatalogFilters, ContentFilters } from "../../services/preferences";
import type { Language } from "../../locales/locale";

const normalize = (text: string) => text.normalize("NFD").toLowerCase()
  .replace(/\p{M}/gu, "").trim().replace(/\s+/g, " ");
const titles = { es: new Intl.Collator("es"), en: new Intl.Collator("en") };

export function indexCatalog(series: Series[]) {
  return series.map((item) => ({
    item,
    title: normalize(item.title),
    contentTitles: item.contents.map((content) => normalize(content.title)),
    statuses: new Set(item.contents.map((content) => content.status)),
  }));
}

function matchesSelection(content: Pick<CatalogContent, "status" | "categoryId" | "ratingTenths">, selection: ContentSelection) {
  switch (selection.kind) {
    case "status": return content.status === selection.value;
    case "category": return content.categoryId === selection.value;
    case "rating": return content.ratingTenths != null && Math.floor(content.ratingTenths / 10) === selection.value;
  }
}

export function filterCatalog(index: ReturnType<typeof indexCatalog>, filters: CatalogFilters, favoritesView: boolean,
  selection?: DashboardSelection, language: Language = "es") {
  const query = normalize(filters.query);
  const results = index.filter(({ item, title, contentTitles, statuses }) => {
    if ((favoritesView || filters.favoriteOnly) && !item.favorite) return false;
    if (filters.status !== null && !statuses.has(filters.status)) return false;
    if (selection?.kind === "genre" && !item.genreIds.includes(selection.value)) return false;
    if (selection && selection.kind !== "genre" && !item.contents.some((content) =>
      matchesSelection(content, selection) && (filters.status === null || content.status === filters.status))) return false;
    if (query && !title.includes(query) && !contentTitles.some((content) => content.includes(query))) return false;
    if (filters.genreIds.length === 0) return true;
    const matches = (id: number) => item.genreIds.includes(id);
    return filters.genreMode === "all" ? filters.genreIds.every(matches) : filters.genreIds.some(matches);
  });

  const compare = (left: string, right: string) => left < right ? -1 : Number(left > right);
  const direction = filters.direction === "asc" ? 1 : -1;
  return results.sort((a, b) => {
    let order: number;
    switch (filters.sortBy) {
      case "rating": {
        const left = a.item.ratingAverage;
        const right = b.item.ratingAverage;
        if (left == null || right == null) {
          return Number(left == null) - Number(right == null) || a.item.id - b.item.id;
        }
        order = left - right;
        break;
      }
      case "created": order = compare(a.item.createdAt, b.item.createdAt); break;
      case "updated": order = compare(a.item.updatedAt, b.item.updatedAt); break;
      case "title": order = titles[language].compare(a.title, b.title); break;
    }
    return direction * order || a.item.id - b.item.id;
  }).map(({ item }) => item);
}

export function filterContents(contents: Content[], { categoryId, status }: ContentFilters, selection?: ContentSelection) {
  return contents.filter((content) => (categoryId === null || categoryId === content.categoryId)
    && (status === null || status === content.watch.session.status)
    && (!selection || matchesSelection({
      status: content.watch.session.status, categoryId: content.categoryId, ratingTenths: content.watch.ratingTenths,
    }, selection)));
}
