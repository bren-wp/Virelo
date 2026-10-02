import { invoke } from '@tauri-apps/api/core';
import type {
  ActivityRecord,
  AppInfo,
  Client,
  CompanyProfile,
  DashboardStats,
  DocumentRecord,
  FinanceRecord,
  Note,
  Project,
  SearchHit,
  TaskRecord
} from './types';

export const api = {
  dashboard: () => invoke<DashboardStats>('dashboard_stats'),
  company: () => invoke<CompanyProfile>('get_company_profile'),
  saveCompany: (profile: CompanyProfile) => invoke<void>('save_company_profile', { profile }),

  clients: () => invoke<Client[]>('list_clients'),
  createClient: (client: Omit<Client, 'id' | 'created_at'>) =>
    invoke<number>('create_client', { client }),
  updateClient: (client: Client) => invoke<void>('update_client', { client }),
  deleteClient: (id: number) => invoke<void>('delete_client', { id }),

  projects: () => invoke<Project[]>('list_projects'),
  createProject: (project: Omit<Project, 'id' | 'client_name' | 'created_at'>) =>
    invoke<number>('create_project', { project }),
  updateProject: (project: Project) => invoke<void>('update_project', { project }),
  deleteProject: (id: number) => invoke<void>('delete_project', { id }),

  notes: () => invoke<Note[]>('list_notes'),
  createNote: (note: Omit<Note, 'id' | 'updated_at'>) =>
    invoke<number>('create_note', { note }),
  updateNote: (note: Note) => invoke<void>('update_note', { note }),
  deleteNote: (id: number) => invoke<void>('delete_note', { id }),

  documents: () => invoke<DocumentRecord[]>('list_documents'),
  importDocument: (
    sourcePath: string,
    title: string,
    clientId: number | null,
    projectId: number | null
  ) => invoke<number>('import_document', { sourcePath, title, clientId, projectId }),
  updateDocument: (id: number, title: string, clientId: number | null, projectId: number | null) =>
    invoke<void>('update_document', { id, title, clientId, projectId }),
  deleteDocument: (id: number) => invoke<void>('delete_document', { id }),

  tasks: () => invoke<TaskRecord[]>('list_tasks'),
  createTask: (task: Omit<TaskRecord, 'id' | 'client_name' | 'project_name' | 'created_at' | 'updated_at'>) =>
    invoke<number>('create_task', { task }),
  updateTask: (task: TaskRecord) => invoke<void>('update_task', { task }),
  deleteTask: (id: number) => invoke<void>('delete_task', { id }),

  activities: () => invoke<ActivityRecord[]>('list_activities'),
  createActivity: (activity: Omit<ActivityRecord, 'id' | 'client_name' | 'project_name' | 'created_at'>) =>
    invoke<number>('create_activity', { activity }),
  deleteActivity: (id: number) => invoke<void>('delete_activity', { id }),

  finance: () => invoke<FinanceRecord[]>('list_finance_records'),
  createFinance: (record: Omit<FinanceRecord, 'id' | 'client_name' | 'created_at' | 'updated_at'>) =>
    invoke<number>('create_finance_record', { record }),
  updateFinance: (record: FinanceRecord) => invoke<void>('update_finance_record', { record }),
  deleteFinance: (id: number) => invoke<void>('delete_finance_record', { id }),

  info: () => invoke<AppInfo>('app_info'),
  exportJson: (destination: string) => invoke<void>('export_workspace_json', { destination }),
  backupDatabase: (destination: string) => invoke<void>('backup_database', { destination }),
  search: (query: string) => invoke<SearchHit[]>('global_search', { query })
};
