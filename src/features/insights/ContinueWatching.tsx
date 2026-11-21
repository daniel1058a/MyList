import { Cover } from "../../components/Cover";
import { useLocale } from "../../locales/locale";
import { type CatalogError } from "../../services/catalog";
import { readContinueWatching, type ContinueItem } from "../../services/insights";
import { EpisodeProgress } from "../content/WatchControls";
import { useInsights } from "./useInsights";

export function ContinueWatching({ revision, busy, writeError, onStep, onOpen }: {
  revision: object | null;
  busy: boolean;
  writeError: CatalogError | null;
  onStep: (item: ContinueItem, delta: -1 | 1) => void;
  onOpen: (item: ContinueItem) => void;
}) {
  const { t, formatRating } = useLocale();
  const { data, ready, error, retry } = useInsights(readContinueWatching, revision);
  const unavailable = busy || !ready || !!error || revision === null;
  const notice = error ?? writeError;
  return (
    <main className="library" aria-labelledby="continue-title">
      <header className="library__header"><div>
        <h1 id="continue-title">{t.navigation.continueWatching}</h1><p>{t.insights.continueDescription}</p>
      </div></header>
      {notice && <div className="catalog-notice">
        <p role="alert" className="catalog-error">{t.library.errors[notice]}</p>
        {error && <button type="button" onClick={retry}>{t.storage.retry}</button>}
      </div>}
      {!data && !error && <p className="catalog-notice" role="status">{t.insights.loading}</p>}
      {data?.length === 0 && <section className="library__empty">
        <h2>{t.insights.emptyContinue}</h2><p>{t.insights.emptyContinueHint}</p>
      </section>}
      {!!data?.length && <ul className="content-list continue-list" aria-busy={busy || (!ready && !error)}>
        {data.map((item) => <li className="content-card" key={item.id}>
          <Cover coverKey={item.coverKey} />
          <div className="content-card__body">
            <h3><button type="button" className="content-card__title" aria-label={t.content.open(item.title)}
              disabled={unavailable} onClick={() => onOpen(item)}>{item.title}<span aria-hidden="true" /></button></h3>
            <p className="content-card__meta">{item.seriesTitle} · {item.category}</p>
            {item.ratingTenths !== null && <p className="rating">{t.watch.ratingValue(formatRating(item.ratingTenths / 10, 1))}</p>}
            <EpisodeProgress seen={item.watchedEpisodes} totalEpisodes={item.totalEpisodes} busy={unavailable}
              onStep={(delta) => onStep(item, delta)} />
          </div>
        </li>)}
      </ul>}
    </main>
  );
}
