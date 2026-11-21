import { useEffect, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { Modal } from "../../components/Modal";
import type { View } from "../../components/Sidebar";
import { selectionLabel, useLocale } from "../../locales/locale";
import { catalogError, deleteSeries, listCatalog, setFavorite, type Catalog, type CatalogError, type Series } from "../../services/catalog";
import { Cover } from "../../components/Cover";
import { SeriesContents } from "../content/SeriesContents";
import { defaultCatalogFilters, defaultContentFilters, type CatalogFilters, type ContentFilters } from "../../services/preferences";
import { CatalogFilterBar } from "./CatalogFilterBar";
import { filterCatalog, indexCatalog } from "./catalogFilters";
import type { useCatalogPreferences } from "./useCatalogPreferences";
import { SeriesForm } from "./SeriesForm";
import { Settings } from "../settings/Settings";
import type { Taxonomy } from "../../services/taxonomy";
import { ContinueWatching } from "../insights/ContinueWatching";
import { StatisticsView } from "../insights/StatisticsView";
import type { ContinueItem, DashboardSelection } from "../../services/insights";
import type { SeriesContents as Contents } from "../../services/content";
import { updateWatch } from "../../services/watch";

type Dialog = { kind: "edit"; series: Series | null } | { kind: "delete"; series: Series };

export function Library({ view, activeSeries, onActiveSeries, onNavigate, settings, backupsPanel }: {
  view: View;
  activeSeries: number | null;
  onActiveSeries: (id: number | null) => void;
  onNavigate: (view: View) => void;
  settings: ReturnType<typeof useCatalogPreferences>;
  backupsPanel: ReactNode;
}) {
  const locale = useLocale();
  const { t, formatRating, language } = locale;
  const copy = t.library;
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [error, setError] = useState<CatalogError | null>(null);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  const [busy, setBusy] = useState(false);
  const [attempt, setAttempt] = useState(0);
  const [tabs, setTabs] = useState<number[]>([]);
  const [entryContent, setEntryContent] = useState<number | null>(null);
  const [stepError, setStepError] = useState<CatalogError | null>(null);
  const [exploration, setExploration] = useState<{ selection: DashboardSelection; filters: CatalogFilters } | null>(null);
  const [scopedContents, setScopedContents] = useState<{ seriesId: number; filters: ContentFilters } | null>(null);
  const focusTitle = useRef<"library" | "series" | null>(null);
  const favorites = view === "favorites";
  const { preferences, error: preferenceError, update: updatePreferences, retry: retryPreferences } = settings;

  useEffect(() => {
    let active = true;
    void listCatalog().then((next) => {
      if (active) setCatalog(next);
    }).catch((reason: unknown) => {
      if (active) setError(catalogError(reason));
    });
    return () => { active = false; };
  }, [attempt]);

  async function change(action: () => Promise<Catalog>) {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      const next = await action();
      setCatalog(next);
      const ids = new Set(next.series.map((series) => series.id));
      setTabs((current) => current.filter((id) => ids.has(id)));
      if (activeSeries !== null && !ids.has(activeSeries)) onActiveSeries(null);
      setDialog(null);
    } catch (reason) {
      setError(catalogError(reason));
    } finally {
      setBusy(false);
    }
  }

  const index = useMemo(() => indexCatalog(catalog?.series ?? []), [catalog]);
  const exploring = view === "library" ? exploration : null;
  const catalogFilters = exploring?.filters ?? preferences;
  const items = catalog && catalogFilters && (view === "library" || favorites) && activeSeries === null
    ? filterCatalog(index, catalogFilters, favorites, exploring?.selection, language) : null;
  const filteredEmpty = catalog?.series.some((series) => !favorites || series.favorite);
  const genreNames = new Map(catalog?.genres.map((genre) => [genre.id, genre.name]));
  const byId = new Map(catalog?.series.map((series) => [series.id, series]));
  const opened = activeSeries === null ? undefined : byId.get(activeSeries);
  const contentSelection = exploring && opened && scopedContents?.seriesId === opened.id && exploring.selection.kind !== "genre"
    ? exploring.selection : undefined;

  useLayoutEffect(() => {
    const target = focusTitle.current;
    if (target && (target === "library" ? view === "library" && !opened : !!opened)) {
      document.getElementById(target + "-title")?.focus();
      focusTitle.current = null;
    }
  }, [view, opened, exploration]);

  function explore(selection: DashboardSelection) {
    setExploration({ selection, filters: defaultCatalogFilters });
    setScopedContents(null);
    focusTitle.current = "library";
    onNavigate("library");
  }

  function openSeries(id: number, contentId: number | null = null, fromCatalog = false) {
    setScopedContents(fromCatalog && exploring && exploring.selection.kind !== "genre" ? {
      seriesId: id, filters: { ...defaultContentFilters, status: exploring.filters.status },
    } : null);
    setEntryContent(contentId);
    setTabs((current) => current.includes(id) ? current : [...current, id]);
    onActiveSeries(id);
  }

  function applyContents(seriesId: number, { ratingAverage, updatedAt, items }: Contents) {
    setCatalog((current) => current && ({
      ...current,
      series: current.series.map((item) => item.id === seriesId ? {
        ...item, ratingAverage, updatedAt,
        contents: items.map((content) => ({
          title: content.title, status: content.watch.session.status,
          categoryId: content.categoryId, ratingTenths: content.watch.ratingTenths,
        })),
      } : item),
    }));
  }

  async function step(item: ContinueItem, delta: -1 | 1) {
    if (busy) return;
    setBusy(true);
    setStepError(null);
    try {
      applyContents(item.seriesId, await updateWatch(item.seriesId, item.id, item.sessionId, { kind: "step", delta }));
    } catch (reason) {
      setStepError(catalogError(reason));
      setCatalog((current) => current && { ...current });
    } finally {
      setBusy(false);
    }
  }
  function taxonomyChanged(next: Taxonomy) {
    if (next.catalog) setCatalog(next.catalog);
    if (!preferences) return;
    const genreIds = preferences.genreIds.filter((id) => next.genres.some((term) => term.id === id));
    const contentFilters = { ...preferences.contentFilters };
    const categoryIds = new Set(next.categories.map((term) => term.id));
    let changed = genreIds.length !== preferences.genreIds.length;
    for (const [seriesId, filters] of Object.entries(contentFilters)) {
      if (filters.categoryId !== null && !categoryIds.has(filters.categoryId)) {
        if (filters.status === null) delete contentFilters[seriesId];
        else contentFilters[seriesId] = { ...filters, categoryId: null };
        changed = true;
      }
    }
    if (changed) updatePreferences({ ...preferences, genreIds, contentFilters });
  }
  const preferenceNotice = preferenceError && <div className="catalog-notice preference-notice">
    <p role="alert" className="catalog-error">{preferenceError === "read" ? t.filters.readError : t.filters.writeError}</p>
    <button type="button" onClick={retryPreferences}>{t.storage.retry}</button>
  </div>;

  return (
    <>
    <nav className="workspace__bar workspace__tabs" aria-label={copy.tabs}>
      <button type="button" aria-current={!opened ? "page" : undefined}
        aria-label={copy.returnToView(t.navigation[view])}
        onClick={() => onActiveSeries(null)}>{t.navigation[view]}</button>
      {tabs.map((id, index) => {
        const series = byId.get(id);
        return series && <div className="workspace__tab" key={id}>
          <button type="button" title={series.title} aria-current={id === activeSeries ? "page" : undefined} disabled={busy}
            onClick={() => {
              if (activeSeries !== id) setScopedContents(null);
              setEntryContent(null);
              onActiveSeries(id);
            }}>{series.title}</button>
          <button type="button" aria-label={copy.closeTab(series.title)} onClick={() => {
            setTabs((current) => current.filter((value) => value !== id));
            if (activeSeries === id) {
              setEntryContent(null);
              onActiveSeries(tabs[index + 1] ?? tabs[index - 1] ?? null);
            }
          }}>×</button>
        </div>;
      })}
    </nav>
    {opened ? <SeriesContents key={opened.id} series={opened} initialContentId={entryContent}
      genreNames={opened.genreIds.flatMap((id) => genreNames.get(id) ?? [])}
      filters={contentSelection ? (scopedContents?.filters ?? defaultContentFilters)
        : (preferences?.contentFilters[opened.id] ?? defaultContentFilters)}
      selection={contentSelection}
      onClearSelection={() => setScopedContents(null)}
      preferenceNotice={preferenceNotice}
      seriesBusy={busy} seriesError={error}
      onSeriesEdit={() => { setError(null); setDialog({ kind: "edit", series: opened }); }}
      onFavorite={() => void change(() => setFavorite(opened.id, !opened.favorite))}
      onFilterChange={(filters) => {
        if (contentSelection) setScopedContents({ seriesId: opened.id, filters });
        else if (preferences) {
          const contentFilters = { ...preferences.contentFilters };
          if (filters.categoryId === null && filters.status === null) delete contentFilters[opened.id];
          else contentFilters[opened.id] = filters;
          updatePreferences({ ...preferences, contentFilters });
        }
      }}
      onContentsChange={(contents) => applyContents(opened.id, contents)} /> : view === "statistics" ? (
      <StatisticsView revision={catalog} onExplore={explore} canOpenSeries={!!catalog && !busy}
        onOpenSeries={(id) => {
          focusTitle.current = "series";
          openSeries(id);
        }} />
    ) : view === "continueWatching" ? (
      <ContinueWatching revision={catalog} busy={busy} writeError={stepError}
        onOpen={(item) => openSeries(item.seriesId, item.id)} onStep={(item, delta) => void step(item, delta)} />
    ) : view === "settings" ? (
      <Settings preferences={preferences} notice={preferenceNotice} onTaxonomyChange={taxonomyChanged}
        backupsPanel={backupsPanel}
        onVisualChange={(value) => {
          if (preferences) updatePreferences({ ...preferences, ...value });
        }} />
    ) : (
    <main className="library" aria-labelledby="library-title">
      <header className="library__header">
        <div>
          <h1 id="library-title" tabIndex={-1}>{t.navigation[view]}</h1>
          <p>{favorites ? copy.favoritesDescription : copy.description}</p>
        </div>
        {!favorites && catalog && (
          <button className="primary" type="button" disabled={busy}
            onClick={() => { setError(null); setDialog({ kind: "edit", series: null }); }}>{copy.add}</button>
        )}
      </header>
      {exploring && <div className="insight-filter">
        <span>{t.insights.filteredBy(selectionLabel(exploring.selection, locale))}</span>
        <button type="button" onClick={() => setExploration(null)}>{t.insights.clearFilter}</button>
      </div>}
      {catalog && catalogFilters && <CatalogFilterBar filters={catalogFilters} genres={catalog.genres}
        favoritesView={favorites}
        onChange={(filters) => {
          if (exploring) setExploration({ ...exploring, filters });
          else if (preferences) updatePreferences({ ...preferences, ...filters });
        }}
        onReset={() => {
          if (exploring) setExploration(null);
          else if (preferences) updatePreferences({ ...preferences, ...defaultCatalogFilters });
        }} />}
      {preferenceNotice}
      {error && !dialog && (
        <div className="catalog-notice">
          <p role="alert" className="catalog-error">{copy.errors[error]}</p>
          <button type="button" onClick={() => {
            setCatalog(null);
            setError(null);
            setAttempt((current) => current + 1);
          }}>{t.storage.retry}</button>
        </div>
      )}
      {(!catalog || !catalogFilters) && !error && <p className="catalog-notice" role="status">{copy.loading}</p>}
      {items?.length === 0 && (
        <section className="library__empty" aria-labelledby="empty-title">
          <div className="film-frame" aria-hidden="true"><span /><span /><span /></div>
          <h2 id="empty-title">{filteredEmpty ? t.filters.empty : favorites ? copy.favoritesEmpty : copy.emptyTitle}</h2>
          <p>{filteredEmpty ? t.filters.emptyHint : favorites ? copy.favoritesHint : copy.emptyDescription}</p>
        </section>
      )}
      {items && items.length > 0 && (
        <ul className="catalog-grid">
          {items.map((series) => (
            <li className="series-card" key={series.id}>
              <div className="series-card__poster">
                <Cover coverKey={series.coverKey} />
                <button className="poster-link" type="button" disabled={busy}
                  onClick={() => openSeries(series.id, null, true)}><span>{copy.open}</span></button>
              </div>
              <div className="series-card__body">
                <h2>{series.title}</h2>
                {series.ratingAverage != null && <span className="rating">
                  {t.watch.averageValue(formatRating(series.ratingAverage, 2))}
                </span>}
                {series.genreIds.length > 0 && <p>{series.genreIds.map((id) => genreNames.get(id)).join(" · ")}</p>}
                <div className="series-card__actions">
                  <button type="button" className="favorite" disabled={busy}
                    aria-label={series.favorite ? copy.unmarkFavorite : copy.markFavorite}
                    aria-pressed={series.favorite}
                    onClick={() => void change(() => setFavorite(series.id, !series.favorite))}>
                    <span aria-hidden="true">{series.favorite ? "★" : "☆"}</span>
                  </button>
                  <button type="button" disabled={busy}
                    onClick={() => { setError(null); setDialog({ kind: "edit", series }); }}>{copy.edit}</button>
                  <button type="button" disabled={busy}
                    onClick={() => { setError(null); setDialog({ kind: "delete", series }); }}>{copy.remove}</button>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </main>
    )}
      {dialog?.kind === "edit" && catalog && (
        <SeriesForm series={dialog.series} catalog={catalog}
          onCancel={() => setDialog(null)}
          onSaved={(next) => { setCatalog(next); setDialog(null); }} />
      )}
      {dialog?.kind === "delete" && (
        <Modal title={copy.remove} busy={busy} onCancel={() => { setDialog(null); setError(null); }}>
          <p>{copy.confirmDelete(dialog.series.title)}</p>
          {error && <p role="alert" className="catalog-error">{copy.errors[error]}</p>}
          <div className="modal__actions">
            <button autoFocus type="button" disabled={busy}
              onClick={() => { setDialog(null); setError(null); }}>{copy.cancel}</button>
            <button type="button" className="danger" disabled={busy}
              onClick={() => void change(() => deleteSeries(dialog.series.id))}>{copy.remove}</button>
          </div>
        </Modal>
      )}
    </>
  );
}
