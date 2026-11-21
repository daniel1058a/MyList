import { StatusFilter } from "../../components/StatusFilter";
import { useLocale } from "../../locales/locale";
import type { Genre } from "../../services/catalog";
import type { CatalogFilters } from "../../services/preferences";

export function CatalogFilterBar({ filters, genres, favoritesView, onChange, onReset }: {
  filters: CatalogFilters;
  genres: Genre[];
  favoritesView: boolean;
  onChange: (filters: CatalogFilters) => void;
  onReset: () => void;
}) {
  const { t } = useLocale();
  const copy = t.filters;
  return (
    <section className="filter-bar" aria-label={copy.label}>
      <label className="field filter-bar__search">{copy.search}
        <input type="search" placeholder={copy.searchHint} maxLength={200} value={filters.query}
          onChange={(event) => onChange({ ...filters, query: event.target.value })} />
      </label>
      {genres.length > 0 && <details className="filter-bar__genres">
        <summary>{copy.genres} ({filters.genreIds.length})</summary>
        <div className="filter-bar__choices">
          {genres.map((genre) => <label className="genre-choice" key={genre.id}>
            <input type="checkbox" checked={filters.genreIds.includes(genre.id)} onChange={(event) => {
              const genreIds = event.target.checked ? [...filters.genreIds, genre.id] : filters.genreIds.filter((id) => id !== genre.id);
              onChange({ ...filters, genreIds });
            }} />{genre.name}
          </label>)}
        </div>
        <label className="field">{copy.genreMode}
          <select aria-label={copy.genreMode} value={filters.genreMode} onChange={(event) => onChange({ ...filters, genreMode: event.target.value as CatalogFilters["genreMode"] })}>
            <option value="all">{copy.allGenres}</option>
            <option value="any">{copy.anyGenre}</option>
          </select>
        </label>
      </details>}
      <div className="filter-bar__row">
        <StatusFilter value={filters.status} onChange={(status) => onChange({ ...filters, status })} />
        <label className="field">{copy.sortBy}
          <select aria-label={copy.sortBy} value={filters.sortBy} onChange={(event) => onChange({ ...filters, sortBy: event.target.value as CatalogFilters["sortBy"] })}>
            {Object.entries(copy.sorts).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
          </select>
        </label>
        <label className="field">{copy.direction}
          <select aria-label={copy.direction} value={filters.direction} onChange={(event) => onChange({ ...filters, direction: event.target.value as CatalogFilters["direction"] })}>
            <option value="asc">{copy.asc}</option>
            <option value="desc">{copy.desc}</option>
          </select>
        </label>
        <label className="genre-choice">
          <input type="checkbox" checked={favoritesView || filters.favoriteOnly} disabled={favoritesView}
            onChange={(event) => onChange({ ...filters, favoriteOnly: event.target.checked })} />{copy.favorites}
        </label>
        <button type="button" onClick={onReset}>{copy.reset}</button>
      </div>
    </section>
  );
}
