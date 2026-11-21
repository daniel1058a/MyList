import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../src/App";
import type { WatchSnapshot } from "../src/services/watch";
import { defaultPreferences, type Preferences } from "../src/services/preferences";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: (key: string) => "/thumbnail/" + key,
}));

const mockInvoke = vi.mocked(invoke);
const emptySeriesMetadata = { contents: [], createdAt: "", updatedAt: "", description: "", panoramaKey: null, ratingAverage: null };

function initialWatch(id: number): WatchSnapshot {
  return {
    session: { id, watchedEpisodes: 0, status: "to_watch", startedOn: null, finishedOn: null, completed: false, active: true },
    totalEpisodes: null, ratingTenths: null, completedViews: 0,
  };
}

// jsdom does not implement the native dialog methods used by WebView2.
Object.defineProperties(HTMLDialogElement.prototype, {
  showModal: { configurable: true, value(this: HTMLDialogElement) { this.open = true; } },
  close: { configurable: true, value(this: HTMLDialogElement) { this.open = false; } },
});

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation(async (command) => (
    command === "storage_health" ? { schemaVersion: 6 } :
      command === "catalog_preferences" ? defaultPreferences : { series: [], genres: [] }
  ) as never);
});
afterEach(cleanup);

describe("MyList", () => {
  it("changes language immediately, persists the choice and keeps user content unchanged", async () => {
    let preferences = { ...defaultPreferences };
    const item = { ...emptySeriesMetadata, id: 1, title: "Órbita", favorite: false, genreIds: [], coverKey: null,
      ratingAverage: 8.5 };
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "catalog_preferences") {
        const value = (args as { value?: Preferences }).value;
        if (value) preferences = value;
        return preferences as never;
      }
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "catalog_taxonomy") return { genres: [], categories: [] } as never;
      if (command === "backup_access") return { kind: "status", status: { automatic: false, lastSuccess: null } } as never;
      return { series: [item], genres: [] } as never;
    });
    const first = render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
    fireEvent.change(await screen.findByRole("combobox", { name: "Idioma" }), { target: { value: "en" } });
    expect(screen.getByRole("heading", { name: "Backups and exports" })).toBeTruthy();
    expect(document.documentElement.lang).toBe("en");
    fireEvent.click(screen.getByRole("button", { name: "Library" }));
    expect(await screen.findByRole("heading", { name: "Órbita" })).toBeTruthy();
    expect(screen.getByText("Average 8.5/10")).toBeTruthy();
    expect(preferences.language).toBe("en");
    first.unmount();
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Library" })).toBeTruthy();
    expect(document.documentElement.lang).toBe("en");
  });

  it("requires explicit confirmation of a validated backup and reopens the restored preferences", async () => {
    let restored = false;
    let previewed = false;
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "catalog_preferences") return { ...defaultPreferences, language: restored ? "en" : "es" } as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "catalog_taxonomy") return { genres: [], categories: [] } as never;
      if (command === "backup_access") {
        const action = (args as { action: { kind: string; token?: string } }).action;
        if (action.kind === "preview") {
          previewed = true;
          return { kind: "preview", token: "validated", summary: { schemaVersion: 6, kind: "manual",
            createdAt: "2026-02-14T13:10:00Z", seriesCount: 4, contentCount: 8, imageCount: 3 } } as never;
        }
        if (action.kind === "restore") {
          if (!previewed || action.token !== "validated") throw new Error("invalid workflow");
          restored = true;
          return { kind: "restored" } as never;
        }
        return { kind: "status", status: { automatic: true, lastSuccess: null } } as never;
      }
      return { series: [], genres: [] } as never;
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
    fireEvent.click(screen.getByRole("button", { name: "Restaurar desde ZIP" }));
    const dialog = within(await screen.findByRole("dialog", { name: "Confirmar restauración" }));
    expect(dialog.getByText(/sustituirá la biblioteca/)).toBeTruthy();
    expect(dialog.getByText("8")).toBeTruthy();
    expect(restored).toBe(false);
    fireEvent.click(dialog.getByRole("button", { name: "Restaurar biblioteca" }));
    expect(await screen.findByText("Library restored. A precautionary backup was saved.")).toBeTruthy();
    expect(restored).toBe(true);
  });

  it("opens chart selections in the library, preserves stored filters and opens ranked series", async () => {
    const pilot = { id: 11, title: "Piloto", categoryId: 1, releaseYear: null, description: "", opinion: "", coverKey: null,
      watch: { ...initialWatch(21), ratingTenths: 99 } };
    const finale = { ...pilot, id: 12, title: "Final", categoryId: 2,
      watch: { ...initialWatch(22), ratingTenths: 100 } };
    const series = { ...emptySeriesMetadata, id: 1, title: "Órbita", genreIds: [1], coverKey: "poster", favorite: false,
      contents: [pilot, finale].map((item) => ({
        title: item.title, status: item.watch.session.status, categoryId: item.categoryId, ratingTenths: item.watch.ratingTenths,
      })) };
    const genres = Array.from({ length: 6 }, (_, index) => ({ id: index + 1, name: "Género " + (index + 1) }));
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return { ...defaultPreferences, query: "búsqueda previa" } as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "content_list") return {
        items: [pilot, finale], categories: [{ id: 1, name: "Corto" }, { id: 2, name: "Temporada" }], ratingAverage: 9.95,
      } as never;
      if (command === "library_statistics") return {
        seriesCount: 1, contentCount: 2, favorites: 0, totalEpisodes: 0, activeEpisodes: 0, pastEpisodes: 0,
        completedViews: 0, average: 9.95, statuses: [{ label: "to_watch", count: 2 }],
        genres: genres.map(({ id, name }) => ({ id, label: name, count: 1 })),
        categories: [{ id: 1, label: "Corto", count: 1 }, { id: 2, label: "Temporada", count: 1 }],
        ratings: [{ label: "9", count: 1 }, { label: "10", count: 1 }],
        ranking: [{ id: 1, title: series.title, average: 9.95, coverKey: series.coverKey }],
      } as never;
      return { series: [series], genres } as never;
    });
    const { container } = render(<App />);
    const statistics = () => fireEvent.click(screen.getByRole("button", { name: "Estadísticas" }));
    statistics();
    const expand = await screen.findByRole("button", { name: "Ver los 6 géneros" });
    expect(container.querySelectorAll(".genre-bars li")).toHaveLength(5);
    fireEvent.click(expand);
    expect(container.querySelectorAll(".genre-bars li")).toHaveLength(6);
    expect(screen.getByRole("button", { name: "Ver menos géneros" }).getAttribute("aria-expanded")).toBe("true");
    expect(container.querySelectorAll(".histogram__marks li")).toHaveLength(10);
    expect(container.querySelector(".ranking-card img")?.getAttribute("src")).toBe("/thumbnail/poster");
    const category = screen.getByRole("button", { name: /Explorar Corto/ });
    expect(category.getAttribute("title")).toMatch(/Corto · 1 \(50\s?%\)/);
    fireEvent.click(category);
    expect(screen.getByRole("heading", { name: "Biblioteca" })).toBe(document.activeElement);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir serie" }));
    await screen.findByRole("button", { name: "Ver detalle de Piloto" });
    expect(screen.queryByRole("button", { name: "Ver detalle de Final" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Quitar filtro de estadísticas" }));
    expect(screen.getByRole("button", { name: "Ver detalle de Final" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Volver a Biblioteca" }));
    fireEvent.click(screen.getByRole("button", { name: "Quitar filtro de estadísticas" }));
    expect((screen.getByRole("searchbox") as HTMLInputElement).value).toBe("búsqueda previa");
    statistics();
    const rank = await screen.findByRole("button", { name: "Abrir serie Órbita" });
    fireEvent.click(rank);
    expect(screen.getByRole("heading", { name: "Órbita", level: 1 })).toBe(document.activeElement);
    expect(within(screen.getByRole("navigation", { name: "Pestañas abiertas" })).getAllByRole("button", { name: "Órbita" })).toHaveLength(1);
    expect(mockInvoke.mock.calls.filter(([command, args]) => command === "catalog_preferences" &&
      (args as { value?: unknown })?.value != null)).toHaveLength(0);
  });

  it("opens only the active watching result in its series tab and refreshes statistics after finishing it", async () => {
    const series = { ...emptySeriesMetadata, id: 1, title: "Órbita", genreIds: [], coverKey: null, favorite: true };
    const original = { ...initialWatch(21), totalEpisodes: 2 };
    const watching = { ...original.session, watchedEpisodes: 1, status: "watching" };
    let finished = false;
    const snapshot = () => ({ ...watching, watchedEpisodes: finished ? 2 : 1,
      status: finished ? "finished" : "watching", completed: finished });
    const contents = () => ({ items: [{
      id: 11, title: "Piloto", categoryId: 1, releaseYear: null, description: "", opinion: "", coverKey: null,
      watch: { ...original, session: snapshot() },
    }], categories: [{ id: 1, name: "Corto" }], ratingAverage: null });
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return { ...defaultPreferences, contentFilters: {
        1: { categoryId: 2, status: "finished" },
      } } as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "watch_history") return [snapshot()] as never;
      if (command === "content_list") return contents() as never;
      if (command === "watch_update") { finished = true; return contents() as never; }
      if (command === "continue_watching") return (finished ? [] : [{
        id: 11, seriesId: 1, seriesTitle: "Órbita", title: "Piloto", category: "Corto",
        coverKey: null, sessionId: 21, watchedEpisodes: 1, totalEpisodes: 2, ratingTenths: 95,
      }]) as never;
      if (command === "library_statistics") return {
        seriesCount: 1, contentCount: 2, favorites: 1, activeEpisodes: finished ? 2 : 1,
        pastEpisodes: 5, totalEpisodes: finished ? 7 : 6, completedViews: finished ? 2 : 1, average: 9.5,
        statuses: [{ label: snapshot().status, count: 1 }], genres: [{ id: 1, label: "Drama", count: 1 }],
        categories: [{ id: 1, label: "Corto", count: 2 }], ranking: [{ id: 1, title: "Órbita", average: 9.5, coverKey: null }],
        ratings: [{ label: "9", count: 1 }],
      } as never;
      return { series: [series], genres: [] } as never;
    });
    const scroll = vi.spyOn(window, "scrollTo").mockImplementation(() => {});
    try {
      render(<App />);
      fireEvent.click(screen.getByRole("button", { name: "Estadísticas" }));
      expect(await screen.findByText("1 en sesiones actuales · 5 en sesiones anteriores")).toBeTruthy();
      expect(screen.getByRole("heading", { name: "Distribución de calificaciones" })).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: "Continuar viendo" }));
      const item = await screen.findByRole("button", { name: "Ver detalle de Piloto" });
      await waitFor(() => expect((item as HTMLButtonElement).disabled).toBe(false));
      expect(screen.getByText("Órbita · Corto")).toBeTruthy();
      fireEvent.click(item);
      expect(await screen.findByRole("heading", { name: "Piloto", level: 1 })).toBe(document.activeElement);
      await screen.findByRole("heading", { name: "Sesión 1 · Activa" });
      fireEvent.click(screen.getByRole("button", { name: "Volver a Continuar viendo" }));
      const increment = await screen.findByRole("button", { name: "Sumar un episodio" });
      await waitFor(() => expect((increment as HTMLButtonElement).disabled).toBe(false));
      fireEvent.click(increment);
      expect(await screen.findByText("No tienes contenidos en curso")).toBeTruthy();
      expect(mockInvoke).toHaveBeenCalledWith("watch_update", {
        input: { seriesId: 1, contentId: 11, sessionId: 21, action: { kind: "step", delta: 1 } },
      });
      fireEvent.click(screen.getByRole("button", { name: "Estadísticas" }));
      expect(await screen.findByText("2 en sesiones actuales · 5 en sesiones anteriores")).toBeTruthy();
    } finally {
      scroll.mockRestore();
    }
  });

  it("shows the empty library and checks the local database", async () => {
    render(<App />);

    expect(screen.getByRole("navigation", { name: "Navegación principal" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Biblioteca" })).toBeTruthy();
    expect(screen.getByText("Comprobando SQLite…")).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Tu biblioteca está vacía" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Estadísticas" })).toBeTruthy();
    expect(await screen.findByText("SQLite disponible · Esquema 6")).toBeTruthy();
    expect(mockInvoke).toHaveBeenCalledWith("storage_health");
    expect(mockInvoke).toHaveBeenCalledWith("catalog_list");
  });

  it("shows a safe message for a storage error and can recheck", async () => {
    let healthCalls = 0;
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command !== "storage_health") return { series: [], genres: [] } as never;
      if (++healthCalls === 1) throw { code: "migration_failed", message: "private/database.sqlite3" };
      return { schemaVersion: 6 } as never;
    });
    render(<App />);

    expect(await screen.findByText("No se pudo actualizar el esquema de la base de datos.")).toBeTruthy();
    expect(screen.queryByText(/private/)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Reintentar" }));
    expect(screen.getByRole("status").textContent).toBe("Comprobando SQLite…");
    expect(await screen.findByText("SQLite disponible · Esquema 6")).toBeTruthy();
    expect(healthCalls).toBe(2);
  });

  it("does not display raw IPC error details", async () => {
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") throw "private/runtime/path";
      return { series: [], genres: [] } as never;
    });
    render(<App />);

    expect(await screen.findByText("No se pudo comprobar el almacenamiento local.")).toBeTruthy();
    expect(screen.queryByText(/private/)).toBeNull();
  });

  it("assigns a genre and explicitly confirms a duplicate", async () => {
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "series_save") {
        const input = (args as { input: { allowDuplicate: boolean } }).input;
        if (!input.allowDuplicate) throw { code: "duplicate_title" };
        return { series: [{ id: 1, title: "My series", favorite: false, ...emptySeriesMetadata, coverKey: null, genreIds: [1] }],
          genres: [{ id: 1, name: "Drama" }] } as never;
      }
      return { series: [], genres: [] } as never;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Añadir serie" }));
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByLabelText("Título"), { target: { value: "My series" } });
    fireEvent.change(within(dialog).getByLabelText("Descripción general"), { target: { value: "Línea uno\nLínea dos" } });
    fireEvent.change(within(dialog).getByLabelText("Nombre del nuevo género"), { target: { value: "Drama" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Añadir género" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Guardar" }));
    expect(await within(dialog).findByRole("alert")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Guardar de todos modos" }));
    expect(await screen.findByRole("heading", { name: "My series" })).toBeTruthy();
    expect(screen.getByText("Sin portada")).toBeTruthy();
    expect(mockInvoke).toHaveBeenCalledWith("series_save", {
      input: {
        id: null, title: "My series", genres: ["Drama"], description: "Línea uno\nLínea dos",
        allowDuplicate: true, cover: { action: "keep" }, panorama: { action: "keep" },
      },
    });
  });

  it("shows favorites, confirms deletion and preserves the previous view on failure", async () => {
    const item = { id: 1, title: "My series", favorite: true, ...emptySeriesMetadata, coverKey: null, genreIds: [] };
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "series_delete") throw { code: "catalog_unavailable" };
      if (command === "series_favorite") return { series: [{ ...item, favorite: false }], genres: [] } as never;
      return { series: [item], genres: [] } as never;
    });
    render(<App />);
    await screen.findByRole("heading", { name: "My series" });
    fireEvent.click(screen.getByRole("button", { name: "Favoritos" }));
    fireEvent.click(screen.getByRole("button", { name: "Eliminar serie" }));
    const dialog = screen.getByRole("dialog");
    expect(mockInvoke).not.toHaveBeenCalledWith("series_delete", { id: 1 });
    fireEvent.click(within(dialog).getByRole("button", { name: "Eliminar serie" }));
    expect(await within(dialog).findByRole("alert")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancelar" }));
    expect(screen.getByRole("heading", { name: "My series" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Quitar de favoritas" }));
    expect(await screen.findByText("Todavía no tienes favoritas")).toBeTruthy();
  });

  it("keeps open series unique and removes the tab after a confirmed cascade", async () => {
    const first = { id: 1, title: "Primera", favorite: false, ...emptySeriesMetadata, coverKey: null, genreIds: [] };
    const second = { ...first, id: 2, title: "Segunda" };
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "content_list") return { items: [], categories: [], ratingAverage: null } as never;
      return { series: command === "series_delete" ? [second] : [first, second], genres: [] } as never;
    });
    render(<App />);
    await screen.findByRole("heading", { name: "Primera" });
    const tabs = within(screen.getByRole("navigation", { name: "Pestañas abiertas" }));
    const open = (title: string) => fireEvent.click(within(
      screen.getByRole("heading", { name: title }).closest("li")!,
    ).getByRole("button", { name: "Abrir serie" }));
    open("Primera");
    await screen.findByText("Esta serie todavía no tiene contenidos");
    fireEvent.click(tabs.getByRole("button", { name: "Volver a Biblioteca" }));
    open("Primera");
    expect(tabs.getAllByRole("button", { name: "Primera" })).toHaveLength(1);
    await screen.findByText("Esta serie todavía no tiene contenidos");
    fireEvent.click(tabs.getByRole("button", { name: "Volver a Biblioteca" }));
    open("Segunda");
    expect(tabs.getByRole("button", { name: "Segunda" }).getAttribute("aria-current")).toBe("page");
    await screen.findByText("Esta serie todavía no tiene contenidos");
    fireEvent.click(tabs.getByRole("button", { name: "Volver a Biblioteca" }));
    const firstCard = within(screen.getByRole("heading", { name: "Primera" }).closest("li")!);
    fireEvent.click(firstCard.getByRole("button", { name: "Eliminar serie" }));
    const confirmation = within(screen.getByRole("dialog"));
    expect(confirmation.getByText(/y todos sus contenidos/)).toBeTruthy();
    fireEvent.click(confirmation.getByRole("button", { name: "Eliminar serie" }));
    await screen.findByRole("button", { name: "Abrir serie" });
    expect(tabs.queryByRole("button", { name: "Primera" })).toBeNull();
    expect(tabs.getByRole("button", { name: "Segunda" })).toBeTruthy();
  });

  it("validates content, saves multiline fields and reorders by keyboard controls", async () => {
    const series = { id: 1, title: "Órbita", favorite: false, ...emptySeriesMetadata, coverKey: null, genreIds: [] };
    const first = { id: 11, categoryId: 1, title: "Original", coverKey: null, watch: initialWatch(21),
      description: "", opinion: "", releaseYear: null };
    const next = { ...first, id: 12, title: "Especial", watch: initialWatch(22),
      description: "Línea uno\nLínea dos", opinion: "Primera opinión\nSegunda opinión", releaseYear: 2020 };
    const categories = [{ id: 1, name: "Corto" }];
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "content_list") return { items: [first], categories, ratingAverage: null } as never;
      if (command === "content_save") return { items: [first, next], categories, ratingAverage: null } as never;
      if (command === "content_reorder") return { items: [next, first], categories, ratingAverage: null } as never;
      return { series: [series], genres: [] } as never;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir serie" }));
    fireEvent.click(await screen.findByRole("button", { name: "Añadir contenido" }));
    const dialog = within(screen.getByRole("dialog"));
    const fill = (label: string, value: string) => fireEvent.change(dialog.getByLabelText(label), { target: { value } });
    fill("Título", next.title);
    fireEvent.click(dialog.getByRole("button", { name: "Guardar" }));
    expect(await dialog.findByText(/Selecciona una categoría o escribe/)).toBeTruthy();
    fill("Categoría", "1");
    fill("Año de estreno", "0");
    fireEvent.click(dialog.getByRole("button", { name: "Guardar" }));
    expect(await dialog.findByText(/Escribe un año entero/)).toBeTruthy();
    fill("Año de estreno", "2020");
    fill("Descripción", next.description);
    fill("Opinión personal", next.opinion);
    fireEvent.click(dialog.getByRole("button", { name: "Guardar" }));
    const special = await screen.findByRole("heading", { name: "Especial" });
    expect(mockInvoke).toHaveBeenCalledWith("content_save", { input: {
      id: null, seriesId: 1, title: next.title, category: "Corto", releaseYear: 2020,
      description: next.description, opinion: next.opinion, cover: { action: "keep" },
    } });
    const specialCard = within(special.closest("li")!);
    fireEvent.click(specialCard.getByText("Más acciones"));
    fireEvent.click(specialCard.getByRole("button", { name: "Subir" }));
    await screen.findByText("Orden guardado.");
    expect(screen.getAllByRole("heading", { level: 3 }).map((item) => item.textContent)).toEqual(["Especial", "Original"]);
    expect(mockInvoke).toHaveBeenCalledWith("content_reorder", { seriesId: 1, id: 12, targetId: 11, placement: "before" });
  });

  it("edits episode totals and decimal ratings, then reflects the average in the catalog", async () => {
    const series = { id: 1, title: "Órbita", favorite: false, ...emptySeriesMetadata, coverKey: null, genreIds: [], ratingAverage: null };
    const item = { id: 11, categoryId: 1, title: "Corto", coverKey: null,
      description: "", opinion: "", releaseYear: null, watch: initialWatch(21) };
    const contents = { items: [item], categories: [{ id: 1, name: "Corto" }], ratingAverage: null };
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "content_list") return contents as never;
      if (command === "watch_update") return {
        ...contents, ratingAverage: 8.5,
        items: [{ ...item, watch: { ...item.watch, totalEpisodes: 3, ratingTenths: 85,
          session: { ...item.watch.session, status: "watching", watchedEpisodes: 1 } } }],
      } as never;
      return { series: [series], genres: [] } as never;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir serie" }));
    fireEvent.click(await screen.findByRole("button", { name: "Editar seguimiento" }));
    const modal = within(screen.getByRole("dialog"));
    const fill = (label: string, value: string) => fireEvent.change(modal.getByLabelText(label), { target: { value } });
    fill("Episodios vistos", "4");
    fill("Total de episodios", "3");
    fireEvent.click(modal.getByRole("button", { name: "Guardar" }));
    expect(await modal.findByRole("alert")).toBeTruthy();
    expect(mockInvoke).not.toHaveBeenCalledWith("watch_update", expect.anything());
    fill("Episodios vistos", "1");
    fill("Calificación", "8,5");
    fireEvent.click(modal.getByRole("button", { name: "Guardar" }));
    expect((await screen.findByRole("progressbar")).getAttribute("value")).toBe("1");
    expect(mockInvoke).toHaveBeenCalledWith("watch_update", { input: {
      seriesId: 1, contentId: 11, sessionId: 21, action: {
        kind: "edit", progress: { watchedEpisodes: 1, totalEpisodes: 3 }, ratingTenths: 85,
        startedOn: null, finishedOn: null,
      },
    } });
    fireEvent.click(screen.getByRole("button", { name: "Volver a Biblioteca" }));
    expect(screen.getByText("Promedio 8,5/10")).toBeTruthy();
  });

  it("presents partial stars, a collapsible series description and independent image fallback", async () => {
    const description = "Primera línea\nSegunda línea\nTercera línea\nCuarta línea";
    const series = {
      ...emptySeriesMetadata, id: 1, title: "Horizonte", description, coverKey: "poster", panoramaKey: "wide",
      genreIds: [1], favorite: false, ratingAverage: 9,
      contents: [{ title: "Corto", status: "finished" }],
    };
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "content_list") return { items: [], categories: [], ratingAverage: 9 } as never;
      return { series: [series], genres: [{ id: 1, name: "Drama" }] } as never;
    });
    const { container } = render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir serie" }));
    const backdrop = () => container.querySelector(".series-hero__art img");
    expect(backdrop()?.getAttribute("src")).toBe("/thumbnail/wide");
    expect(container.querySelector(".series-hero__layout > .cover img")?.getAttribute("src")).toBe("/thumbnail/poster");
    fireEvent.error(backdrop()!);
    expect(backdrop()?.getAttribute("src")).toBe("/thumbnail/poster");
    expect(screen.getByText("Promedio 9/10")).toBeTruthy();
    const fills = [...container.querySelectorAll<HTMLElement>(".star-rating__star > span")].map((star) => star.style.width);
    expect(fills).toEqual(["100%", "100%", "100%", "100%", "50%"]);
    expect(within(screen.getByRole("list", { name: "Contenidos por estado" })).getByText(/Finalizado/).textContent).toBe("1 Finalizado");
    const expand = screen.getByRole("button", { name: "Leer más" });
    expect(screen.queryByText(/Cuarta línea/)).toBeNull();
    fireEvent.click(expand);
    expect(screen.getByText(/Cuarta línea/).textContent).toBe(description);
    expect(screen.getByRole("button", { name: "Leer menos" }).getAttribute("aria-expanded")).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "Editar serie" }));
    expect((within(screen.getByRole("dialog")).getByLabelText("Descripción general") as HTMLTextAreaElement).value).toBe(description);
    expect(within(screen.getByRole("dialog")).getByRole("button", { name: "Quitar imagen panorámica" })).toBeTruthy();
  });

  it("opens the content detail inside its series tab and restores the filter, focus and scroll", async () => {
    const series = { ...emptySeriesMetadata, id: 1, title: "Horizonte", coverKey: null, genreIds: [], favorite: false, ratingAverage: null };
    const watch = initialWatch(21);
    const item = {
      id: 11, categoryId: 1, title: "Prólogo", coverKey: null, description: "Introducción\nCompleta",
      opinion: "Una buena apertura", releaseYear: 2024, watch: { ...watch, ratingTenths: 85, totalEpisodes: 3,
        session: { ...watch.session, watchedEpisodes: 1, status: "watching", startedOn: "2026-10-20" } },
    };
    const categories = [{ id: 1, name: "Especial" }, { id: 2, name: "Corto" }];
    const preferences = { ...defaultPreferences, contentFilters: { 1: { categoryId: 1, status: "watching" } } };
    mockInvoke.mockImplementation(async (command) => {
      if (command === "catalog_preferences") return preferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "watch_history") return [item.watch.session] as never;
      if (command === "content_list") return { items: [item], categories, ratingAverage: 8.5 } as never;
      return { series: [series], genres: [] } as never;
    });
    const scroll = vi.spyOn(window, "scrollTo").mockImplementation(() => {});
    const position = vi.spyOn(window, "scrollY", "get").mockReturnValue(420);
    try {
      render(<App />);
      fireEvent.click(await screen.findByRole("button", { name: "Abrir serie" }));
      const open = await screen.findByRole("button", { name: "Ver detalle de Prólogo" });
      fireEvent.click(open);
      expect(screen.getByRole("heading", { level: 1, name: "Prólogo" })).toBe(document.activeElement);
      expect(screen.getAllByRole("button", { name: "Horizonte" })).toHaveLength(1);
      expect(screen.getByText("Especial · 2024", { selector: ".content-detail p" })).toBeTruthy();
      expect(screen.getByText(/Introducción Completa/).textContent).toBe(item.description);
      expect(screen.getByText(item.opinion)).toBeTruthy();
      expect(await screen.findByRole("heading", { name: "Sesión 1 · Activa" })).toBeTruthy();
      expect(screen.getAllByText(new Intl.DateTimeFormat("es", {
        dateStyle: "medium", timeZone: "UTC",
      }).format(new Date("2026-10-20")))).toHaveLength(2);
      expect(mockInvoke).toHaveBeenCalledWith("watch_history", { seriesId: 1, contentId: 11 });
      expect(scroll).toHaveBeenLastCalledWith(0, 0);
      fireEvent.click(screen.getByRole("button", { name: "Volver a la serie" }));
      expect((screen.getByLabelText("Filtrar por categoría") as HTMLSelectElement).value).toBe("1");
      expect((screen.getByLabelText("Filtrar por estado") as HTMLSelectElement).value).toBe("watching");
      expect(document.activeElement).toBe(open);
      expect(scroll).toHaveBeenLastCalledWith(0, 420);
    } finally {
      scroll.mockRestore();
      position.mockRestore();
    }
  });

  it("confirms a manual reset and uses the active session for a rewatch", async () => {
    const series = { id: 1, title: "Órbita", favorite: false, ...emptySeriesMetadata, coverKey: null, genreIds: [], ratingAverage: null };
    const watch = initialWatch(21);
    const item = { id: 11, categoryId: 1, title: "Corto", coverKey: null,
      description: "", opinion: "", releaseYear: null, watch: {
        ...watch, totalEpisodes: 2, completedViews: 1, session: {
          ...watch.session, status: "finished" as const, completed: true, watchedEpisodes: 2,
        },
      },
    };
    const contents = { items: [item], categories: [], ratingAverage: null };
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "catalog_preferences") return defaultPreferences as never;
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "content_list") return contents as never;
      if (command === "watch_update") {
        const rewatch = (args as { input: { action: { kind: string } } }).input.action.kind === "rewatch";
        return { ...contents, items: [{ ...item, watch: { ...item.watch, session: {
          ...watch.session, id: rewatch ? 22 : 21, completed: !rewatch,
        } } }] } as never;
      }
      return { series: [series], genres: [] } as never;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir serie" }));
    fireEvent.change(await screen.findByLabelText("Estado de visualización"), { target: { value: "to_watch" } });
    let modal = within(screen.getByRole("dialog"));
    fireEvent.click(modal.getByRole("button", { name: "Cancelar" }));
    expect(screen.getByText("2 de 2 episodios vistos")).toBeTruthy();
    expect(mockInvoke).not.toHaveBeenCalledWith("watch_update", expect.anything());
    fireEvent.change(screen.getByLabelText("Estado de visualización"), { target: { value: "to_watch" } });
    modal = within(screen.getByRole("dialog"));
    fireEvent.click(modal.getByRole("button", { name: "Reiniciar" }));
    await screen.findByText("0 de 2 episodios vistos");
    fireEvent.click(screen.getByRole("button", { name: "Volver a ver" }));
    expect(mockInvoke).toHaveBeenCalledWith("watch_update", { input: {
      seriesId: 1, contentId: 11, sessionId: 21, action: { kind: "rewatch" },
    } });
    await screen.findByText("1 visualización completa");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Volver a ver" })).toBeNull());
  });

  it("restores the catalog filters and saves the reset", async () => {
    let saved = { ...defaultPreferences, query: "pelicula", favoriteOnly: true, direction: "desc" as const };
    const matching = { id: 1, title: "Órbita", favorite: true, ...emptySeriesMetadata,
      contents: [{ title: "Película 1", status: "finished" }, { title: "Película 2", status: "watching" }],
      genreIds: [], coverKey: null };
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "catalog_preferences") {
        const value = (args as { value: typeof saved | null }).value;
        if (value) saved = value;
        return saved as never;
      }
      return { series: [matching, { ...matching, id: 2, title: "Estación", favorite: false }], genres: [] } as never;
    });
    const first = render(<App />);
    const search = await screen.findByRole("searchbox", { name: "Buscar series o contenidos" });
    expect((search as HTMLInputElement).value).toBe("pelicula");
    expect(screen.getAllByRole("heading", { name: "Órbita" })).toHaveLength(1);
    expect(screen.queryByRole("heading", { name: "Estación" })).toBeNull();
    fireEvent.change(search, { target: { value: "inexistente" } });
    expect(screen.getByRole("heading", { name: "Ninguna serie coincide" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Restablecer filtros" }));
    await waitFor(() => expect(saved).toEqual(defaultPreferences));
    first.unmount();
    render(<App />);
    expect((await screen.findByRole("searchbox") as HTMLInputElement).value).toBe("");
    expect(screen.getByRole("heading", { name: "Estación" })).toBeTruthy();
  });

  it("applies the theme and cover size immediately and restores them on reopening", async () => {
    let saved: Preferences = { ...defaultPreferences, theme: "light", coverSize: "small", query: "Órbita" };
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "catalog_preferences") {
        const next = (args as { value: Preferences | null }).value;
        if (next) saved = next;
        return saved as never;
      }
      if (command === "catalog_taxonomy") return { genres: [], categories: [], catalog: null } as never;
      return { series: [], genres: [] } as never;
    });
    const first = render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
    const theme = await screen.findByRole("combobox", { name: "Tema" });
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(first.container.querySelector(".app")?.getAttribute("data-cover-size")).toBe("small");
    fireEvent.change(theme, { target: { value: "dark" } });
    fireEvent.change(screen.getByRole("combobox", { name: "Tamaño de portada" }), { target: { value: "large" } });
    await waitFor(() => expect(saved).toMatchObject({ theme: "dark", coverSize: "large", query: "Órbita" }));
    first.unmount();
    const reopened = render(<App />);
    await screen.findByRole("searchbox");
    expect(reopened.container.querySelector(".app")?.getAttribute("data-cover-size")).toBe("large");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("confirms unlinking a genre and requires a destination before deleting a used category", async () => {
    const genres = [{ id: 1, name: "Drama", usage: 1 }];
    const categories = [{ id: 1, name: "Corto", usage: 2 }, { id: 2, name: "Película", usage: 0 }];
    let saved: Preferences = { ...defaultPreferences, genreIds: [1],
      contentFilters: { "9": { categoryId: 1, status: "finished" } } };
    const catalog = { series: [], genres: [] };
    mockInvoke.mockImplementation(async (command, args) => {
      if (command === "storage_health") return { schemaVersion: 6 } as never;
      if (command === "catalog_preferences") {
        const next = (args as { value: Preferences | null }).value;
        if (next) saved = next;
        return saved as never;
      }
      if (command === "catalog_taxonomy") {
        const change = (args as { change: { kind: string; group: string } | null }).change;
        if (change?.group === "genre") return { genres: [], categories, catalog } as never;
        if (change?.group === "category") return {
          genres: [], categories: [{ ...categories[1], usage: 2 }], catalog,
        } as never;
        return { genres, categories, catalog: null } as never;
      }
      return catalog as never;
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
    expect(await screen.findByText("1 serie")).toBeTruthy();
    expect(screen.getByText("2 contenidos")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Eliminar Drama" }));
    let modal = within(screen.getByRole("dialog"));
    expect(modal.getByText(/Se desvinculará de 1 serie/)).toBeTruthy();
    fireEvent.click(modal.getByRole("button", { name: "Cancelar" }));
    expect(screen.getByText("Drama")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Eliminar Drama" }));
    modal = within(screen.getByRole("dialog"));
    fireEvent.click(modal.getByRole("button", { name: "Eliminar" }));
    await waitFor(() => expect(screen.queryByText("Drama")).toBeNull());
    fireEvent.click(screen.getByRole("button", { name: "Eliminar Corto" }));
    modal = within(screen.getByRole("dialog"));
    const submit = modal.getByRole("button", { name: "Reasignar y eliminar" }) as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
    fireEvent.change(modal.getByRole("combobox", { name: "Categoría de destino" }), { target: { value: "2" } });
    fireEvent.click(submit);
    expect(mockInvoke).toHaveBeenCalledWith("catalog_taxonomy", {
      change: { kind: "delete", group: "category", id: 1, replacementId: 2 },
    });
    await waitFor(() => expect(saved).toMatchObject({
      genreIds: [], contentFilters: { "9": { categoryId: null, status: "finished" } },
    }));
  });
});
