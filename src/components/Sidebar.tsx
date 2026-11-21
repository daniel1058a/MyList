import { useLocale } from "../locales/locale";

export type View = "library" | "favorites" | "continueWatching" | "statistics" | "settings";

const sections = [
  { view: "library", icon: "▤" },
  { view: "favorites", icon: "☆" },
  { view: "continueWatching", icon: "▷" },
  { view: "statistics", icon: "▥" },
  { view: "settings", icon: "⚙" },
] as const;

export function Sidebar({ view, onNavigate }: { view: View; onNavigate: (view: View) => void }) {
  const { t } = useLocale();
  const { navigation } = t;
  return (
    <aside className="sidebar">
      <div className="brand">
        <span className="brand__mark" aria-hidden="true" />
        {t.appName}
      </div>
      <nav aria-label={navigation.label}>
        <ul className="sidebar__list">
          {sections.map(({ view: item, icon }) => (
            <li className="sidebar__link" key={item}>
              <button type="button" aria-current={item === view ? "page" : undefined}
                className={item === view ? "sidebar__current" : undefined}
                onClick={() => onNavigate(item)}><span className="sidebar__icon" aria-hidden="true">{icon}</span>{navigation[item]}</button>
            </li>
          ))}
        </ul>
      </nav>
    </aside>
  );
}
