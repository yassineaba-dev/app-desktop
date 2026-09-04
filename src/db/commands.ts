import { invoke } from "@tauri-apps/api/core";
import type {
  User,
  CreateUserData,
  UpdateUserData,
  Incoming,
  CreateIncomingData,
  UpdateIncomingData,
  IncomingFileInfo,
  Outgoing,
  CreateOutgoingData,
  UpdateOutgoingData,
  OutgoingFileInfo,
  AuditLog,
  PaginatedResult,
  SyncStats,
  LoginData,
  LoginResponse,
} from "./types";

export const authCommands = {
  login: (data: LoginData) =>
    invoke<LoginResponse>("login", { data }),

  resetPasswordByPin: (email: string, pin: string, newPassword: string) =>
    invoke<string>("reset_password_by_pin", { email, pin, newPassword }),
};

export const incomingCommands = {
  getAll: (page?: number, perPage?: number, search?: string, exactDate?: string) =>
    invoke<PaginatedResult<Incoming>>("get_incoming", { page, perPage, search, exactDate }),

  getById: (id: string) =>
    invoke<Incoming>("get_incoming_by_id", { id }),

  create: (data: CreateIncomingData) =>
    invoke<Incoming>("create_incoming", { data }),

  update: (id: string, data: UpdateIncomingData) =>
    invoke<Incoming>("update_incoming", { id, data }),

  delete: (id: string) =>
    invoke<void>("delete_incoming", { id }),

  saveFile: (sourcePath: string) =>
    invoke<IncomingFileInfo>("save_incoming_file", { sourcePath }),

  downloadFile: (id: string) =>
    invoke<string>("download_incoming_file", { id }),
};

export const outgoingCommands = {
  getAll: (page?: number, perPage?: number, search?: string, exactDate?: string) =>
    invoke<PaginatedResult<Outgoing>>("get_outgoing", { page, perPage, search, exactDate }),

  getById: (id: string) =>
    invoke<Outgoing>("get_outgoing_by_id", { id }),

  create: (data: CreateOutgoingData) =>
    invoke<Outgoing>("create_outgoing", { data }),

  update: (id: string, data: UpdateOutgoingData) =>
    invoke<Outgoing>("update_outgoing", { id, data }),

  delete: (id: string) =>
    invoke<void>("delete_outgoing", { id }),

  saveFile: (sourcePath: string) =>
    invoke<OutgoingFileInfo>("save_outgoing_file", { sourcePath }),

  downloadFile: (id: string) =>
    invoke<string>("download_outgoing_file", { id }),

  downloadFileIn: (id: string) =>
    invoke<string>("download_outgoing_file_in", { id }),
};

export const userCommands = {
  getAll: () =>
    invoke<User[]>("get_users"),

  getById: (id: string) =>
    invoke<User>("get_user_by_id", { id }),

  create: (data: CreateUserData) =>
    invoke<User>("create_user", { data }),

  update: (id: string, data: UpdateUserData) =>
    invoke<User>("update_user", { id, data }),

  delete: (id: string) =>
    invoke<void>("delete_user", { id }),

  block: (id: string, blocked: boolean) =>
    invoke<User>("block_user", { id, blocked }),
};

export const auditCommands = {
  getAll: (page?: number, perPage?: number, entity?: string) =>
    invoke<PaginatedResult<AuditLog>>("get_audit_logs", { page, perPage, entity }),
};

export const syncCommands = {
  push: () =>
    invoke<SyncStats>("sync_push"),

  pull: () =>
    invoke<SyncStats>("sync_pull"),

  full: () =>
    invoke<SyncStats>("sync_full"),

  getStatus: () =>
    invoke<string>("get_sync_status"),

  getArabicStatus: () =>
    invoke<string>("get_sync_arabic_status"),
};

export const settingsCommands = {
  saveAndOpenHtml: (filename: string, content: string) =>
    invoke<void>("save_and_open_html", { filename, content }),

  getPinCode: () =>
    invoke<string>("get_pin_code"),

  setPinCode: (pin: string) =>
    invoke<void>("set_pin_code", { pin }),
};

export const databaseCommands = {
  exportToDesktop: () =>
    invoke<string>("export_database_to_desktop"),

  importFromPath: (sourcePath: string) =>
    invoke<string>("import_database_from_pc", { sourcePath }),

  downloadFilteredDb: (request: FilteredExportRequest) =>
    invoke<string>("download_filtered_db", { request }),
};

export const excelCommands = {
  analyze: (path: string) =>
    invoke<ExcelAnalysis>("analyze_excel", { path }),

  import: (request: ExcelImportRequest) =>
    invoke<ExcelImportResult>("import_excel", { request }),
};

export interface ExcelColumn {
  header: string;
  group: string | null;
  field: string | null;
}

export interface ExcelPreviewRow {
  source_row: number;
  cells: string[];
}

export interface RowIssue {
  source_row: number;
  reason: string;
}

export interface ExcelAnalysis {
  file_name: string;
  sheet_name: string;
  kind: string;
  kind_confident: boolean;
  header_rows: number;
  columns: ExcelColumn[];
  rows: ExcelDataRow[];
  preview: ExcelPreviewRow[];
  total_rows: number;
  valid_rows: number;
  invalid_rows: number;
  duplicate_rows: number;
  sample_issues: RowIssue[];
}

export interface ExcelDataRow {
  source_row: number;
  values: Array<string | null>;
  is_duplicate: boolean;
}

export interface ExcelImportRequest {
  file_name: string;
  kind: string;
  columns: ExcelColumn[];
  rows: ExcelDataRow[];
}

export interface RowFailure {
  source_row: number;
  reason: string;
}

export interface ExcelImportResult {
  total: number;
  imported: number;
  skipped: number;
  duplicates: number;
  errors: number;
  failures: RowFailure[];
}

export interface FilteredExportRequest {
  incoming: boolean;
  outgoing: boolean;
  incomingSearch?: string | null;
  incomingDate?: string | null;
  outgoingSearch?: string | null;
  outgoingDate?: string | null;
}
