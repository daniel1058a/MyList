import { useEffect, useState } from "react";
import { Modal } from "../../components/Modal";
import { useLocale } from "../../locales/locale";
import { catalogError, type CatalogError } from "../../services/catalog";
import { watchHistory, type WatchSession } from "../../services/watch";

export function WatchHistory({ seriesId, contentId, onClose }: {
  seriesId: number;
  contentId: number;
  onClose: () => void;
}) {
  const { t } = useLocale();
  const copy = t.watch;
  return (
    <Modal title={copy.history} busy={false} onCancel={onClose}>
      <WatchSessionList seriesId={seriesId} contentId={contentId} />
      <div className="modal__actions"><button type="button" onClick={onClose}>{copy.closeHistory}</button></div>
    </Modal>
  );
}

export function WatchDates({ session }: { session: WatchSession }) {
  const { t, date } = useLocale();
  const copy = t.watch;
  if (!session.startedOn && !session.finishedOn) return null;
  return (
    <dl className="watch-dates">
      {session.startedOn && <div><dt>{copy.startedOn}</dt><dd><time dateTime={session.startedOn}>{date(session.startedOn)}</time></dd></div>}
      {session.finishedOn && <div><dt>{copy.finishedOn}</dt><dd><time dateTime={session.finishedOn}>{date(session.finishedOn)}</time></dd></div>}
    </dl>
  );
}

export function WatchSessionList({ seriesId, contentId, revision }: {
  seriesId: number;
  contentId: number;
  revision?: WatchSession;
}) {
  const { t } = useLocale();
  const copy = t.watch;
  const [sessions, setSessions] = useState<WatchSession[] | null>(null);
  const [error, setError] = useState<CatalogError | null>(null);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let active = true;
    void watchHistory(seriesId, contentId).then((data) => {
      if (active) {
        setSessions(data);
        setError(null);
      }
    }).catch((reason: unknown) => {
      if (active) setError(catalogError(reason));
    });
    return () => { active = false; };
  }, [seriesId, contentId, attempt, revision]);

  return (
    <>
      {error && <>
        <p role="alert" className="catalog-error">{t.library.errors[error]}</p>
        <button type="button" onClick={() => { setError(null); setAttempt((value) => value + 1); }}>{t.storage.retry}</button>
      </>}
      {!sessions && !error && <p role="status">{copy.loadingHistory}</p>}
      {sessions?.length === 0 && <p className="field-hint">{copy.emptyHistory}</p>}
      {!!sessions?.length && <ol className="watch-history">
        {sessions.map((session, index) => (
          <li key={session.id}>
            <h3>{copy.session(sessions.length - index)}{session.active && " · " + copy.active}</h3>
            <p>{copy.states[session.status]} · {copy.episodes(session.watchedEpisodes, null)}</p>
            {session.completed && <p>{copy.recorded}</p>}
            <WatchDates session={session} />
          </li>
        ))}
      </ol>}
    </>
  );
}
