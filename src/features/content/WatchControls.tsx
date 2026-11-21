import { useId } from "react";
import { useLocale } from "../../locales/locale";
import type { WatchSnapshot, WatchStatus } from "../../services/watch";

export const watchStatuses = ["to_watch", "watching", "finished"] as const;

export function WatchControls({ watch, busy, onStep, onStatus, onEdit, onRewatch, showCompleted = false }: {
  watch: WatchSnapshot;
  busy: boolean;
  onStep: (delta: -1 | 1) => void;
  onStatus: (status: WatchStatus) => void;
  onEdit: () => void;
  onRewatch: () => void;
  showCompleted?: boolean;
}) {
  const { t, formatRating } = useLocale();
  const copy = t.watch;
  const { session, totalEpisodes, ratingTenths, completedViews } = watch;
  const statusId = useId();
  return (
    <div className="watch">
      <div className="watch__meta">
        <label htmlFor={statusId}>{copy.status}</label>
        <select id={statusId} value={session.status} disabled={busy}
          onChange={(event) => onStatus(event.target.value as WatchStatus)}>
          {watchStatuses.map((status) => <option key={status} value={status}>{copy.states[status]}</option>)}
        </select>
        {ratingTenths !== null && <span className="rating">{copy.ratingValue(formatRating(ratingTenths / 10, 1))}</span>}
      </div>
      <EpisodeProgress seen={session.watchedEpisodes} totalEpisodes={totalEpisodes} busy={busy} onStep={onStep} />
      <div className="watch__actions">
        {(showCompleted || completedViews > 0) && <span>{copy.completed(completedViews)}</span>}
        <button type="button" disabled={busy} onClick={onEdit}>{copy.edit}</button>
        {session.completed && <button type="button" disabled={busy} onClick={onRewatch}>{copy.rewatch}</button>}
      </div>
    </div>
  );
}

export function EpisodeProgress({ seen, totalEpisodes, busy, onStep }: {
  seen: number;
  totalEpisodes: number | null;
  busy: boolean;
  onStep: (delta: -1 | 1) => void;
}) {
  const { t } = useLocale();
  const copy = t.watch;
  return (
    <>
      <div className="watch__counter">
        <span>{copy.episodes(seen, totalEpisodes)}</span>
        <button type="button" aria-label={copy.decrement} disabled={busy || seen === 0} onClick={() => onStep(-1)}>−1</button>
        <button type="button" aria-label={copy.increment} disabled={busy || seen >= (totalEpisodes ?? 4_294_967_295)} onClick={() => onStep(1)}>+1</button>
      </div>
      {totalEpisodes !== null && <progress className="watch__progress" aria-label={copy.progress} max={totalEpisodes} value={seen} />}
    </>
  );
}
