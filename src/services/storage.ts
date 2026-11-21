import { invoke } from "@tauri-apps/api/core";

type StorageHealth = Readonly<{ schemaVersion: number }>;

export type StorageErrorCode =
  | "data_directory_unavailable"
  | "database_unavailable"
  | "schema_unreadable"
  | "schema_unsupported"
  | "migration_failed"
  | "diagnostic_failed";

export function checkStorage(): Promise<StorageHealth> {
  return invoke<StorageHealth>("storage_health");
}
