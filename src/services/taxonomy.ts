import { invoke } from "@tauri-apps/api/core";
import type { Catalog } from "./catalog";

export type TermGroup = "genre" | "category";
export type Term = { id: number; name: string; usage: number };
export type Taxonomy = { genres: Term[]; categories: Term[]; catalog: Catalog | null };
type TermChange =
  | { kind: "save"; group: TermGroup; id: number | null; name: string }
  | { kind: "delete"; group: TermGroup; id: number; replacementId: number | null };

export const accessTaxonomy = (change: TermChange | null) =>
  invoke<Taxonomy>("catalog_taxonomy", { change });
