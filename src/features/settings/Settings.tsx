import { useEffect, useState, type ReactNode } from "react";
import { useLocale } from "../../locales/locale";
import { catalogError, type CatalogError } from "../../services/catalog";
import type { Preferences, VisualPreferences } from "../../services/preferences";
import { accessTaxonomy, type Taxonomy, type Term, type TermGroup } from "../../services/taxonomy";
import { TermDialog, type TermAction } from "./TermDialog";

export function Settings({ preferences, onVisualChange, onTaxonomyChange, notice, backupsPanel }: {
  preferences: Preferences | null;
  onVisualChange: (value: Partial<VisualPreferences>) => void;
  onTaxonomyChange: (data: Taxonomy) => void;
  notice: ReactNode;
  backupsPanel: ReactNode;
}) {
  const { t } = useLocale();
  const copy = t.settings;
  const [data, setData] = useState<Taxonomy | null>(null);
  const [error, setError] = useState<CatalogError | null>(null);
  const [attempt, setAttempt] = useState(0);
  const [dialog, setDialog] = useState<TermAction | null>(null);

  useEffect(() => {
    let active = true;
    void accessTaxonomy(null).then((next) => {
      if (active) setData(next);
    }).catch((reason: unknown) => {
      if (active) setError(catalogError(reason));
    });
    return () => { active = false; };
  }, [attempt]);

  return (
    <main className="library settings" aria-labelledby="settings-title">
      <header className="library__header"><div>
        <h1 id="settings-title">{t.navigation.settings}</h1>
        <p>{copy.description}</p>
      </div></header>
      <section className="settings__panel" aria-labelledby="appearance-title">
        <h2 id="appearance-title">{copy.appearance}</h2>
        {preferences && <div className="settings__appearance">
          <label className="field">{copy.language}
            <select aria-label={copy.language} value={preferences.language ?? "es"}
              onChange={(event) => onVisualChange({ language: event.target.value === "en" ? "en" : "es" })}>
              <option value="es" lang="es">Español</option>
              <option value="en" lang="en">English</option>
            </select>
          </label>
          <label className="field">{copy.theme}
            <select aria-label={copy.theme} value={preferences.theme}
              onChange={(event) => onVisualChange({ theme: event.target.value as VisualPreferences["theme"] })}>
              {Object.entries(copy.themes).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
            </select>
          </label>
          <label className="field">{copy.coverSize}
            <select aria-label={copy.coverSize} value={preferences.coverSize}
              onChange={(event) => onVisualChange({ coverSize: event.target.value as VisualPreferences["coverSize"] })}>
              {Object.entries(copy.sizes).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
            </select>
          </label>
        </div>}
        {notice}
      </section>
      {backupsPanel}
      <section className="settings__management" aria-labelledby="management-title">
        <h2 id="management-title">{copy.management}</h2>
        {!data && !error && <p role="status">{copy.loading}</p>}
        {error && <div className="catalog-notice">
          <p role="alert" className="catalog-error">{t.library.errors[error]}</p>
          <button type="button" onClick={() => {
            setError(null);
            setAttempt((value) => value + 1);
          }}>{t.storage.retry}</button>
        </div>}
        {data && <div className="settings__groups">
          <TermList group="genre" terms={data.genres} onAction={setDialog} />
          <TermList group="category" terms={data.categories} onAction={setDialog} />
        </div>}
      </section>
      {dialog && <TermDialog action={dialog} categories={data?.categories ?? []} onCancel={() => setDialog(null)}
        onSave={(next) => {
          setData(next);
          onTaxonomyChange(next);
          setDialog(null);
        }} />}
    </main>
  );
}

function TermList({ group, terms, onAction }: {
  group: TermGroup;
  terms: Term[];
  onAction: (action: TermAction) => void;
}) {
  const { t } = useLocale();
  const copy = t.settings;
  const genre = group === "genre";
  const heading = "settings-" + group;
  const usage = genre ? copy.genreUsage : copy.categoryUsage;
  return (
    <section className="settings__panel" aria-labelledby={heading}>
      <header className="settings__panel-header">
        <h3 id={heading}>{genre ? copy.genres : copy.categories}</h3>
        <button type="button" className="primary" onClick={() => onAction({ kind: "save", group, term: null })}>
          {genre ? copy.addGenre : copy.addCategory}
        </button>
      </header>
      {terms.length === 0 ? <p className="field-hint">{genre ? copy.noGenres : copy.noCategories}</p> :
        <ul className="term-list">
          {terms.map((term) => <li className="term-list__item" key={term.id}>
            <div className="term-list__label"><strong>{term.name}</strong><small>{usage(term.usage)}</small></div>
            <div className="term-list__actions">
              <button type="button" aria-label={copy.rename + " " + term.name}
                onClick={() => onAction({ kind: "save", group, term })}>{copy.rename}</button>
              <button type="button" aria-label={copy.remove + " " + term.name}
                onClick={() => onAction({ kind: "delete", group, term })}>{copy.remove}</button>
            </div>
          </li>)}
        </ul>}
    </section>
  );
}
