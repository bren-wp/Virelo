import { invoke } from '@tauri-apps/api/core';
import type {
  Client,
  CompanyProfile,
  DashboardStats,
  DocumentRecord,
  Note,
  Project,
  SearchHit
} from './types';

export const api = {
  dashboard: () => invoke<DashboardStats>('dashboard_stats'),
  company: () => invoke<CompanyProfile>('get_company_profile'),
  saveCompany: (profile: CompanyProfile) => invoke<void>('save_company_profile', { profile }),
  clients: () => invoke<Client[]>('list_clients'),
  createClient: (client: Omit<Client, 'id' | 'created_at'>) =>
    invoke<number>('create_client', { client }),
  deleteClient: (id: number) => invoke<void>('delete_client', { id }),
  projects: () => invoke<Project[]>('list_projects'),
  createProject: (project: Omit<Project, 'id' | 'client_name' | 'created_at'>) =>
    invoke<number>('create_project', { project }),
  deleteProject: (id: number) => invoke<void>('delete_project', { id }),
  notes: () => invoke<Note[]>('list_notes'),
  createNote: (note: Omit<Note, 'id' | 'updated_at'>) =>
    invoke<number>('create_note', { note }),
  deleteNote: (id: number) => invoke<void>('delete_note', { id }),
  documents: () => invoke<DocumentRecord[]>('list_documents'),
  importDocument: (
    sourcePath: string,
    title: string,
    clientId: number | null,
    projectId: number | null
  ) => invoke<number>('import_document', { sourcePath, title, clientId, projectId }),
  deleteDocument: (id: number) => invoke<void>('delete_document', { id }),
  search: (query: string) => invoke<SearchHit[]>('global_search', { query })
};
