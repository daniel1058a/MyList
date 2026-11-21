import { Cover } from "../../components/Cover";
import { StarRating } from "../../components/StarRating";
import { useLocale } from "../../locales/locale";
import { readStatistics, type DashboardSelection, type Statistics } from "../../services/insights";
import { watchStatuses } from "../content/WatchControls";
import { Donut, GenreChart, Histogram, type ChartEntry } from "./StatisticsCharts";
import { useInsights } from "./useInsights";

function Metrics({ data }: { data: Statistics }) {
  const { t, count, formatRating } = useLocale();
  const copy = t.insights;
  const items = [
    { label: copy.series, value: data.seriesCount, icon: "▤", tone: "blue" },
    { label: copy.contents, value: data.contentCount, icon: "▷", tone: "blue" },
    { label: copy.favorites, value: data.favorites, icon: "♥", tone: "rose" },
    { label: copy.allEpisodes, value: data.totalEpisodes, icon: "◷", tone: "gold",
      detail: copy.episodeBreakdown(count(data.activeEpisodes), count(data.pastEpisodes)) },
    { label: copy.completed, value: data.completedViews, icon: "✓", tone: "green" },
    { label: copy.average, value: data.average === null ? t.watch.noRating : formatRating(data.average, 2) + "/10",
      icon: "★", tone: "gold" },
  ];
  return (
    <dl className="insight-metrics">
      {items.map(({ label, value, icon, tone, detail }) => <div key={label} data-tone={tone}>
        <dt><span className="insight-metrics__icon" aria-hidden="true">{icon}</span>{label}</dt>
        <dd data-score={label === copy.average} data-unrated={value === t.watch.noRating}>
          {typeof value === "number" ? count(value) : value}
          {detail && <p className="insight-metrics__detail">{detail}</p>}
        </dd>
      </div>)}
    </dl>
  );
}

function Ranking({ ranking, canOpenSeries, onOpenSeries }: {
  ranking: Statistics["ranking"];
  canOpenSeries: boolean;
  onOpenSeries: (id: number) => void;
}) {
  const { t } = useLocale();
  const copy = t.insights;
  return (
    <section className="insight-panel insight-ranking" aria-labelledby="insight-ranking-title">
      <h2 id="insight-ranking-title">{copy.ranking}</h2>
      {ranking.length === 0 ? <p className="field-hint">{copy.noRatings}</p> : <ol className="insight-ranking__grid">
        {ranking.map((item, index) => <li key={item.id} className="ranking-card">
            <div className="ranking-card__poster">
              <Cover coverKey={item.coverKey} />
              <span className="ranking-card__position"><span className="visually-hidden">{copy.position} </span>{index + 1}</span>
            </div>
            <div className="ranking-card__info">
              <strong className="ranking-card__title">{item.title}</strong>
              <StarRating value={item.average} />
            </div>
          <button type="button" className="ranking-card__link" aria-label={copy.openRank(item.title)}
            disabled={!canOpenSeries} onClick={() => onOpenSeries(item.id)} />
        </li>)}
      </ol>}
    </section>
  );
}

function Charts({ data, onExplore }: { data: Statistics; onExplore: (selection: DashboardSelection) => void }) {
  const { t, count, percent, formatRating } = useLocale();
  const copy = t.insights;
  const statuses: ChartEntry[] = watchStatuses.map((status) => ({
    count: data.statuses.find((item) => item.label === status)?.count ?? 0,
    selection: { kind: "status", value: status, label: t.watch.states[status] },
    color: "var(--insight-" + status + ")",
  }));
  const categories: ChartEntry[] = data.categories.map((item, index) => ({
    count: item.count,
    selection: { kind: "category", value: item.id, label: item.label },
    color: "hsl(" + Math.round(index * 137.508 % 360) + " 58% var(--insight-category-lightness))",
  }));
  const genres: ChartEntry[] = data.genres.map((item) => ({
    count: item.count, selection: { kind: "genre", value: item.id, label: item.label },
  }));
  const ratings: ChartEntry[] = Array.from({ length: 10 }, (_, index) => {
    const value = index + 1;
    return {
      count: data.ratings.find((item) => item.label === String(value))?.count ?? 0,
      selection: { kind: "rating", value, label: value === 10 ? "10" : value + "–" + formatRating(value + .9, 1) },
    };
  });
  const rated = data.ratings.reduce((total, item) => total + item.count, 0);

  return (
    <div className="insight-panels">
      <div className="insight-panels__column">
        <Donut title={copy.statuses} entries={statuses} total={data.contentCount}
          center={percent(data.contentCount ? statuses[2].count / data.contentCount : 0)} caption={copy.finishedShare} onExplore={onExplore} />
        <Histogram entries={ratings} total={rated} onExplore={onExplore} />
      </div>
      <div className="insight-panels__column">
        <Donut title={copy.categories} entries={categories} total={data.contentCount}
          center={count(data.contentCount)} caption={copy.categoryTotal} onExplore={onExplore} />
        <GenreChart entries={genres} total={data.seriesCount} onExplore={onExplore} />
      </div>
    </div>
  );
}

export function StatisticsView({ revision, onExplore, canOpenSeries, onOpenSeries }: {
  revision: object | null;
  onExplore: (selection: DashboardSelection) => void;
  canOpenSeries: boolean;
  onOpenSeries: (id: number) => void;
}) {
  const { t } = useLocale();
  const copy = t.insights;
  const { data, error, retry } = useInsights(readStatistics, revision);
  return (
    <main className="library statistics" aria-labelledby="statistics-title">
      <header className="library__header"><div>
        <h1 id="statistics-title">{t.navigation.statistics}</h1><p>{copy.statisticsDescription}</p>
      </div></header>
      {error && <div className="catalog-notice"><p role="alert" className="catalog-error">{t.library.errors[error]}</p>
        <button type="button" onClick={retry}>{t.storage.retry}</button></div>}
      {!data && !error && <p className="catalog-notice" role="status">{copy.loading}</p>}
      {data && <>
        <Metrics data={data} />
        {data.seriesCount === 0 && <p className="catalog-notice">{copy.emptyStatistics}</p>}
        <Charts data={data} onExplore={onExplore} />
        <Ranking ranking={data.ranking} onOpenSeries={onOpenSeries} canOpenSeries={canOpenSeries} />
      </>}
    </main>
  );
}
