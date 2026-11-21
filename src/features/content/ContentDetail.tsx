import type { ReactNode } from "react";
import { Cover } from "../../components/Cover";
import { useLocale } from "../../locales/locale";
import type { Content } from "../../services/content";
import { WatchDates, WatchSessionList } from "./WatchHistory";

export function ContentDetail({ seriesId, item, category, controls, notice, busy, onBack, onEdit, onDelete }: {
  seriesId: number;
  item: Content;
  category: string | undefined;
  controls: ReactNode;
  notice: ReactNode;
  busy: boolean;
  onBack: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { t } = useLocale();
  const copy = t.content;
  return (
    <div className="content-detail">
      <button className="text-button content-detail__back" type="button" onClick={onBack}>
        <span aria-hidden="true">←</span> {t.seriesDetail.back}
      </button>
      <header className="content-detail__header">
        <Cover coverKey={item.coverKey} eager />
        <div className="content-detail__info">
          <h1 id="content-title" tabIndex={-1}>{item.title}</h1>
          <p className="content-card__meta">{category}{item.releaseYear !== null && <> · {item.releaseYear}</>}</p>
          {item.watch.ratingTenths === null && <p className="field-hint">{t.watch.noRating}</p>}
          {controls}
          <WatchDates session={item.watch.session} />
          <div className="content-detail__actions">
            <button type="button" disabled={busy} onClick={onEdit}>{copy.edit}</button>
            <button type="button" className="text-button" disabled={busy} onClick={onDelete}>{copy.remove}</button>
          </div>
          {notice}
        </div>
      </header>
      {item.description && <section className="content-detail__text"><h2>{copy.description}</h2><p>{item.description}</p></section>}
      {item.opinion && <section className="content-detail__text"><h2>{copy.opinion}</h2><p>{item.opinion}</p></section>}
      <section aria-labelledby="history-title" className="content-detail__history">
        <h2 id="history-title">{t.watch.history}</h2>
        <WatchSessionList key={item.id} seriesId={seriesId} contentId={item.id} revision={item.watch.session} />
      </section>
    </div>
  );
}
