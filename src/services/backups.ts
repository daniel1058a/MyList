import { invoke } from "@tauri-apps/api/core";
import type { Language } from "../locales/locale";
import type { es } from "../locales/es";

export type BackupStatus = { automatic: boolean; lastSuccess: string | null };
export type BackupSummary = {
  createdAt: string;
  schemaVersion: number;
  kind: "manual" | "automatic" | "preventive";
  seriesCount: number;
  contentCount: number;
  imageCount: number;
};

export type BackupAction = { kind: "status" | "check_automatic" | "save" | "preview" | "open_folder" }
  | { kind: "set_automatic"; enabled: boolean }
  | { kind: "restore" | "cancel"; token: string }
  | { kind: "export"; format: "csv" | "markdown" };
export type BackupOutcome = { kind: "status"; status: BackupStatus }
  | { kind: "saved"; name: string }
  | { kind: "preview"; token: string; summary: BackupSummary }
  | { kind: "restored" | "cancelled" | "opened" };

export const accessBackup = (action: BackupAction, language?: Language) =>
  invoke<BackupOutcome>("backup_access", { action, language });

export type BackupError = keyof typeof es.backups.errors;
export function backupError(reason: unknown, errors: Record<BackupError, string>): BackupError {
  if (typeof reason === "object" && reason !== null && "code" in reason && typeof reason.code === "string"
    && Object.hasOwn(errors, reason.code)) return reason.code as BackupError;
  return "io";
}
