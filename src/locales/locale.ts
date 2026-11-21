import { createContext, useContext } from "react";
import { en, type Messages } from "./en";
import { es } from "./es";
import type { DashboardSelection } from "../services/insights";

export type Language = "es" | "en";

function createLocale(language: Language) {
  const t: Messages<typeof es> = language === "en" ? en : es;
  const count = new Intl.NumberFormat(language);
  const percent = new Intl.NumberFormat(language, { style: "percent", maximumFractionDigits: 1 });
  const date = new Intl.DateTimeFormat(language, { dateStyle: "medium", timeZone: "UTC" });
  const timestamp = new Intl.DateTimeFormat(language, { dateStyle: "medium", timeStyle: "short" });
  const formatDate = (formatter: Intl.DateTimeFormat, value: string) => {
    const parsed = new Date(value);
    return Number.isNaN(parsed.valueOf()) ? value : formatter.format(parsed);
  };
  return {
    language, t, count: (value: number) => count.format(value),
    percent: (value: number) => percent.format(value),
    formatRating: (value: number, digits: 1 | 2) => value.toLocaleString(language, { maximumFractionDigits: digits }),
    date: (value: string) => formatDate(date, value),
    timestamp: (value: string) => formatDate(timestamp, value),
  };
}

export const locales = { es: createLocale("es"), en: createLocale("en") };
export const LocaleContext = createContext(locales.es);
export const useLocale = () => useContext(LocaleContext);

export function selectionLabel(selection: DashboardSelection, locale: ReturnType<typeof useLocale>) {
  if (selection.kind === "status") return locale.t.watch.states[selection.value];
  if (selection.kind === "rating") return selection.value === 10 ? "10"
    : selection.value + "–" + locale.formatRating(selection.value + .9, 1);
  return selection.label;
}
