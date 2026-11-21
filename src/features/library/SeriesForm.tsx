import { useState, type FormEvent } from "react";
import { Modal } from "../../components/Modal";
import { CoverPicker, useCoverDraft } from "../../components/CoverPicker";
import { useLocale } from "../../locales/locale";
import { catalogError, saveSeries, type Catalog, type CatalogError, type Series } from "../../services/catalog";

const normalize = (value: string) => value.trim().replace(/\s+/g, " ").normalize("NFD").toLowerCase().replace(/\p{M}/gu, "");

export function SeriesForm({ series, catalog, onSaved, onCancel }: {
  series: Series | null;
  catalog: Catalog;
  onSaved: (next: Catalog) => void;
  onCancel: () => void;
}) {
  const { t } = useLocale();
  const copy = t.library;
  const [title, setTitle] = useState(series?.title ?? "");
  const [description, setDescription] = useState(series?.description ?? "");
  const [names, setNames] = useState(() => catalog.genres
    .filter((genre) => series?.genreIds.includes(genre.id)).map((genre) => genre.name));
  const [draftGenres, setDraftGenres] = useState<string[]>([]);
  const [genreInput, setGenreInput] = useState("");
  const [error, setError] = useState<CatalogError | null>(null);
  const coverDraft = useCoverDraft(setError);
  const panoramaDraft = useCoverDraft(setError);
  const [busy, setBusy] = useState(false);

  const options = [...catalog.genres.map((genre) => genre.name), ...draftGenres];
  const toggle = (name: string, checked: boolean) => setNames((current) =>
    checked ? [...current, name] : current.filter((item) => item !== name));

  function addGenre() {
    const name = genreInput.trim();
    const normalized = normalize(name);
    if (!normalized || name.length > 60) {
      setError("invalid_genre");
      return;
    }
    const existing = options.find((option) => normalize(option) === normalized);
    if (!existing) setDraftGenres([...draftGenres, name]);
    const selected = existing ?? name;
    if (!names.includes(selected)) toggle(selected, true);
    setGenreInput("");
    setError(null);
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busy) return;
    const button = (event.nativeEvent as SubmitEvent).submitter;
    const allowDuplicate = error === "duplicate_title" && button?.getAttribute("value") === "duplicate";
    if (!title.trim()) {
      setError("invalid_title");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const [cover, panorama] = await Promise.all([coverDraft.change(), panoramaDraft.change()]);
      onSaved(await saveSeries({ id: series?.id ?? null, title, genres: names, description, cover, panorama, allowDuplicate }));
    } catch (reason) {
      setError(catalogError(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal title={series ? copy.edit : copy.add} busy={busy} onCancel={onCancel}>
      <form onSubmit={(event) => void submit(event)}>
        <fieldset disabled={busy} className="modal__fields">
          <label className="field">{copy.title}
            <input autoFocus value={title} maxLength={200} required
              aria-invalid={error === "invalid_title"}
              onChange={(event) => { setTitle(event.target.value); setError(null); }} />
          </label>
          <fieldset className="genre-picker">
            <legend>{copy.genres}</legend>
            <div className="genre-picker__options">
              {options.map((name) => <label key={name} className="genre-choice">
                <input type="checkbox" checked={names.includes(name)}
                  onChange={(event) => toggle(name, event.target.checked)} />{name}
              </label>)}
            </div>
            <div className="field-row">
              <input aria-label={copy.newGenre} placeholder={copy.newGenre} value={genreInput} maxLength={60}
                onChange={(event) => setGenreInput(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" && !event.nativeEvent.isComposing) {
                    event.preventDefault();
                    addGenre();
                  }
                }} />
              <button type="button" onClick={addGenre}>{copy.addGenre}</button>
            </div>
            <p className="field-hint">{copy.genreHint}</p>
          </fieldset>
          <label className="field">{copy.seriesDescription}
            <textarea rows={4} maxLength={10_000} value={description}
              onChange={(event) => setDescription(event.target.value)}
              aria-invalid={error === "invalid_series_description"} />
          </label>
          <CoverPicker coverKey={series?.coverKey ?? null} draft={coverDraft} busy={busy} />
          <CoverPicker panorama coverKey={series?.panoramaKey ?? null} draft={panoramaDraft} busy={busy} />
        </fieldset>
        {error && <p className="catalog-error" role="alert">{copy.errors[error]}</p>}
        <div className="modal__actions">
          <button type="button" onClick={onCancel} disabled={busy}>{copy.cancel}</button>
          {error === "duplicate_title" ?
            <button type="submit" className="primary" value="duplicate">{copy.saveDuplicate}</button> :
            <button type="submit" className="primary" disabled={busy}>{busy ? copy.saving : copy.save}</button>}
        </div>
      </form>
    </Modal>
  );
}
