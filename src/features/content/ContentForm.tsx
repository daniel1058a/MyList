import { useId, useState, type FormEvent } from "react";
import { CoverPicker, useCoverDraft } from "../../components/CoverPicker";
import { Modal } from "../../components/Modal";
import { useLocale } from "../../locales/locale";
import { catalogError, type CatalogError } from "../../services/catalog";
import { saveContent, type Category, type Content, type SeriesContents } from "../../services/content";

export function ContentForm({ seriesId, item, categories, onSave, onCancel }: {
  seriesId: number;
  item: Content | null;
  categories: Category[];
  onSave: (contents: SeriesContents) => void;
  onCancel: () => void;
}) {
  const { t } = useLocale();
  const copy = t.content;
  const fields = t.library;
  const [title, setTitle] = useState(item?.title ?? "");
  const [selection, setSelection] = useState(item ? String(item.categoryId) : categories.length ? "" : "new");
  const [newCategory, setNewCategory] = useState("");
  const [year, setYear] = useState(item?.releaseYear?.toString() ?? "");
  const [description, setDescription] = useState(item?.description ?? "");
  const [opinion, setOpinion] = useState(item?.opinion ?? "");
  const [error, setError] = useState<CatalogError | null>(null);
  const [busy, setBusy] = useState(false);
  const categoryId = useId();
  const coverDraft = useCoverDraft(setError);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busy) return;
    if (!title.trim()) { setError("invalid_title"); return; }
    const category = selection === "new" ? newCategory.trim() : categories.find((entry) => String(entry.id) === selection)?.name;
    if (!category) { setError("invalid_category"); return; }
    const releaseYear = year === "" ? null : Number(year);
    if (releaseYear !== null && (!/^\d{1,4}$/.test(year) || releaseYear < 1)) {
      setError("invalid_year"); return;
    }
    setError(null);
    setBusy(true);
    try {
      const cover = await coverDraft.change();
      onSave(await saveContent({
        id: item?.id ?? null, seriesId, title, category, releaseYear, description, opinion, cover,
      }));
    } catch (reason) {
      setError(catalogError(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal title={item ? copy.edit : copy.add} busy={busy} onCancel={onCancel}>
      <form noValidate onSubmit={(event) => void submit(event)}>
        <fieldset disabled={busy} className="modal__fields">
          <label className="field">{fields.title}
            <input autoFocus required maxLength={200} value={title}
              aria-invalid={error === "invalid_title"} onChange={(event) => setTitle(event.target.value)} />
          </label>
          <div className="content-form__row">
            <div className="field">
              <label htmlFor={categoryId}>{copy.category}</label>
              <select id={categoryId} required value={selection} aria-invalid={error === "invalid_category"} onChange={(event) => setSelection(event.target.value)}>
                <option value="" disabled>{copy.chooseCategory}</option>
                {categories.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}
                <option value="new">{copy.newCategory}</option>
              </select>
            </div>
            <label className="field">{copy.year}
              <input inputMode="numeric" maxLength={4} value={year}
                aria-invalid={error === "invalid_year"} onChange={(event) => setYear(event.target.value)} />
            </label>
          </div>
          {selection === "new" && <label className="field">{copy.categoryName}
            <input required maxLength={60} value={newCategory} onChange={(event) => setNewCategory(event.target.value)} />
          </label>}
          <label className="field">{copy.description}
            <textarea rows={3} maxLength={10000} value={description} onChange={(event) => setDescription(event.target.value)} />
          </label>
          <label className="field">{copy.opinion}
            <textarea rows={3} maxLength={10000} value={opinion} onChange={(event) => setOpinion(event.target.value)} />
          </label>
          <CoverPicker coverKey={item?.coverKey ?? null} draft={coverDraft} busy={busy} />
        </fieldset>
        {error && <p className="catalog-error" role="alert">{fields.errors[error]}</p>}
        <div className="modal__actions">
          <button type="button" disabled={busy} onClick={onCancel}>{fields.cancel}</button>
          <button type="submit" className="primary" disabled={busy}>{busy ? fields.saving : fields.save}</button>
        </div>
      </form>
    </Modal>
  );
}
