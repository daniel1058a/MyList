import { useLocale } from "../locales/locale";

export function StarRating({ value }: { value: number | null }) {
  const { t, formatRating } = useLocale();
  const score = value === null ? null : formatRating(value, 2);
  return (
    <div className="star-rating">
      <span className="star-rating__stars" aria-hidden="true">
        {Array.from({ length: 5 }, (_, index) => {
          const fill = Math.min(1, Math.max(0, (value ?? 0) / 2 - index)) * 100;
          return <span className="star-rating__star" key={index}>★
            <span style={{ width: fill + "%" }}>★</span>
          </span>;
        })}
      </span>
      {score === null ? <span className="star-rating__empty">{t.watch.noRating}</span> : (
        <strong><span className="visually-hidden">{t.watch.averageValue(score)}</span>
          <span aria-hidden="true">{score}<small>/10</small></span>
        </strong>
      )}
    </div>
  );
}
