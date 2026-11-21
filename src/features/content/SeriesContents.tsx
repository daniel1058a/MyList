import { useEffect, useLayoutEffect, useRef, useState, type DragEvent, type ReactNode } from "react";
import { Cover } from "../../components/Cover";
import { Modal } from "../../components/Modal";
import { StatusFilter } from "../../components/StatusFilter";
import { selectionLabel, useLocale } from "../../locales/locale";
import { catalogError, type CatalogError, type Series } from "../../services/catalog";
import { deleteContent, listContents, reorderContent, type Content, type Placement, type SeriesContents as Contents } from "../../services/content";
import { updateWatch, type WatchAction, type WatchStatus } from "../../services/watch";
import { defaultContentFilters, type ContentFilters } from "../../services/preferences";
import type { ContentSelection } from "../../services/insights";
import { filterContents } from "../library/catalogFilters";
import { ContentForm } from "./ContentForm";
import { ContentDetail } from "./ContentDetail";
import { SeriesHero } from "./SeriesHero";
import { WatchControls } from "./WatchControls";
import { WatchForm } from "./WatchForm";
import { WatchHistory } from "./WatchHistory";

const dragType = "application/x-mylist-content";
type Dialog = { kind: "edit"; item: Content | null } | { kind: "delete" | "watch" | "reset" | "history"; item: Content };

function placementAt(event: DragEvent<HTMLElement>): Placement {
  const { top, height } = event.currentTarget.getBoundingClientRect();
  return event.clientY < top + height / 2 ? "before" : "after";
}

export function SeriesContents({ series, initialContentId, genreNames, filters, selection, onClearSelection, preferenceNotice, seriesBusy, seriesError, onSeriesEdit, onFavorite, onFilterChange, onContentsChange }: {
  series: Series;
  initialContentId: number | null;
  genreNames: string[];
  filters: ContentFilters;
  selection?: ContentSelection;
  onClearSelection: () => void;
  preferenceNotice: ReactNode;
  seriesBusy: boolean;
  seriesError: CatalogError | null;
  onSeriesEdit: () => void;
  onFavorite: () => void;
  onFilterChange: (filters: ContentFilters) => void;
  onContentsChange: (contents: Contents) => void;
}) {
  const locale = useLocale();
  const { t } = locale;
  const copy = t.content;
  const common = t.library;
  const [contents, setContents] = useState<Contents | null>(null);
  const [error, setError] = useState<CatalogError | null>(null);
  const [attempt, setAttempt] = useState(0);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  const [mutatingContent, setBusy] = useState(false);
  const [dragged, setDragged] = useState<number | null>(null);
  const [dropTarget, setDropTarget] = useState<{ id: number; placement: Placement } | null>(null);
  const [announcement, setAnnouncement] = useState("");
  const [selectedId, setSelectedId] = useState(initialContentId);
  const directEntry = useRef(initialContentId !== null);
  const returnPosition = useRef<{ top: number; button: HTMLButtonElement } | null>(null);
  const page = useRef<HTMLElement>(null);
  const seriesId = series.id;
  const busy = mutatingContent || seriesBusy;
  const selected = contents?.items.find((item) => item.id === selectedId);
  const detailVisible = !!selected;

  useEffect(() => {
    let active = true;
    void listContents(seriesId).then((data) => {
      if (active) setContents(data);
    }).catch((reason: unknown) => {
      if (active) setError(catalogError(reason));
    });
    return () => { active = false; };
  }, [seriesId, attempt]);

  useLayoutEffect(() => {
    const position = returnPosition.current;
    if (!position) return;
    if (selectedId !== null) {
      window.scrollTo(0, 0);
      page.current?.querySelector<HTMLElement>("#content-title")?.focus({ preventScroll: true });
    } else {
      const fallback = page.current?.querySelector<HTMLElement>("#series-title");
      const target = position.button.isConnected ? position.button : fallback;
      target?.focus({ preventScroll: true });
      window.scrollTo(0, position.top);
      returnPosition.current = null;
    }
  }, [selectedId]);

  useLayoutEffect(() => {
    if (!detailVisible || !directEntry.current) return;
    directEntry.current = false;
    window.scrollTo(0, 0);
    page.current?.querySelector<HTMLElement>("#content-title")?.focus({ preventScroll: true });
  }, [detailVisible]);

  function applyContents(data: Contents) {
    setContents(data);
    onContentsChange(data);
    setDialog(null);
    if (selectedId !== null && !data.items.some((item) => item.id === selectedId)) setSelectedId(null);
  }

  async function change(action: () => Promise<Contents>, message = "") {
    if (busy) return;
    setBusy(true);
    setError(null);
    setAnnouncement("");
    try {
      applyContents(await action());
      setAnnouncement(message);
    } catch (reason) {
      setError(catalogError(reason));
    } finally {
      setBusy(false);
    }
  }

  function reorder(id: number, target: Content | undefined, placement: Placement) {
    if (target && id !== target.id) {
      void change(() => reorderContent(seriesId, id, target.id, placement), copy.reordered);
    }
  }

  function update(item: Content, action: WatchAction) {
    return updateWatch(seriesId, item.id, item.watch.session.id, action);
  }

  function changeStatus(item: Content, status: WatchStatus) {
    if (status === "to_watch" && item.watch.session.watchedEpisodes > 0) {
      setError(null);
      setDialog({ kind: "reset", item });
    } else {
      void change(() => update(item, { kind: "set_status", status, resetProgress: false }));
    }
  }

  function showDialog(kind: "edit" | "delete" | "history" | "watch", item: Content) {
    setError(null);
    setDialog({ kind, item });
  }

  function watchControls(item: Content, showCompleted = false) {
    return <WatchControls watch={item.watch} busy={busy} showCompleted={showCompleted}
      onStep={(delta) => void change(() => update(item, { kind: "step", delta }))}
      onStatus={(status) => changeStatus(item, status)}
      onEdit={() => showDialog("watch", item)}
      onRewatch={() => void change(() => update(item, { kind: "rewatch" }))} />;
  }

  const names = new Map(contents?.categories.map((category) => [category.id, category.name]));
  const average = contents ? contents.ratingAverage : series.ratingAverage;
  const visibleItems = contents ? filterContents(contents.items, filters, selection) : [];

  return (
    <main className="library series-page" ref={page} aria-labelledby={selected ? "content-title" : "series-title"}>
      {selected && <ContentDetail seriesId={seriesId} item={selected} category={names.get(selected.categoryId)}
        busy={busy} controls={watchControls(selected, true)}
        notice={error && !dialog && <p role="alert" className="catalog-error">{common.errors[error]}</p>}
        onBack={() => { setError(null); setSelectedId(null); }}
        onEdit={() => showDialog("edit", selected)} onDelete={() => showDialog("delete", selected)} />}
      <div hidden={!!selected}>
        <SeriesHero series={series} genreNames={genreNames} average={average}
          busy={busy} onEdit={onSeriesEdit} onFavorite={onFavorite} />
        {seriesError && <p role="alert" className="catalog-error">{common.errors[seriesError]}</p>}
        <section className="series-contents" aria-labelledby="contents-title">
          <header className="library__header">
            <h2 id="contents-title">{copy.label}</h2>
            {contents && <button className="primary" type="button" disabled={busy} onClick={() => {
              setError(null); setDialog({ kind: "edit", item: null });
            }}>{copy.add}</button>}
          </header>
          {selection && <div className="insight-filter">
            <span>{t.insights.filteredBy(selectionLabel(selection, locale))}</span>
            <button type="button" onClick={onClearSelection}>{t.insights.clearFilter}</button>
          </div>}
          {contents && <div className="filter-bar filter-bar__row" role="group" aria-label={t.filters.internal}>
            <label className="field">{t.filters.category}
              <select aria-label={t.filters.category} value={filters.categoryId ?? ""}
                onChange={(event) => onFilterChange({ ...filters, categoryId: event.target.value ? Number(event.target.value) : null })}>
                <option value="">{t.filters.allCategories}</option>
                {contents.categories.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}
              </select>
            </label>
            <StatusFilter value={filters.status} onChange={(status) => onFilterChange({ ...filters, status })} />
            <button type="button" onClick={() => selection ? onClearSelection() : onFilterChange(defaultContentFilters)}>{t.filters.reset}</button>
          </div>}
          {preferenceNotice}
          {error && !dialog && <div className="catalog-notice">
            <p role="alert" className="catalog-error">{common.errors[error]}</p>
            <button type="button" onClick={() => {
              setError(null); setContents(null); setAttempt((value) => value + 1);
            }}>{t.storage.retry}</button>
          </div>}
          {!contents && !error && <p className="catalog-notice" role="status">{copy.loading}</p>}
          {contents && <p className="visually-hidden" role="status">{announcement}</p>}
          {contents && visibleItems.length === 0 && <p className="catalog-notice">
            {contents.items.length ? t.filters.emptyContents : copy.empty}
          </p>}
          {visibleItems.length > 0 && <ul className="content-list">
            {visibleItems.map((item, position) => (
              <li className="content-card" key={item.id}
                data-drop={dropTarget?.id === item.id ? dropTarget.placement : undefined}
                onDragOver={(event) => {
                  if (busy || dragged === null || dragged === item.id || !event.dataTransfer.types.includes(dragType)) return;
                  event.preventDefault();
                  event.dataTransfer.dropEffect = "move";
                  const placement = placementAt(event);
                  setDropTarget((current) => current?.id === item.id && current.placement === placement ? current : { id: item.id, placement });
                }}
                onDragLeave={(event) => {
                  if (event.relatedTarget instanceof Node && event.currentTarget.contains(event.relatedTarget)) return;
                  setDropTarget((current) => current?.id === item.id ? null : current);
                }}
                onDrop={(event) => {
                  if (!event.dataTransfer.types.includes(dragType)) return;
                  event.preventDefault();
                  const source = dragged;
                  setDragged(null);
                  setDropTarget(null);
                  if (!busy && source !== null && event.dataTransfer.getData(dragType) === String(source)) {
                    reorder(source, item, placementAt(event));
                  }
                }}>
                <Cover coverKey={item.coverKey} />
                <div className="content-card__body">
                  <h3 aria-label={item.title}><button type="button" className="content-card__title" aria-label={copy.open(item.title)}
                    disabled={busy} onClick={(event) => {
                      returnPosition.current = { top: window.scrollY, button: event.currentTarget };
                      setError(null);
                      setSelectedId(item.id);
                    }}>
                    {item.title}<span aria-hidden="true" />
                  </button></h3>
                  <p className="content-card__meta">
                    {names.get(item.categoryId)}{item.releaseYear !== null && <> · {item.releaseYear}</>}
                  </p>
                  {watchControls(item)}
                  <div className="content-card__actions">
                    <span className="content-card__drag" draggable={!busy && visibleItems.length > 1} title={copy.drag} aria-hidden="true"
                      onDragStart={(event) => {
                        event.dataTransfer.setData(dragType, String(item.id));
                        event.dataTransfer.effectAllowed = "move";
                        setDragged(item.id);
                      }}
                      onDragEnd={() => { setDragged(null); setDropTarget(null); }}>⠿</span>
                    <details className="content-card__menu" name="content-actions">
                      <summary><span aria-hidden="true">⋯</span> {copy.moreActions}</summary>
                      <div>
                        <button type="button" disabled={busy || position === 0}
                          onClick={() => reorder(item.id, visibleItems[position - 1], "before")}>{copy.up}</button>
                        <button type="button" disabled={busy || position === visibleItems.length - 1}
                          onClick={() => reorder(item.id, visibleItems[position + 1], "after")}>{copy.down}</button>
                        <button type="button" disabled={busy} onClick={() => showDialog("history", item)}>{t.watch.history}</button>
                        <button type="button" disabled={busy} onClick={() => showDialog("edit", item)}>{copy.edit}</button>
                        <button type="button" disabled={busy} onClick={() => showDialog("delete", item)}>{copy.remove}</button>
                      </div>
                    </details>
                  </div>
                </div>
              </li>
            ))}
          </ul>}
        </section>
      </div>
      {dialog?.kind === "edit" && contents && <ContentForm seriesId={seriesId}
        item={dialog.item} categories={contents.categories} onCancel={() => setDialog(null)}
        onSave={applyContents} />}
      {dialog?.kind === "watch" && <WatchForm watch={dialog.item.watch} onCancel={() => setDialog(null)}
        onSave={async (action) => { applyContents(await update(dialog.item, action)); }} />}
      {dialog?.kind === "history" && <WatchHistory seriesId={seriesId} contentId={dialog.item.id} onClose={() => setDialog(null)} />}
      {dialog?.kind === "reset" && <Modal title={t.watch.resetTitle} busy={busy}
        onCancel={() => { setDialog(null); setError(null); }}>
        <p>{t.watch.resetMessage(dialog.item.watch.session.watchedEpisodes)}</p>
        {error && <p role="alert" className="catalog-error">{common.errors[error]}</p>}
        <div className="modal__actions">
          <button autoFocus type="button" disabled={busy} onClick={() => {
            setDialog(null); setError(null);
          }}>{common.cancel}</button>
          <button type="button" className="danger" disabled={busy}
            onClick={() => void change(() => update(dialog.item, { kind: "set_status", status: "to_watch", resetProgress: true }))}>
            {t.watch.reset}
          </button>
        </div>
      </Modal>}
      {dialog?.kind === "delete" && <Modal title={copy.remove} busy={busy}
        onCancel={() => { setDialog(null); setError(null); }}>
        <p>{copy.confirmDelete(dialog.item.title)}</p>
        {error && <p role="alert" className="catalog-error">{common.errors[error]}</p>}
        <div className="modal__actions">
          <button autoFocus type="button" disabled={busy} onClick={() => {
            setDialog(null); setError(null);
          }}>{common.cancel}</button>
          <button type="button" className="danger" disabled={busy}
            onClick={() => void change(() => deleteContent(seriesId, dialog.item.id))}>{copy.remove}</button>
        </div>
      </Modal>}
    </main>
  );
}
