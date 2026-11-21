import { useState, type FormEvent } from "react";
import { Modal } from "../../components/Modal";
import { useLocale } from "../../locales/locale";
import { catalogError, type CatalogError } from "../../services/catalog";
import type { WatchAction, WatchSnapshot } from "../../services/watch";

function wholeNumber(value: string) {
  return /^\d+$/.test(value) ? Number(value) : NaN;
}

function toTenths(value: string) {
  const decimal = /^(\d{1,2})(?:[,.](\d))?$/.exec(value);
  return decimal ? Number(decimal[1]) * 10 + Number(decimal[2] ?? 0) : NaN;
}

export function WatchForm({ watch, onSave, onCancel }: {
  watch: WatchSnapshot;
  onSave: (action: WatchAction) => Promise<void>;
  onCancel: () => void;
}) {
  const { t, formatRating } = useLocale();
  const copy = t.watch;
  const common = t.library;
  const { session } = watch;
  const [total, setTotal] = useState(watch.totalEpisodes?.toString() ?? "");
  const [seen, setSeen] = useState(session.watchedEpisodes.toString());
  const [rating, setRating] = useState(watch.ratingTenths === null ? "" : formatRating(watch.ratingTenths / 10, 1));
  const [started, setStarted] = useState(session.startedOn ?? "");
  const [finished, setFinished] = useState(session.finishedOn ?? "");
  const [error, setError] = useState<CatalogError | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busy) return;
    const totalEpisodes = total === "" ? null : wholeNumber(total);
    const watchedEpisodes = wholeNumber(seen);
    if ((totalEpisodes !== null && (!Number.isInteger(totalEpisodes) || totalEpisodes < 1 || totalEpisodes > 4_294_967_295))
      || !Number.isInteger(watchedEpisodes) || watchedEpisodes < 0 || watchedEpisodes > (totalEpisodes ?? 4_294_967_295)) {
      setError("invalid_progress");
      return;
    }
    const ratingText = rating.trim();
    const ratingTenths = ratingText === "" ? null : toTenths(ratingText);
    if (ratingTenths !== null && (!Number.isInteger(ratingTenths) || ratingTenths < 10 || ratingTenths > 100)) {
      setError("invalid_rating");
      return;
    }
    const progressChanged = totalEpisodes !== watch.totalEpisodes || watchedEpisodes !== session.watchedEpisodes;
    setBusy(true);
    setError(null);
    try {
      await onSave({
        kind: "edit", progress: progressChanged ? { totalEpisodes, watchedEpisodes } : null,
        ratingTenths, startedOn: started || null, finishedOn: finished || null,
      });
    } catch (reason) {
      setError(catalogError(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal title={copy.edit} busy={busy} onCancel={onCancel}>
      <form onSubmit={(event) => void submit(event)}>
        <fieldset disabled={busy} className="modal__fields">
          <div className="watch-form__row">
            <label className="field">{copy.seen}
              <input autoFocus inputMode="numeric" maxLength={10} value={seen} onChange={(event) => setSeen(event.target.value)} />
            </label>
            <label className="field">{copy.total}
              <input inputMode="numeric" maxLength={10} value={total} onChange={(event) => setTotal(event.target.value)} />
            </label>
          </div>
          <p className="field-hint">{copy.totalHint}</p>
          <label className="field">{copy.rating}
            <input inputMode="decimal" value={rating} onChange={(event) => setRating(event.target.value)} />
          </label>
          <p className="field-hint">{copy.ratingHint}</p>
          <div className="watch-form__row">
            <label className="field">{copy.startedOn}
              <input type="date" min="0001-01-01" max="9999-12-31" value={started} onChange={(event) => setStarted(event.target.value)} />
            </label>
            <label className="field">{copy.finishedOn}
              <input type="date" min="0001-01-01" max="9999-12-31" value={finished} onChange={(event) => setFinished(event.target.value)} />
            </label>
          </div>
        </fieldset>
        {error && <p className="catalog-error" role="alert">{common.errors[error]}</p>}
        <div className="modal__actions">
          <button type="button" disabled={busy} onClick={onCancel}>{common.cancel}</button>
          <button type="submit" className="primary" disabled={busy}>{busy ? common.saving : common.save}</button>
        </div>
      </form>
    </Modal>
  );
}
