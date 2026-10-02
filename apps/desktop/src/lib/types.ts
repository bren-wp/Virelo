export type Section =
  | 'dashboard'
  | 'company'
  | 'clients'
  | 'projects'
  | 'tasks'
  | 'notes'
  | 'documents'
  | 'activities'
  | 'finance'
  | 'settings';

export interface DashboardStats {
  clients: number;
  active_projects: number;
  notes: number;
  documents: number;
}

export interface CompanyProfile {
  name: string;
  tax_id: string;
  registration_id: string;
  address: string;
  city: string;
  postal_code: string;
  country: string;
  email: string;
  phone: string;
  website: string;
  iban: string;
  bic: string;
  bank_name: string;
  notes: string;
}

export interface Client {
  id: number;
  name: string;
  tax_id: string;
  registration_id: string;
  email: string;
  phone: string;
  website: string;
  address: string;
  city: string;
  country: string;
  status: string;
  notes: string;
  created_at: string;
}

export interface Project {
  id: number;
  client_id: number | null;
  client_name: string | null;
  name: string;
  status: string;
  priority: string;
  due_date: string;
  value_cents: number;
  currency: string;
  notes: string;
  created_at: string;
}

export interface Note {
  id: number;
  title: string;
  body_markdown: string;
  client_id: number | null;
  project_id: number | null;
  tags: string;
  updated_at: string;
}

export interface DocumentRecord {
  id: number;
  title: string;
  file_name: string;
  file_path: string;
  client_id: number | null;
  project_id: number | null;
  created_at: string;
}

export interface TaskRecord {
  id: number;
  title: string;
  client_id: number | null;
  client_name: string | null;
  project_id: number | null;
  project_name: string | null;
  status: string;
  priority: string;
  due_date: string;
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface ActivityRecord {
  id: number;
  kind: string;
  title: string;
  details: string;
  client_id: number | null;
  client_name: string | null;
  project_id: number | null;
  project_name: string | null;
  happened_at: string;
  created_at: string;
}

export interface FinanceRecord {
  id: number;
  kind: string;
  number: string;
  title: string;
  client_id: number | null;
  client_name: string | null;
  amount_cents: number;
  currency: string;
  status: string;
  issue_date: string;
  due_date: string;
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface SearchHit {
  kind: string;
  id: number;
  title: string;
  subtitle: string;
}

export interface AppInfo {
  version: string;
  data_dir: string;
  database_path: string;
}
