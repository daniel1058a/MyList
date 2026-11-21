import { useLocale } from "../locales/locale";
import type { WatchStatus } from "../services/watch";

export function StatusFilter({ value, onChange }: {
  value: WatchStatus | null;
  onChange: (value: WatchStatus | null) => void;
}) {
  const { t } = useLocale();
  return (
    <label className="field">{t.filters.status}
      <select aria-label={t.filters.status} value={value ?? ""} onChange={(event) => onChange((event.target.value || null) as WatchStatus | null)}>
        <option value="">{t.filters.allStatuses}</option>
        {Object.entries(t.watch.states).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
      </select>
    </label>
  );
}
