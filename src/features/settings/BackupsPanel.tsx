import { useState } from "react";
import { Modal } from "../../components/Modal";
import { useLocale } from "../../locales/locale";
import { accessBackup, backupError, type BackupAction, type BackupOutcome } from "../../services/backups";
import type { useAutomaticBackup } from "./useAutomaticBackup";

type Preview = Extract<BackupOutcome, { kind: "preview" }>;

export function BackupsPanel({ automatic, beforeRestore, onRestored, restored }: {
  automatic: ReturnType<typeof useAutomaticBackup>;
  beforeRestore: () => Promise<void>;
  onRestored: () => void;
  restored: boolean;
}) {
  const { t, timestamp, count, language } = useLocale();
  const copy = t.backups;
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<unknown>(null);

  async function run(action: BackupAction) {
    if (busy) return;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      if (action.kind === "restore") await beforeRestore();
      const outcome = await accessBackup(action, language);
      switch (outcome.kind) {
        case "status":
          automatic.setStatus(outcome.status);
          automatic.setError(null);
          if (action.kind === "set_automatic" && action.enabled) {
            const next = await accessBackup({ kind: "check_automatic" }, language);
            if (next.kind === "status") automatic.setStatus(next.status);
          }
          break;
        case "saved": setResult(outcome.name); break;
        case "preview": setPreview(outcome); break;
        case "restored": onRestored(); break;
      }
    } catch (reason) {
      setError(reason);
      if (action.kind === "restore") setPreview(null);
    } finally {
      setBusy(false);
    }
  }

  function cancel() {
    if (!preview || busy) return;
    void accessBackup({ kind: "cancel", token: preview.token }, language).catch(() => {});
    setPreview(null);
  }

  const available = automatic.status;
  const failure = error ?? automatic.error;
  return (
    <section className="settings__panel backups" aria-labelledby="backups-title">
      <h2 id="backups-title">{copy.title}</h2>
      {available && <label className="genre-choice">
        <input type="checkbox" checked={available.automatic} disabled={busy}
          onChange={(event) => void run({ kind: "set_automatic", enabled: event.target.checked })} />
        {copy.automatic}
      </label>}
      {!available && !failure && <p role="status">{copy.working}</p>}
      <p className="field-hint">{copy.automaticHint}</p>
      {available && <p className="field-hint">{available.lastSuccess ? <>
        {copy.last} · <time dateTime={available.lastSuccess}>{timestamp(available.lastSuccess)}</time>
      </> : copy.never}</p>}
      <div className="backups__actions">
        <button type="button" disabled={busy} onClick={() => void run({ kind: "open_folder" })}>{copy.folder}</button>
        <button type="button" className="primary" disabled={busy} onClick={() => void run({ kind: "save" })}>{copy.save}</button>
        <button type="button" disabled={busy} onClick={() => void run({ kind: "preview" })}>{copy.restore}</button>
      </div>
      <p className="field-hint">{copy.exportHint}</p>
      <div className="backups__actions">
        <button type="button" disabled={busy} onClick={() => void run({ kind: "export", format: "csv" })}>{copy.csv}</button>
        <button type="button" disabled={busy} onClick={() => void run({ kind: "export", format: "markdown" })}>{copy.markdown}</button>
      </div>
      {busy && !preview && <p role="status">{copy.working}</p>}
      {failure != null && <p role="alert" className="catalog-error">{copy.errors[backupError(failure, copy.errors)]}</p>}
      {result ? <p role="status">{copy.saved(result)}</p> : restored && !busy && <p role="status">{copy.restored}</p>}
      {preview && <Modal title={copy.confirmTitle} busy={busy} onCancel={cancel}>
        <p>{copy.warning}</p>
        <dl className="backup-summary">
          <div><dt>{copy.date}</dt><dd>{timestamp(preview.summary.createdAt)}</dd></div>
          <div><dt>{copy.schema}</dt><dd>{count(preview.summary.schemaVersion)}</dd></div>
          <div><dt>{copy.series}</dt><dd>{count(preview.summary.seriesCount)}</dd></div>
          <div><dt>{copy.contents}</dt><dd>{count(preview.summary.contentCount)}</dd></div>
          <div><dt>{copy.images}</dt><dd>{count(preview.summary.imageCount)}</dd></div>
        </dl>
        <div className="modal__actions">
          <button autoFocus type="button" disabled={busy} onClick={cancel}>{t.library.cancel}</button>
          <button type="button" className="danger" disabled={busy}
            onClick={() => void run({ kind: "restore", token: preview.token })}>{busy ? copy.working : copy.confirm}</button>
        </div>
      </Modal>}
    </section>
  );
}
