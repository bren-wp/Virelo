import { invoke } from '@tauri-apps/api/core';
import type {
  ActivityRecord,
  AppInfo,
  BankAccount,
  Client,
  ClientContact,
  CompanyProfile,
  ContractRecord,
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

  contacts: () => invoke<ClientContact[]>('list_client_contacts'),
  createContact: (contact: Omit<ClientContact, 'id' | 'client_name' | 'created_at' | 'updated_at'>) =>
    invoke<number>('create_client_contact', { contact }),
  updateContact: (contact: ClientContact) => invoke<void>('update_client_contact', { contact }),
  deleteContact: (id: number) => invoke<void>('delete_client_contact', { id }),

  bankAccounts: () => invoke<BankAccount[]>('list_bank_accounts'),
  createBankAccount: (account: Omit<BankAccount, 'id' | 'created_at' | 'updated_at'>) =>
    invoke<number>('create_bank_account', { account }),
  updateBankAccount: (account: BankAccount) => invoke<void>('update_bank_account', { account }),
  deleteBankAccount: (id: number) => invoke<void>('delete_bank_account', { id }),

  contracts: () => invoke<ContractRecord[]>('list_contracts'),
  createContract: (contract: Omit<ContractRecord, 'id' | 'client_name' | 'project_name' | 'document_title' | 'created_at' | 'updated_at'>) =>
    invoke<number>('create_contract', { contract }),
  updateContract: (contract: ContractRecord) => invoke<void>('update_contract', { contract }),
  deleteContract: (id: number) => invoke<void>('delete_contract', { id }),

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
  exportCsv: (destination: string) => invoke<void>('export_workspace_csv', { destination }),
  exportMarkdown: (destination: string) => invoke<void>('export_workspace_markdown', { destination }),
  exportHtml: (destination: string) => invoke<void>('export_workspace_html', { destination }),
  exportYaml: (destination: string) => invoke<void>('export_workspace_yaml', { destination }),
  exportXml: (destination: string) => invoke<void>('export_workspace_xml', { destination }),
  exportArchive: (destination: string) => invoke<void>('export_workspace_archive', { destination }),
  backupDatabase: (destination: string) => invoke<void>('backup_database', { destination }),
  search: (query: string) => invoke<SearchHit[]>('global_search', { query })
};
