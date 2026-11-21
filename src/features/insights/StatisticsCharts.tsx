import { useId, useState, type ReactNode } from "react";
import { useLocale } from "../../locales/locale";
import type { DashboardSelection } from "../../services/insights";

export type ChartEntry = { count: number; selection: DashboardSelection; color?: string };
type Explore = (selection: DashboardSelection) => void;

function tooltip(entry: ChartEntry, total: number, locale: ReturnType<typeof useLocale>) {
  return locale.t.insights.chartValue(entry.selection.label, locale.count(entry.count), locale.percent(total ? entry.count / total : 0));
}

function point(fraction: number) {
  const radians = (fraction - .25) * 2 * Math.PI;
  return [60 + 47 * Math.cos(radians), 60 + 47 * Math.sin(radians)].join(" ");
}

function ChartMark({ entry, total, onExplore, children, className }: {
  entry: ChartEntry;
  total: number;
  onExplore: Explore;
  children: ReactNode;
  className?: string;
}) {
  const locale = useLocale();
  const copy = locale.t.insights;
  const description = tooltip(entry, total, locale);
  return entry.count > 0 ? (
    <button type="button" className={className} title={description}
      aria-label={copy.explore(description)} onClick={() => onExplore(entry.selection)}>{children}</button>
  ) : <span className={className} title={description}>{children}</span>;
}

export function Donut({ title, entries, total, center, caption, onExplore }: {
  title: string;
  entries: ChartEntry[];
  total: number;
  center: string;
  caption: string;
  onExplore: Explore;
}) {
  const locale = useLocale();
  const copy = locale.t.insights;
  const heading = useId();
  let position = 0;
  return (
    <section className="insight-panel" aria-labelledby={heading}>
      <h2 id={heading}>{title}</h2>
      {total === 0 ? <p className="field-hint">{copy.noData}</p> : (
        <div className="donut">
          <div className="donut__plot">
            <svg viewBox="0 0 120 120" aria-hidden="true">
              <circle className="donut__track" cx="60" cy="60" r="47" />
              {entries.filter((entry) => entry.count > 0).map((entry) => {
                const size = entry.count / total;
                const start = position;
                position += size;
                const props = { className: "donut__segment", stroke: entry.color, onClick: () => onExplore(entry.selection) };
                const title = <title>{tooltip(entry, total, locale)}</title>;
                return size === 1
                  ? <circle key={entry.selection.value} {...props} cx="60" cy="60" r="47">{title}</circle>
                  : <path key={entry.selection.value} {...props}
                    d={["M", point(start), "A 47 47 0", size > .5 ? 1 : 0, 1, point(position)].join(" ")}>{title}</path>;
              })}
            </svg>
            <div className="donut__center"><strong>{center}</strong><span>{caption}</span></div>
          </div>
          <ul className="donut__legend">
            {entries.map((entry) => <li key={entry.selection.value}>
              <ChartMark entry={entry} total={total} onExplore={onExplore} className="chart-legend-row">
                <span className="chart-dot" style={{ backgroundColor: entry.color }} aria-hidden="true" />
                <span className="chart-legend-row__label">{entry.selection.label}</span>
                <strong>{locale.count(entry.count)}</strong><small>{locale.percent(total ? entry.count / total : 0)}</small>
              </ChartMark>
            </li>)}
          </ul>
        </div>
      )}
    </section>
  );
}

export function GenreChart({ entries, total, onExplore }: {
  entries: ChartEntry[];
  total: number;
  onExplore: Explore;
}) {
  const locale = useLocale();
  const copy = locale.t.insights;
  const [expanded, setExpanded] = useState(false);
  const listId = useId();
  return (
    <section className="insight-panel">
      <h2>{copy.genres}</h2>
      {entries.length === 0 ? <p className="field-hint">{copy.noData}</p> : <>
        <ul className="genre-bars" id={listId}>
          {(expanded ? entries : entries.slice(0, 5)).map((entry) => <li key={entry.selection.value}>
            <ChartMark entry={entry} total={total} onExplore={onExplore} className="genre-bar">
              <span>{entry.selection.label}</span><strong>{locale.count(entry.count)}</strong>
              <small>{locale.percent(total ? entry.count / total : 0)}</small>
              <span className="genre-bar__track" aria-hidden="true"><span style={{ width: (entry.count / total * 100) + "%" }} /></span>
            </ChartMark>
          </li>)}
        </ul>
        {entries.length > 5 && <button type="button" className="text-button genre-bars__toggle" aria-expanded={expanded}
          aria-controls={listId} onClick={() => setExpanded((current) => !current)}>
          {expanded ? copy.fewerGenres : copy.allGenres(entries.length)}
        </button>}
      </>}
    </section>
  );
}

export function Histogram({ entries, total, onExplore }: { entries: ChartEntry[]; total: number; onExplore: Explore }) {
  const locale = useLocale();
  const copy = locale.t.insights;
  const maxCount = Math.max(1, ...entries.map((entry) => entry.count));
  const interval = Math.ceil(maxCount / 4);
  const ticks = Array.from({ length: Math.ceil(maxCount / interval) + 1 }, (_, index) => index * interval);
  const ceiling = ticks[ticks.length - 1];
  return (
    <section className="insight-panel">
      <h2>{copy.ratings}</h2>
      {total === 0 && <p className="field-hint">{copy.noRatings}</p>}
      <p className="histogram__axis-title">{copy.contents}</p>
      <div className="histogram">
        <div className="histogram__grid" aria-hidden="true">
          {ticks.map((tick) => <div className="histogram__tick" key={tick} style={{ bottom: (tick / ceiling * 100) + "%" }}>
            <span>{locale.count(tick)}</span>
          </div>)}
        </div>
        <ul className="histogram__marks">
          {entries.map((entry) => <li key={entry.selection.value} data-high={Number(entry.selection.value) >= 9}>
            <ChartMark entry={entry} total={total} onExplore={onExplore} className="histogram__mark">
              <span className="histogram__bar" aria-hidden="true" style={{ height: (entry.count / ceiling * 100) + "%" }} />
              <span className="visually-hidden">{entry.selection.label} · {locale.count(entry.count)}</span>
            </ChartMark>
          </li>)}
        </ul>
        <div className="histogram__labels" aria-hidden="true">
          {entries.map((entry) => <span key={entry.selection.value}>{entry.selection.label}</span>)}
        </div>
      </div>
      <p className="histogram__caption">{copy.ratingAxis}</p>
    </section>
  );
}
