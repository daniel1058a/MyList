import { convertFileSrc } from "@tauri-apps/api/core";
import { useId, useState } from "react";
import { Cover } from "../../components/Cover";
import { StarRating } from "../../components/StarRating";
import { useLocale } from "../../locales/locale";
import type { Series } from "../../services/catalog";
import { watchStatuses } from "./WatchControls";

export function SeriesHero({ series, genreNames, average, busy, onEdit, onFavorite }: {
  series: Series;
  genreNames: string[];
  average: number | null;
  busy: boolean;
  onEdit: () => void;
  onFavorite: () => void;
}) {
  const { t, count } = useLocale();
  const copy = t.seriesDetail;
  const [failed, setFailed] = useState<string[]>([]);
  const backdrop = [series.panoramaKey, series.coverKey].find((key) => key && !failed.includes(key));
  const counts = { to_watch: 0, watching: 0, finished: 0 };
  for (const { status } of series.contents) counts[status] += 1;
  const favoriteLabel = series.favorite ? t.library.unmarkFavorite : t.library.markFavorite;

  return (
    <header className="series-hero">
      <div className="series-hero__art" aria-hidden="true">
        {backdrop && <img key={backdrop} src={convertFileSrc(backdrop, "mylist-cover")} alt="" decoding="async"
          onError={() => setFailed((keys) => [...keys, backdrop])} />}
      </div>
      <div className="series-hero__layout">
        <Cover coverKey={series.coverKey} eager />
        <div className="series-hero__info">
          <h1 id="series-title" tabIndex={-1}>{series.title}</h1>
          {genreNames.length > 0 && <ul className="genre-tags" aria-label={t.library.genres}>
            {genreNames.map((name) => <li key={name}>{name}</li>)}
          </ul>}
          {series.description && <SeriesDescription text={series.description} />}
          <StarRating value={average} />
          <ul className="series-hero__counts" aria-label={copy.statusSummary}>
            {watchStatuses.map((status) => <li key={status} data-state={status}>
              <strong>{count(counts[status])}</strong> {t.watch.states[status]}
            </li>)}
          </ul>
          <div className="series-hero__actions">
            <button type="button" disabled={busy} onClick={onEdit}>{t.library.edit}</button>
            <button type="button" className="favorite" aria-pressed={series.favorite} disabled={busy} onClick={onFavorite}>
              <span aria-hidden="true">{series.favorite ? "★" : "☆"}</span> {favoriteLabel}
            </button>
          </div>
        </div>
      </div>
    </header>
  );
}

function SeriesDescription({ text }: { text: string }) {
  const { t } = useLocale();
  const copy = t.seriesDetail;
  const [expanded, setExpanded] = useState(false);
  const descriptionId = useId();
  const preview = Array.from(text.split("\n").slice(0, 3).join("\n")).slice(0, 260).join("").trimEnd();
  const long = preview !== text;
  return (
    <div className="series-hero__description">
      <p id={descriptionId}>{!long || expanded ? text : preview + "…"}</p>
      {long && <button className="text-button" type="button" aria-expanded={expanded} aria-controls={descriptionId}
        onClick={() => setExpanded((value) => !value)}>{expanded ? copy.hideDescription : copy.showDescription}</button>}
    </div>
  );
}
