export interface User {
  id: string;
  full_name: string;
  email: string;
  role: string;
  blocked: boolean;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  sync_version: number;
}

export interface CreateUserData {
  full_name: string;
  email: string;
  password: string;
  role?: string;
}

export interface UpdateUserData {
  full_name?: string;
  email?: string;
  role?: string;
}

export interface Incoming {
  id: string;
  registration_number: string;
  correspondence_number: string | null;
  date: string;
  arrival_date: string | null;
  subject: string;
  sender: string;
  destination_service: string;
  source: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  created_by: string | null;
  sync_version: number;
  file_name: string | null;
  file_path: string | null;
  is_duplicate: boolean;
}

export interface IncomingFileInfo {
  file_name: string;
  file_path: string;
}

export interface CreateIncomingData {
  registration_number: string;
  correspondence_number?: string;
  date: string;
  arrival_date?: string;
  subject: string;
  sender: string;
  destination_service: string;
  source?: string;
  notes?: string;
  file_name?: string;
  file_path?: string;
  is_duplicate?: boolean;
}

export interface UpdateIncomingData {
  registration_number?: string;
  correspondence_number?: string;
  date?: string;
  arrival_date?: string;
  subject?: string;
  sender?: string;
  destination_service?: string;
  source?: string;
  notes?: string;
  file_name?: string;
  file_path?: string;
  is_duplicate?: boolean;
}

export interface Outgoing {
  id: string;
  registration_number: string;
  correspondence_number: string | null;
  date: string;
  subject: string;
  recipient: string;
  destination_service: string;
  source: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  created_by: string | null;
  sync_version: number;
  file_name: string | null;
  file_path: string | null;
  file_name_in: string | null;
  file_path_in: string | null;
}

export interface OutgoingFileInfo {
  file_name: string;
  file_path: string;
}

export interface CreateOutgoingData {
  registration_number: string;
  correspondence_number?: string;
  date: string;
  subject: string;
  recipient: string;
  destination_service: string;
  source?: string;
  notes?: string;
  file_name?: string;
  file_path?: string;
  file_name_in?: string;
  file_path_in?: string;
}

export interface UpdateOutgoingData {
  registration_number?: string;
  correspondence_number?: string;
  date?: string;
  subject?: string;
  recipient?: string;
  destination_service?: string;
  source?: string;
  notes?: string;
  file_name?: string;
  file_path?: string;
  file_name_in?: string;
  file_path_in?: string;
}

export interface AuditLog {
  id: string;
  user_id: string | null;
  action: string;
  entity: string;
  entity_id: string;
  timestamp: string;
  metadata: string | null;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

export interface SyncStats {
  pushed: number;
  pulled: number;
  last_sync_at: string;
  pending_push: number;
  status: string;
}

export interface LoginData {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: User;
  token: string;
}

