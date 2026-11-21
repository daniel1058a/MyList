import { useState, type FormEvent } from "react";
import { Modal } from "../../components/Modal";
import { useLocale } from "../../locales/locale";
import { catalogError, type CatalogError } from "../../services/catalog";
import { accessTaxonomy, type Taxonomy, type Term, type TermGroup } from "../../services/taxonomy";

export type TermAction =
  | { kind: "save"; group: TermGroup; term: Term | null }
  | { kind: "delete"; group: TermGroup; term: Term };

export function TermDialog({ action, categories, onSave, onCancel }: {
  action: TermAction;
  categories: Term[];
  onSave: (data: Taxonomy) => void;
  onCancel: () => void;
}) {
  const { t } = useLocale();
  const copy = t.settings;
  const common = t.library;
  const [name, setName] = useState(action.term?.name ?? "");
  const [replacement, setReplacement] = useState("");
  const [reassign, setReassign] = useState(
    action.kind === "delete" && action.group === "category" && action.term.usage > 0,
  );
  const [error, setError] = useState<CatalogError | null>(null);
  const [busy, setBusy] = useState(false);
  const genre = action.group === "genre";
  const deleting = action.kind === "delete";
  const title = deleting ? (genre ? copy.deleteGenre : copy.deleteCategory) :
    action.term ? (genre ? copy.renameGenre : copy.renameCategory) : (genre ? copy.addGenre : copy.addCategory);
  const destinations = categories.filter((term) => term.id !== action.term?.id);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busy) return;
    if (action.kind === "save" && !name.trim()) {
      setError(genre ? "invalid_genre" : "invalid_category");
      return;
    }
    if (reassign && !replacement) {
      setError("category_in_use");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      onSave(await accessTaxonomy(action.kind === "save"
        ? { kind: "save", group: action.group, id: action.term?.id ?? null, name }
        : { kind: "delete", group: action.group, id: action.term.id, replacementId: replacement ? Number(replacement) : null }));
    } catch (reason) {
      const code = catalogError(reason);
      if (code === "category_in_use") setReassign(true);
      setError(code);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal title={title} busy={busy} onCancel={onCancel}>
      <form noValidate onSubmit={(event) => void submit(event)}>
        <fieldset className="modal__fields" disabled={busy}>
          {action.kind === "save" ? <label className="field">{copy.name}
            <input autoFocus required maxLength={60} value={name}
              aria-invalid={error === "duplicate_term" || error === "invalid_genre" || error === "invalid_category"}
              onChange={(event) => setName(event.target.value)} />
          </label> : <>
            <p>{copy.confirm(action.term.name)}</p>
            {genre && action.term.usage > 0 && <p className="field-hint">{copy.genreDetach(action.term.usage)}</p>}
            {reassign && <>
              {action.term.usage > 0 && <p className="field-hint">{copy.categoryMove(action.term.usage)}</p>}
              {destinations.length > 0 ? <label className="field">{copy.replacement}
                <select aria-label={copy.replacement} value={replacement} onChange={(event) => setReplacement(event.target.value)}>
                  <option value="">{copy.chooseReplacement}</option>
                  {destinations.map((term) => <option key={term.id} value={term.id}>{term.name}</option>)}
                </select>
              </label> : <p className="field-hint">{copy.noReplacement}</p>}
            </>}
          </>}
        </fieldset>
        {error && <p role="alert" className="catalog-error">{common.errors[error]}</p>}
        <div className="modal__actions">
          <button type="button" autoFocus={deleting} disabled={busy} onClick={onCancel}>{common.cancel}</button>
          <button type="submit" className={deleting ? "danger" : "primary"}
            disabled={busy || (reassign && !replacement)}>
            {busy ? common.saving : reassign ? copy.reassignAndRemove : deleting ? copy.remove : common.save}
          </button>
        </div>
      </form>
    </Modal>
  );
}
