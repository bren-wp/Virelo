<script lang="ts">
  import { onMount } from 'svelte';
  import { confirm, open, save } from '@tauri-apps/plugin-dialog';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { api } from '$lib/api';
  import Icon from '$lib/Icon.svelte';
  import BrandMark from '$lib/BrandMark.svelte';
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
    Section,
    TaskRecord
  } from '$lib/types';

  const emptyCompany = (): CompanyProfile => ({
    name: '',
    tax_id: '',
    registration_id: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'Hrvatska',
    email: '',
    phone: '',
    website: '',
    iban: '',
    bic: '',
    bank_name: '',
    notes: ''
  });

  const emptyClient = () => ({
    name: '',
    tax_id: '',
    registration_id: '',
    email: '',
    phone: '',
    website: '',
    address: '',
    city: '',
    country: 'Hrvatska',
    status: 'Aktivan',
    notes: ''
  });

  const emptyContact = () => ({
    client_id: null as number | null,
    name: '',
    role: '',
    email: '',
    phone: '',
    notes: ''
  });

  const emptyBankAccount = () => ({
    label: '',
    iban: '',
    bic: '',
    bank_name: '',
    currency: 'EUR',
    is_default: false,
    notes: ''
  });

  const emptyContract = () => ({
    number: '',
    title: '',
    client_id: null as number | null,
    project_id: null as number | null,
    document_id: null as number | null,
    status: 'Aktivan',
    signed_date: '',
    start_date: '',
    end_date: '',
    currency: 'EUR',
    notes: ''
  });

  const emptyProject = () => ({
    client_id: null as number | null,
    name: '',
    status: 'Aktivan',
    priority: 'Normalan',
    due_date: '',
    currency: 'EUR',
    notes: ''
  });

  const emptyNote = () => ({
    title: '',
    body_markdown: '',
    client_id: null as number | null,
    project_id: null as number | null,
    tags: ''
  });

  const emptyTask = () => ({
    title: '',
    client_id: null as number | null,
    project_id: null as number | null,
    status: 'Otvoren',
    priority: 'Normalan',
    due_date: '',
    notes: ''
  });

  const emptyActivity = () => ({
    kind: 'Bilješka',
    title: '',
    details: '',
    client_id: null as number | null,
    project_id: null as number | null,
    happened_at: ''
  });

  const emptyFinance = () => ({
    kind: 'Račun',
    number: '',
    title: '',
    client_id: null as number | null,
    currency: 'EUR',
    status: 'Nacrt',
    issue_date: '',
    due_date: '',
    notes: ''
  });

  let section: Section = 'dashboard';
  let stats: DashboardStats = { clients: 0, active_projects: 0, notes: 0, documents: 0 };
  let company: CompanyProfile = emptyCompany();
  let clients: Client[] = [];
  let contacts: ClientContact[] = [];
  let bankAccounts: BankAccount[] = [];
  let contracts: ContractRecord[] = [];
  let projects: Project[] = [];
  let notes: Note[] = [];
  let documents: DocumentRecord[] = [];
  let tasks: TaskRecord[] = [];
  let activities: ActivityRecord[] = [];
  let finance: FinanceRecord[] = [];
  let appInfo: AppInfo | null = null;

  let searchQuery = '';
  let searchResults: SearchHit[] = [];
  let busy = false;
  let message = '';
  let messageError = false;
  let sidebarOpen = false;
  let searchInput: HTMLInputElement;

  let clientForm = emptyClient();
  let editingClientId: number | null = null;

  let contactForm = emptyContact();
  let editingContactId: number | null = null;

  let bankAccountForm = emptyBankAccount();
  let editingBankAccountId: number | null = null;

  let contractForm = emptyContract();
  let contractValue = 0;
  let editingContractId: number | null = null;

  let projectForm = emptyProject();
  let projectValue = 0;
  let editingProjectId: number | null = null;

  let noteForm = emptyNote();
  let editingNoteId: number | null = null;

  let taskForm = emptyTask();
  let editingTaskId: number | null = null;

  let activityForm = emptyActivity();

  let financeForm = emptyFinance();
  let financeAmount = 0;
  let editingFinanceId: number | null = null;

  const documentCategories = [
    'Ugovor',
    'Ponuda',
    'Račun',
    'Trošak',
    'Projekt',
    'Identifikacija',
    'Porezno',
    'Banka',
    'Sastanak',
    'Ostalo'
  ];

  let documentClientId: number | null = null;
  let documentProjectId: number | null = null;
  let documentEditId: number | null = null;
  let documentTitle = '';
  let documentCategory = 'Ostalo';
  let documentTags = '';
  let documentDescription = '';
  let documentFilter = '';
  let documentCategoryFilter = 'Sve';
  let documentClientFilter: number | null = null;
  let documentProjectFilter: number | null = null;

  const nav: Array<{ id: Section; label: string; group: 'workspace' | 'tools' }> = [
    { id: 'dashboard', label: 'Pregled', group: 'workspace' },
    { id: 'company', label: 'Moja firma', group: 'workspace' },
    { id: 'clients', label: 'Klijenti', group: 'workspace' },
    { id: 'contacts', label: 'Kontakti', group: 'workspace' },
    { id: 'banking', label: 'Bankovni računi', group: 'workspace' },
    { id: 'contracts', label: 'Ugovori', group: 'workspace' },
    { id: 'projects', label: 'Projekti', group: 'workspace' },
    { id: 'tasks', label: 'Zadaci', group: 'workspace' },
    { id: 'notes', label: 'Bilješke', group: 'workspace' },
    { id: 'documents', label: 'Dokumenti', group: 'workspace' },
    { id: 'activities', label: 'Aktivnosti', group: 'workspace' },
    { id: 'finance', label: 'Financije', group: 'workspace' },
    { id: 'settings', label: 'Sigurnosna kopija', group: 'tools' }
  ];

  function navIcon(sectionId: Section) {
    return ({
      dashboard: 'home',
      company: 'building',
      clients: 'users',
      contacts: 'contact',
      banking: 'bank',
      contracts: 'contract',
      projects: 'briefcase',
      tasks: 'check',
      notes: 'note',
      documents: 'file',
      activities: 'activity',
      finance: 'wallet',
      settings: 'backup'
    } as Record<Section, string>)[sectionId];
  }

  function showMessage(text: string, error = false) {
    message = text;
    messageError = error;
  }

  function navigate(target: Section) {
    section = target;
    sidebarOpen = false;
    searchQuery = '';
    searchResults = [];
  }

  function friendlyError(error: unknown) {
    const raw = String(error ?? '');
    return raw.replace(/^Error:\s*/i, '').trim() || 'Radnju nije moguće izvršiti.';
  }

  function kindLabel(kind: string) {
    return ({
      client: 'Klijent',
      contact: 'Kontakt',
      bank: 'Bankovni račun',
      contract: 'Ugovor',
      project: 'Projekt',
      task: 'Zadatak',
      note: 'Bilješka',
      document: 'Dokument',
      activity: 'Aktivnost',
      finance: 'Financije'
    } as Record<string, string>)[kind] ?? kind;
  }

  function formatDate(value: string) {
    if (!value) return '—';

    const hasTime = value.includes(':');
    let normalized = value;

    if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
      normalized = `${value}T00:00:00`;
    } else if (value.includes(' ') && !value.endsWith('Z')) {
      normalized = `${value.replace(' ', 'T')}Z`;
    }

    const date = new Date(normalized);
    if (Number.isNaN(date.getTime())) return value;

    return new Intl.DateTimeFormat(
      'hr-HR',
      hasTime ? { dateStyle: 'medium', timeStyle: 'short' } : { dateStyle: 'medium' }
    ).format(date);
  }

  async function confirmRemoval(messageText: string) {
    return confirm(messageText, {
      title: 'Virelo',
      kind: 'warning',
      okLabel: 'Ukloni',
      cancelLabel: 'Odustani'
    });
  }

  async function refresh() {
    const result = await Promise.all([
      api.dashboard(),
      api.company(),
      api.clients(),
      api.contacts(),
      api.bankAccounts(),
      api.contracts(),
      api.projects(),
      api.notes(),
      api.documents(),
      api.tasks(),
      api.activities(),
      api.finance(),
      api.info()
    ]);

    [stats, company, clients, contacts, bankAccounts, contracts, projects, notes, documents, tasks, activities, finance, appInfo] = result;
  }

  async function run(action: () => Promise<unknown>, success: string): Promise<boolean> {
    busy = true;
    message = '';
    messageError = false;
    try {
      await action();
      await refresh();
      showMessage(success);
      return true;
    } catch (error) {
      showMessage(friendlyError(error), true);
      return false;
    } finally {
      busy = false;
    }
  }

  async function saveCompany() {
    await run(() => api.saveCompany(company), 'Podaci firme su spremljeni.');
  }

  function resetClientForm() {
    clientForm = emptyClient();
    editingClientId = null;
  }

  function editClient(client: Client) {
    editingClientId = client.id;
    clientForm = {
      name: client.name,
      tax_id: client.tax_id,
      registration_id: client.registration_id,
      email: client.email,
      phone: client.phone,
      website: client.website,
      address: client.address,
      city: client.city,
      country: client.country,
      status: client.status,
      notes: client.notes
    };
  }

  async function saveClient() {
    if (!clientForm.name.trim()) {
      showMessage('Naziv klijenta je obavezan.', true);
      return;
    }

    const ok = editingClientId
      ? await run(
          () =>
            api.updateClient({
              ...clientForm,
              id: editingClientId as number,
              created_at: ''
            }),
          'Klijent je ažuriran.'
        )
      : await run(() => api.createClient(clientForm), 'Klijent je dodan.');

    if (ok) resetClientForm();
  }

  function resetContactForm() {
    contactForm = emptyContact();
    editingContactId = null;
  }

  function editContact(contact: ClientContact) {
    editingContactId = contact.id;
    contactForm = {
      client_id: contact.client_id,
      name: contact.name,
      role: contact.role,
      email: contact.email,
      phone: contact.phone,
      notes: contact.notes
    };
  }

  async function saveContact() {
    if (!contactForm.client_id || !contactForm.name.trim()) {
      showMessage('Klijent i ime kontakta su obavezni.', true);
      return;
    }

    const ok = editingContactId
      ? await run(
          () =>
            api.updateContact({
              ...contactForm,
              id: editingContactId as number,
              client_id: contactForm.client_id as number,
              client_name: null,
              created_at: '',
              updated_at: ''
            }),
          'Kontakt je ažuriran.'
        )
      : await run(
          () =>
            api.createContact({
              ...contactForm,
              client_id: contactForm.client_id as number
            }),
          'Kontakt je dodan.'
        );

    if (ok) resetContactForm();
  }

  function resetBankAccountForm() {
    bankAccountForm = emptyBankAccount();
    editingBankAccountId = null;
  }

  function editBankAccount(account: BankAccount) {
    editingBankAccountId = account.id;
    bankAccountForm = {
      label: account.label,
      iban: account.iban,
      bic: account.bic,
      bank_name: account.bank_name,
      currency: account.currency,
      is_default: account.is_default,
      notes: account.notes
    };
  }

  async function saveBankAccount() {
    if (!bankAccountForm.label.trim() || !bankAccountForm.iban.trim()) {
      showMessage('Naziv računa i IBAN su obavezni.', true);
      return;
    }

    const ok = editingBankAccountId
      ? await run(
          () =>
            api.updateBankAccount({
              ...bankAccountForm,
              id: editingBankAccountId as number,
              created_at: '',
              updated_at: ''
            }),
          'Bankovni račun je ažuriran.'
        )
      : await run(() => api.createBankAccount(bankAccountForm), 'Bankovni račun je dodan.');

    if (ok) resetBankAccountForm();
  }

  function resetContractForm() {
    contractForm = emptyContract();
    contractValue = 0;
    editingContractId = null;
  }

  function editContract(contract: ContractRecord) {
    editingContractId = contract.id;
    contractValue = contract.value_cents / 100;
    contractForm = {
      number: contract.number,
      title: contract.title,
      client_id: contract.client_id,
      project_id: contract.project_id,
      document_id: contract.document_id,
      status: contract.status,
      signed_date: contract.signed_date,
      start_date: contract.start_date,
      end_date: contract.end_date,
      currency: contract.currency,
      notes: contract.notes
    };
  }

  async function saveContract() {
    if (!contractForm.title.trim()) {
      showMessage('Naziv ugovora je obavezan.', true);
      return;
    }

    const value_cents = Math.round((Number(contractValue) || 0) * 100);
    const ok = editingContractId
      ? await run(
          () =>
            api.updateContract({
              ...contractForm,
              id: editingContractId as number,
              client_name: null,
              project_name: null,
              document_title: null,
              value_cents,
              created_at: '',
              updated_at: ''
            }),
          'Ugovor je ažuriran.'
        )
      : await run(
          () => api.createContract({ ...contractForm, value_cents }),
          'Ugovor je dodan.'
        );

    if (ok) resetContractForm();
  }

  function resetProjectForm() {
    projectForm = emptyProject();
    projectValue = 0;
    editingProjectId = null;
  }

  function editProject(project: Project) {
    editingProjectId = project.id;
    projectValue = project.value_cents / 100;
    projectForm = {
      client_id: project.client_id,
      name: project.name,
      status: project.status,
      priority: project.priority,
      due_date: project.due_date,
      currency: project.currency,
      notes: project.notes
    };
  }

  async function saveProject() {
    if (!projectForm.name.trim()) {
      showMessage('Naziv projekta je obavezan.', true);
      return;
    }
    const value_cents = Math.round((Number(projectValue) || 0) * 100);

    const ok = editingProjectId
      ? await run(
          () =>
            api.updateProject({
              ...projectForm,
              id: editingProjectId as number,
              client_name: null,
              value_cents,
              created_at: ''
            }),
          'Projekt je ažuriran.'
        )
      : await run(
          () => api.createProject({ ...projectForm, value_cents }),
          'Projekt je dodan.'
        );

    if (ok) resetProjectForm();
  }

  function resetNoteForm() {
    noteForm = emptyNote();
    editingNoteId = null;
  }

  function editNote(note: Note) {
    editingNoteId = note.id;
    noteForm = {
      title: note.title,
      body_markdown: note.body_markdown,
      client_id: note.client_id,
      project_id: note.project_id,
      tags: note.tags
    };
  }

  async function saveNote() {
    if (!noteForm.title.trim()) {
      showMessage('Naslov bilješke je obavezan.', true);
      return;
    }

    const ok = editingNoteId
      ? await run(
          () =>
            api.updateNote({
              ...noteForm,
              id: editingNoteId as number,
              updated_at: ''
            }),
          'Bilješka je ažurirana.'
        )
      : await run(() => api.createNote(noteForm), 'Bilješka je dodana.');

    if (ok) resetNoteForm();
  }

  function resetTaskForm() {
    taskForm = emptyTask();
    editingTaskId = null;
  }

  function editTask(task: TaskRecord) {
    editingTaskId = task.id;
    taskForm = {
      title: task.title,
      client_id: task.client_id,
      project_id: task.project_id,
      status: task.status,
      priority: task.priority,
      due_date: task.due_date,
      notes: task.notes
    };
  }

  async function saveTask() {
    if (!taskForm.title.trim()) {
      showMessage('Naslov zadatka je obavezan.', true);
      return;
    }

    const ok = editingTaskId
      ? await run(
          () =>
            api.updateTask({
              ...taskForm,
              id: editingTaskId as number,
              client_name: null,
              project_name: null,
              created_at: '',
              updated_at: ''
            }),
          'Zadatak je ažuriran.'
        )
      : await run(() => api.createTask(taskForm), 'Zadatak je dodan.');

    if (ok) resetTaskForm();
  }

  async function saveActivity() {
    if (!activityForm.title.trim()) {
      showMessage('Naslov aktivnosti je obavezan.', true);
      return;
    }

    const ok = await run(() => api.createActivity(activityForm), 'Aktivnost je evidentirana.');
    if (ok) activityForm = emptyActivity();
  }

  function resetFinanceForm() {
    financeForm = emptyFinance();
    financeAmount = 0;
    editingFinanceId = null;
  }

  function editFinance(record: FinanceRecord) {
    editingFinanceId = record.id;
    financeAmount = record.amount_cents / 100;
    financeForm = {
      kind: record.kind,
      number: record.number,
      title: record.title,
      client_id: record.client_id,
      currency: record.currency,
      status: record.status,
      issue_date: record.issue_date,
      due_date: record.due_date,
      notes: record.notes
    };
  }

  async function saveFinance() {
    if (!financeForm.title.trim()) {
      showMessage('Naziv financijskog zapisa je obavezan.', true);
      return;
    }

    const amount_cents = Math.round((Number(financeAmount) || 0) * 100);
    const ok = editingFinanceId
      ? await run(
          () =>
            api.updateFinance({
              ...financeForm,
              id: editingFinanceId as number,
              client_name: null,
              amount_cents,
              created_at: '',
              updated_at: ''
            }),
          'Financijski zapis je ažuriran.'
        )
      : await run(
          () => api.createFinance({ ...financeForm, amount_cents }),
          'Financijski zapis je dodan.'
        );

    if (ok) resetFinanceForm();
  }

  function resetDocumentForm() {
    documentEditId = null;
    documentTitle = '';
    documentClientId = null;
    documentProjectId = null;
    documentCategory = 'Ostalo';
    documentTags = '';
    documentDescription = '';
  }

  function filteredDocuments() {
    const query = documentFilter.trim().toLocaleLowerCase('hr-HR');
    return documents.filter((document) => {
      if (documentCategoryFilter !== 'Sve' && document.category !== documentCategoryFilter) return false;
      if (documentClientFilter !== null && document.client_id !== documentClientFilter) return false;
      if (documentProjectFilter !== null && document.project_id !== documentProjectFilter) return false;
      if (!query) return true;

      return [
        document.title,
        document.file_name,
        document.category,
        document.tags,
        document.description,
        document.client_name || '',
        document.project_name || ''
      ].some((value) => value.toLocaleLowerCase('hr-HR').includes(query));
    });
  }

  function clearDocumentFilters() {
    documentFilter = '';
    documentCategoryFilter = 'Sve';
    documentClientFilter = null;
    documentProjectFilter = null;
  }

  async function addDocument() {
    const selected = await open({ multiple: false, directory: false });
    if (!selected || Array.isArray(selected)) return;
    const title = selected.split(/[\\/]/).pop() || 'Dokument';
    const ok = await run(
      () => api.importDocument(
        selected,
        title,
        documentClientId,
        documentProjectId,
        documentCategory,
        documentTags,
        documentDescription
      ),
      'Dokument je uvezen u Virelo.'
    );
    if (ok) resetDocumentForm();
  }

  function editDocument(document: DocumentRecord) {
    documentEditId = document.id;
    documentTitle = document.title;
    documentClientId = document.client_id;
    documentProjectId = document.project_id;
    documentCategory = document.category || 'Ostalo';
    documentTags = document.tags;
    documentDescription = document.description;
  }

  async function saveDocumentMetadata() {
    if (!documentEditId || !documentTitle.trim()) return;
    const ok = await run(
      () => api.updateDocument(
        documentEditId as number,
        documentTitle,
        documentClientId,
        documentProjectId,
        documentCategory,
        documentTags,
        documentDescription
      ),
      'Podaci dokumenta su ažurirani.'
    );
    if (ok) resetDocumentForm();
  }

  async function openDocument(document: DocumentRecord) {
    try {
      await openPath(document.file_path);
    } catch (error) {
      showMessage(friendlyError(error), true);
    }
  }

  async function exportJson() {
    const destination = await save({
      defaultPath: 'Virelo-podaci.json',
      filters: [{ name: 'Izvoz podataka', extensions: ['json'] }]
    });
    if (!destination) return;
    await run(() => api.exportJson(destination), 'Izvoz podataka je spremljen.');
  }

  async function exportCsv() {
    const destination = await save({
      defaultPath: 'Virelo-podaci.csv',
      filters: [{ name: 'CSV tablica', extensions: ['csv'] }]
    });
    if (!destination) return;
    await run(() => api.exportCsv(destination), 'CSV izvoz je spremljen.');
  }

  async function exportMarkdown() {
    const destination = await save({
      defaultPath: 'Virelo-podaci.md',
      filters: [{ name: 'Markdown', extensions: ['md'] }]
    });
    if (!destination) return;
    await run(() => api.exportMarkdown(destination), 'Markdown izvoz je spremljen.');
  }

  async function exportHtml() {
    const destination = await save({
      defaultPath: 'Virelo-podaci.html',
      filters: [{ name: 'HTML izvještaj', extensions: ['html'] }]
    });
    if (!destination) return;
    await run(() => api.exportHtml(destination), 'HTML izvještaj je spremljen.');
  }

  async function exportYaml() {
    const destination = await save({
      defaultPath: 'Virelo-podaci.yaml',
      filters: [{ name: 'YAML', extensions: ['yaml', 'yml'] }]
    });
    if (!destination) return;
    await run(() => api.exportYaml(destination), 'YAML izvoz je spremljen.');
  }

  async function exportXml() {
    const destination = await save({
      defaultPath: 'Virelo-podaci.xml',
      filters: [{ name: 'XML', extensions: ['xml'] }]
    });
    if (!destination) return;
    await run(() => api.exportXml(destination), 'XML izvoz je spremljen.');
  }

  async function exportArchive() {
    const destination = await save({
      defaultPath: 'Virelo-potpuna-arhiva.zip',
      filters: [{ name: 'Virelo arhiva', extensions: ['zip'] }]
    });
    if (!destination) return;
    await run(
      () => api.exportArchive(destination),
      'Potpuna Virelo arhiva s podacima i dokumentima je spremljena.'
    );
  }

  async function backupDatabase() {
    const destination = await save({
      defaultPath: 'Virelo-sigurnosna-kopija.db',
      filters: [{ name: 'Sigurnosna kopija', extensions: ['db'] }]
    });
    if (!destination) return;
    await run(() => api.backupDatabase(destination), 'Sigurnosna kopija je spremljena.');
  }

  async function removeClient(client: Client) {
    if (!(await confirmRemoval(`Ukloniti klijenta “${client.name}”? Povezani projekti i zapisi neće se automatski izbrisati.`))) return;
    await run(() => api.deleteClient(client.id), 'Klijent je uklonjen.');
  }

  async function removeContact(contact: ClientContact) {
    if (!(await confirmRemoval(`Ukloniti kontakt “${contact.name}”?`))) return;
    await run(() => api.deleteContact(contact.id), 'Kontakt je uklonjen.');
  }

  async function removeBankAccount(account: BankAccount) {
    if (!(await confirmRemoval(`Ukloniti bankovni račun “${account.label}”?`))) return;
    await run(() => api.deleteBankAccount(account.id), 'Bankovni račun je uklonjen.');
  }

  async function removeContract(contract: ContractRecord) {
    if (!(await confirmRemoval(`Ukloniti ugovor “${contract.title}”?`))) return;
    await run(() => api.deleteContract(contract.id), 'Ugovor je uklonjen.');
  }

  async function removeProject(project: Project) {
    if (!(await confirmRemoval(`Ukloniti projekt “${project.name}”?`))) return;
    await run(() => api.deleteProject(project.id), 'Projekt je uklonjen.');
  }

  async function removeTask(task: TaskRecord) {
    if (!(await confirmRemoval(`Ukloniti zadatak “${task.title}”?`))) return;
    await run(() => api.deleteTask(task.id), 'Zadatak je uklonjen.');
  }

  async function removeNote(note: Note) {
    if (!(await confirmRemoval(`Ukloniti bilješku “${note.title}”?`))) return;
    await run(() => api.deleteNote(note.id), 'Bilješka je uklonjena.');
  }

  async function removeDocument(document: DocumentRecord) {
    if (!(await confirmRemoval(`Ukloniti dokument “${document.title}”? Datoteka spremljena kroz Virelo također će biti uklonjena.`))) return;
    await run(() => api.deleteDocument(document.id), 'Dokument je uklonjen.');
  }

  async function removeActivity(activity: ActivityRecord) {
    if (!(await confirmRemoval(`Ukloniti aktivnost “${activity.title}”?`))) return;
    await run(() => api.deleteActivity(activity.id), 'Aktivnost je uklonjena.');
  }

  async function removeFinance(record: FinanceRecord) {
    if (!(await confirmRemoval(`Ukloniti zapis “${record.title}”?`))) return;
    await run(() => api.deleteFinance(record.id), 'Financijski zapis je uklonjen.');
  }

  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  function queueSearch() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(async () => {
      try {
        searchResults = searchQuery.trim().length >= 2 ? await api.search(searchQuery.trim()) : [];
      } catch {
        searchResults = [];
      }
    }, 180);
  }

  function openSearchHit(hit: SearchHit) {
    const target: Record<string, Section> = {
      client: 'clients',
      contact: 'contacts',
      bank: 'banking',
      contract: 'contracts',
      project: 'projects',
      task: 'tasks',
      note: 'notes',
      document: 'documents',
      activity: 'activities',
      finance: 'finance'
    };
    navigate(target[hit.kind] ?? 'dashboard');
  }

  function money(cents: number, currency = 'EUR') {
    try {
      return new Intl.NumberFormat('hr-HR', {
        style: 'currency',
        currency
      }).format(cents / 100);
    } catch {
      return `${(cents / 100).toFixed(2)} ${currency}`;
    }
  }

  function dateDeltaDays(value: string) {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;
    const target = new Date(`${value}T00:00:00`);
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    return Math.ceil((target.getTime() - today.getTime()) / 86_400_000);
  }

  function overdueTasksCount() {
    return tasks.filter((task) => {
      const days = dateDeltaDays(task.due_date);
      return task.status !== 'Završen' && days !== null && days < 0;
    }).length;
  }

  function expiringContractsCount() {
    return contracts.filter((contract) => {
      const days = dateDeltaDays(contract.end_date);
      return !['Završen', 'Otkazan'].includes(contract.status) &&
        days !== null &&
        days >= 0 &&
        days <= 30;
    }).length;
  }

  function overdueFinanceCount() {
    return finance.filter((record) => {
      const days = dateDeltaDays(record.due_date);
      return record.kind === 'Račun' &&
        !['Plaćeno', 'Otkazano'].includes(record.status) &&
        days !== null &&
        days < 0;
    }).length;
  }

  function expiringContracts() {
    return contracts
      .filter((contract) => {
        const days = dateDeltaDays(contract.end_date);
        return !['Završen', 'Otkazan'].includes(contract.status) &&
          days !== null &&
          days >= 0 &&
          days <= 30;
      })
      .sort((a, b) => a.end_date.localeCompare(b.end_date))
      .slice(0, 5);
  }

  function overdueInvoices() {
    return finance
      .filter((record) => {
        const days = dateDeltaDays(record.due_date);
        return record.kind === 'Račun' &&
          !['Plaćeno', 'Otkazano'].includes(record.status) &&
          days !== null &&
          days < 0;
      })
      .sort((a, b) => a.due_date.localeCompare(b.due_date))
      .slice(0, 5);
  }


  onMount(() => {
    void refresh().catch((error) => showMessage(friendlyError(error), true));

    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        searchInput?.focus();
      }

      if (event.key === 'Escape') {
        sidebarOpen = false;
        searchQuery = '';
        searchResults = [];
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });
</script>

<svelte:head>
  <title>Virelo</title>
  <meta name="description" content="Virelo poslovni sustav" />
</svelte:head>

<div class:sidebar-open={sidebarOpen} class="app-shell">
  {#if sidebarOpen}<button class="sidebar-scrim" aria-label="Zatvori navigaciju" onclick={() => (sidebarOpen = false)}></button>{/if}
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark"><BrandMark /></div>
      <div>
        <strong>Virelo</strong>
        <span>Poslovni sustav</span>
      </div>
    </div>

    <div class="nav-label">Poslovanje</div>
    {#each nav.filter((item) => item.group === 'workspace') as item}
      <button class:active={section === item.id} class="nav-button" onclick={() => navigate(item.id)}>
        <Icon name={navIcon(item.id)} size={17} />
        <span>{item.label}</span>
      </button>
    {/each}

    <div class="nav-label">Alati</div>
    {#each nav.filter((item) => item.group === 'tools') as item}
      <button class:active={section === item.id} class="nav-button" onclick={() => navigate(item.id)}>
        <Icon name={navIcon(item.id)} size={17} />
        <span>{item.label}</span>
      </button>
    {/each}

    <div class="sidebar-footer">
      <strong>Virelo {appInfo ? `v${appInfo.version}` : ''}</strong>
      <span>Poslovni sustav</span>
    </div>
  </aside>

  <main class="main">
    <div class="topbar">
      <button class="menu-button" aria-label="Otvori navigaciju" onclick={() => (sidebarOpen = true)}><Icon name="menu" size={19} /></button>
      <div class="search-wrap">
        <span class="search-icon"><Icon name="search" size={17} /></span>
        <input
          class="search-input"
          placeholder="Pretraži Virelo…"
          bind:this={searchInput}
          bind:value={searchQuery}
          oninput={queueSearch}
        />
        {#if searchResults.length}
          <div class="search-results">
            {#each searchResults as hit}
              <button class="search-result" onclick={() => openSearchHit(hit)}>
                <strong>{hit.title}</strong>
                <small>{kindLabel(hit.kind)} · {hit.subtitle}</small>
              </button>
            {/each}
          </div>
        {/if}
      </div>
      <button class="quick-action" onclick={() => navigate('tasks')}><Icon name="plus" size={16} />Novi zadatak</button>
    </div>

    {#if message}
      <div class:notice-error={messageError} class="notice">{message}</div>
    {/if}

    {#if section === 'dashboard'}
      <h1 class="page-title">Pregled</h1>
      <p class="page-subtitle">Najvažnije informacije, rokovi i obaveze na jednom mjestu.</p>

      <div class="stats-grid">
        <button class="card stat-card" onclick={() => navigate('clients')}>
          <div class="stat-head"><Icon name="users" size={18} /><span>Klijenti</span></div><div class="stat-value">{stats.clients}</div>
        </button>
        <button class="card stat-card" onclick={() => navigate('projects')}>
          <div class="stat-head"><Icon name="briefcase" size={18} /><span>Aktivni projekti</span></div><div class="stat-value">{stats.active_projects}</div>
        </button>
        <button class="card stat-card" onclick={() => navigate('tasks')}>
          <div class="stat-head"><Icon name="check" size={18} /><span>Otvoreni zadaci</span></div>
          <div class="stat-value">{tasks.filter((task) => task.status !== 'Završen').length}</div>
        </button>
        <button class:attention={overdueTasksCount() > 0} class="card stat-card" onclick={() => navigate('tasks')}>
          <div class="stat-head"><Icon name="alert" size={18} /><span>Zakašnjeli zadaci</span></div>
          <div class="stat-value">{overdueTasksCount()}</div>
        </button>
        <button class:attention={expiringContractsCount() > 0} class="card stat-card" onclick={() => navigate('contracts')}>
          <div class="stat-head"><Icon name="clock" size={18} /><span>Ugovori do 30 dana</span></div>
          <div class="stat-value">{expiringContractsCount()}</div>
        </button>
        <button class:attention={overdueFinanceCount() > 0} class="card stat-card" onclick={() => navigate('finance')}>
          <div class="stat-head"><Icon name="invoice" size={18} /><span>Dospjeli računi</span></div>
          <div class="stat-value">{overdueFinanceCount()}</div>
        </button>
        <button class="card stat-card" onclick={() => navigate('documents')}>
          <div class="stat-head"><Icon name="file" size={18} /><span>Dokumenti</span></div><div class="stat-value">{stats.documents}</div>
        </button>
      </div>

      <div class="dashboard-grid">
        <section class="card">
          <div class="section-heading">
            <div><strong>Sljedeći zadaci</strong><span>Prioriteti i rokovi</span></div>
            <button class="text-button" onclick={() => navigate('tasks')}>Svi zadaci</button>
          </div>
          <div class="compact-list">
            {#each tasks.filter((task) => task.status !== 'Završen').slice(0, 5) as task}
              <div>
                <strong>{task.title}</strong>
                <span>{task.priority} · {task.due_date || 'bez roka'}</span>
              </div>
            {:else}
              <div class="empty compact">Nema otvorenih zadataka.</div>
            {/each}
          </div>
        </section>

        <section class="card">
          <div class="section-heading">
            <div><strong>Nedavne aktivnosti</strong><span>Zadnji zapisi</span></div>
            <button class="text-button" onclick={() => navigate('activities')}>Sve aktivnosti</button>
          </div>
          <div class="compact-list">
            {#each activities.slice(0, 5) as activity}
              <div>
                <strong>{activity.title}</strong>
                <span>{activity.kind} · {formatDate(activity.happened_at || activity.created_at)}</span>
              </div>
            {:else}
              <div class="empty compact">Još nema aktivnosti.</div>
            {/each}
          </div>
        </section>

        <section class="card">
          <div class="section-heading">
            <div><strong>Ugovori koji istječu</strong><span>Sljedećih 30 dana</span></div>
            <button class="text-button" onclick={() => navigate('contracts')}>Svi ugovori</button>
          </div>
          <div class="compact-list">
            {#each expiringContracts() as contract}
              <div>
                <strong>{contract.title}</strong>
                <span>{contract.client_name || 'bez klijenta'} · {formatDate(contract.end_date)}</span>
              </div>
            {:else}
              <div class="empty compact">Nema ugovora koji uskoro istječu.</div>
            {/each}
          </div>
        </section>

        <section class="card">
          <div class="section-heading">
            <div><strong>Dospjeli računi</strong><span>Za provjeru naplate</span></div>
            <button class="text-button" onclick={() => navigate('finance')}>Financije</button>
          </div>
          <div class="compact-list">
            {#each overdueInvoices() as record}
              <div>
                <strong>{record.title}</strong>
                <span>{money(record.amount_cents, record.currency)} · {formatDate(record.due_date)}</span>
              </div>
            {:else}
              <div class="empty compact">Nema dospjelih računa.</div>
            {/each}
          </div>
        </section>
      </div>
    {:else if section === 'company'}
      <h1 class="page-title">Moja firma</h1>
      <p class="page-subtitle">Identifikacijski, kontaktni i bankovni podaci tvrtke.</p>
      <div class="card form-grid">
        <label>Naziv<input class="field" bind:value={company.name} /></label>
        <label>OIB / PIB / DIČ<input class="field" bind:value={company.tax_id} /></label>
        <label>MBO / MBS / IČO<input class="field" bind:value={company.registration_id} /></label>
        <label>Država<input class="field" bind:value={company.country} /></label>
        <label>Adresa<input class="field" bind:value={company.address} /></label>
        <label>Grad<input class="field" bind:value={company.city} /></label>
        <label>Poštanski broj<input class="field" bind:value={company.postal_code} /></label>
        <label>E-mail<input class="field" type="email" bind:value={company.email} /></label>
        <label>Telefon<input class="field" bind:value={company.phone} /></label>
        <label>Web<input class="field" bind:value={company.website} /></label>
        <label>IBAN<input class="field" bind:value={company.iban} /></label>
        <label>SWIFT / BIC<input class="field" bind:value={company.bic} /></label>
        <label>Banka<input class="field" bind:value={company.bank_name} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={company.notes}></textarea></label>
        <div class="wide form-actions"><button class="button primary" disabled={busy} onclick={saveCompany}><Icon name="save" size={15} />Spremi podatke</button></div>
      </div>
    {:else if section === 'clients'}
      <h1 class="page-title">Klijenti</h1>
      <p class="page-subtitle">Kontakti, porezni identifikatori i interna evidencija klijenata.</p>
      <div class="card form-grid">
        <label>Naziv<input class="field" bind:value={clientForm.name} /></label>
        <label>Status<select class="field" bind:value={clientForm.status}><option>Aktivan</option><option>Potencijalni</option><option>Neaktivan</option></select></label>
        <label>OIB / PIB / DIČ<input class="field" bind:value={clientForm.tax_id} /></label>
        <label>MBO / MBS / IČO<input class="field" bind:value={clientForm.registration_id} /></label>
        <label>E-mail<input class="field" type="email" bind:value={clientForm.email} /></label>
        <label>Telefon<input class="field" bind:value={clientForm.phone} /></label>
        <label>Web<input class="field" bind:value={clientForm.website} /></label>
        <label>Adresa<input class="field" bind:value={clientForm.address} /></label>
        <label>Grad<input class="field" bind:value={clientForm.city} /></label>
        <label>Država<input class="field" bind:value={clientForm.country} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={clientForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveClient}><Icon name={editingClientId ? 'save' : 'plus'} size={15} />{editingClientId ? 'Spremi izmjene' : 'Dodaj klijenta'}</button>
          {#if editingClientId}<button class="button" onclick={resetClientForm}>Odustani</button>{/if}
        </div>
      </div>

      <div class="toolbar"><strong>{clients.length} klijenata</strong></div>
      <div class="list">
        {#each clients as client}
          <div class="list-item">
            <div>
              <h3>{client.name}</h3>
              <p>{client.status} · {client.tax_id || 'bez poreznog ID-a'} · {client.email || 'bez e-maila'}</p>
            </div>
            <div class="row-actions">
              <button class="button" onclick={() => editClient(client)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeClient(client)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema klijenata.</div>
        {/each}
      </div>
    {:else if section === 'contacts'}
      <div class="page-heading-row">
        <div>
          <h1 class="page-title">Kontakti</h1>
          <p class="page-subtitle">Kontakt-osobe klijenata s funkcijom, telefonom, e-mailom i bilješkama.</p>
        </div>
        <div class="heading-badge"><Icon name="contact" size={16} />{contacts.length} kontakata</div>
      </div>

      <div class="card form-grid">
        <label>Klijent<select class="field" bind:value={contactForm.client_id}><option value={null}>Odaberi klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Ime i prezime<input class="field" bind:value={contactForm.name} /></label>
        <label>Funkcija / odjel<input class="field" bind:value={contactForm.role} placeholder="npr. direktor, računovodstvo" /></label>
        <label>E-mail<input class="field" type="email" bind:value={contactForm.email} /></label>
        <label>Telefon<input class="field" bind:value={contactForm.phone} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={contactForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveContact}><Icon name={editingContactId ? 'edit' : 'plus'} size={15} />{editingContactId ? 'Spremi izmjene' : 'Dodaj kontakt'}</button>
          {#if editingContactId}<button class="button" onclick={resetContactForm}>Odustani</button>{/if}
        </div>
      </div>

      <div class="toolbar">
        <strong>{contacts.length} kontakata</strong>
        <span class="muted">Kontakt-osobe ostaju povezane sa svojim klijentom.</span>
      </div>
      <div class="list">
        {#each contacts as contact}
          <div class="list-item contact-item">
            <div class="contact-avatar">{contact.name.slice(0, 1).toUpperCase()}</div>
            <div>
              <h3>{contact.name}</h3>
              <p>{contact.client_name || 'bez klijenta'}{contact.role ? ` · ${contact.role}` : ''}</p>
              <div class="contact-meta">
                {#if contact.email}<span><Icon name="mail" size={13} />{contact.email}</span>{/if}
                {#if contact.phone}<span><Icon name="phone" size={13} />{contact.phone}</span>{/if}
              </div>
            </div>
            <div class="row-actions">
              <button class="button" onclick={() => editContact(contact)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeContact(contact)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema kontakt-osoba. Dodaj prvu osobu povezanu s klijentom.</div>
        {/each}
      </div>
    {:else if section === 'banking'}
      <div class="page-heading-row">
        <div>
          <h1 class="page-title">Bankovni računi</h1>
          <p class="page-subtitle">Vodi više poslovnih računa, valuta i banaka na jednom mjestu.</p>
        </div>
        <span class="heading-badge"><Icon name="bank" size={15} /> {bankAccounts.length} računa</span>
      </div>

      <div class="card form-grid">
        <label>Naziv računa<input class="field" bind:value={bankAccountForm.label} placeholder="Glavni račun" /></label>
        <label>IBAN<input class="field" bind:value={bankAccountForm.iban} /></label>
        <label>SWIFT / BIC<input class="field" bind:value={bankAccountForm.bic} /></label>
        <label>Banka<input class="field" bind:value={bankAccountForm.bank_name} /></label>
        <label>Valuta<select class="field" bind:value={bankAccountForm.currency}><option>EUR</option><option>USD</option><option>GBP</option><option>CHF</option></select></label>
        <label class="toggle-label"><input type="checkbox" bind:checked={bankAccountForm.is_default} /> Zadani račun</label>
        <label class="wide">Bilješke<textarea class="field" bind:value={bankAccountForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveBankAccount}><Icon name="bank" size={15} /> {editingBankAccountId ? 'Spremi izmjene' : 'Dodaj račun'}</button>
          {#if editingBankAccountId}<button class="button" onclick={resetBankAccountForm}>Odustani</button>{/if}
        </div>
      </div>

      <div class="list section-list">
        {#each bankAccounts as account}
          <div class="list-item">
            <div class="item-icon"><Icon name="bank" size={18} /></div>
            <div>
              <h3>{account.label} {#if account.is_default}<span class="chip">Zadani</span>{/if}</h3>
              <p>{account.iban} · {account.bank_name || 'Banka nije navedena'} · {account.currency}</p>
            </div>
            <div class="row-actions">
              <button class="button" onclick={() => editBankAccount(account)}><Icon name="edit" size={14} /> Uredi</button>
              <button class="button danger" onclick={() => removeBankAccount(account)}><Icon name="trash" size={14} /> Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema dodatnih bankovnih računa.</div>
        {/each}
      </div>

    {:else if section === 'contracts'}
      <div class="page-heading-row">
        <div>
          <h1 class="page-title">Ugovori</h1>
          <p class="page-subtitle">Ugovori, rokovi, vrijednosti i povezani dokumenti bez traženja po mapama.</p>
        </div>
        <span class="heading-badge"><Icon name="contract" size={15} /> {contracts.length} ugovora</span>
      </div>

      <div class="card form-grid">
        <label>Broj / oznaka<input class="field" bind:value={contractForm.number} /></label>
        <label>Status<select class="field" bind:value={contractForm.status}><option>Aktivan</option><option>Nacrt</option><option>Na čekanju</option><option>Istječe</option><option>Završen</option><option>Otkazan</option></select></label>
        <label class="wide">Naziv ugovora<input class="field" bind:value={contractForm.title} /></label>
        <label>Klijent<select class="field" bind:value={contractForm.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Projekt<select class="field" bind:value={contractForm.project_id}><option value={null}>Bez projekta</option>{#each projects as project}<option value={project.id}>{project.name}</option>{/each}</select></label>
        <label>Dokument<select class="field" bind:value={contractForm.document_id}><option value={null}>Bez dokumenta</option>{#each documents as document}<option value={document.id}>{document.title}</option>{/each}</select></label>
        <label>Potpisan<input class="field" type="date" bind:value={contractForm.signed_date} /></label>
        <label>Početak<input class="field" type="date" bind:value={contractForm.start_date} /></label>
        <label>Završetak<input class="field" type="date" bind:value={contractForm.end_date} /></label>
        <label>Vrijednost ({contractForm.currency})<input class="field" type="number" min="0" step="0.01" bind:value={contractValue} /></label>
        <label>Valuta<select class="field" bind:value={contractForm.currency}><option>EUR</option><option>USD</option><option>GBP</option><option>CHF</option></select></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={contractForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveContract}><Icon name="contract" size={15} /> {editingContractId ? 'Spremi izmjene' : 'Dodaj ugovor'}</button>
          {#if editingContractId}<button class="button" onclick={resetContractForm}>Odustani</button>{/if}
        </div>
      </div>

      <div class="list section-list">
        {#each contracts as contract}
          <div class="list-item">
            <div class="item-icon"><Icon name="contract" size={18} /></div>
            <div>
              <h3>{contract.title}</h3>
              <p>{contract.number || 'Bez oznake'} · {contract.status} · {contract.client_name || 'bez klijenta'} · {contract.end_date ? `do ${formatDate(contract.end_date)}` : 'bez roka'} · {money(contract.value_cents, contract.currency)}</p>
              {#if contract.document_title}<p class="item-secondary">Dokument: {contract.document_title}</p>{/if}
            </div>
            <div class="row-actions">
              <button class="button" onclick={() => editContract(contract)}><Icon name="edit" size={14} /> Uredi</button>
              <button class="button danger" onclick={() => removeContract(contract)}><Icon name="trash" size={14} /> Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema ugovora.</div>
        {/each}
      </div>

    {:else if section === 'projects'}
      <h1 class="page-title">Projekti</h1>
      <p class="page-subtitle">Status, rok, prioritet, vrijednost i povezanost s klijentom.</p>
      <div class="card form-grid">
        <label>Naziv<input class="field" bind:value={projectForm.name} /></label>
        <label>Klijent<select class="field" bind:value={projectForm.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Status<select class="field" bind:value={projectForm.status}><option>Aktivan</option><option>Na čekanju</option><option>Završen</option></select></label>
        <label>Prioritet<select class="field" bind:value={projectForm.priority}><option>Nizak</option><option>Normalan</option><option>Visok</option><option>Hitan</option></select></label>
        <label>Rok<input class="field" type="date" bind:value={projectForm.due_date} /></label>
        <label>Vrijednost ({projectForm.currency})<input class="field" type="number" min="0" step="0.01" bind:value={projectValue} /></label>
        <label>Valuta<select class="field" bind:value={projectForm.currency}><option>EUR</option><option>USD</option><option>GBP</option><option>CHF</option></select></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={projectForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveProject}><Icon name={editingProjectId ? 'save' : 'plus'} size={15} />{editingProjectId ? 'Spremi izmjene' : 'Dodaj projekt'}</button>
          {#if editingProjectId}<button class="button" onclick={resetProjectForm}>Odustani</button>{/if}
        </div>
      </div>

      <div class="toolbar"><strong>{projects.length} projekata</strong></div>
      <div class="list">
        {#each projects as project}
          <div class="list-item">
            <div>
              <h3>{project.name}</h3>
              <p>{project.status} · {project.client_name || 'bez klijenta'} · {project.due_date || 'bez roka'} · {money(project.value_cents, project.currency)}</p>
            </div>
            <div class="row-actions">
              <span class="chip">{project.priority}</span>
              <button class="button" onclick={() => editProject(project)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeProject(project)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema projekata.</div>
        {/each}
      </div>
    {:else if section === 'tasks'}
      <h1 class="page-title">Zadaci</h1>
      <p class="page-subtitle">Operativne obaveze povezane s klijentima i projektima.</p>
      <div class="card form-grid">
        <label class="wide">Naslov<input class="field" bind:value={taskForm.title} /></label>
        <label>Klijent<select class="field" bind:value={taskForm.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Projekt<select class="field" bind:value={taskForm.project_id}><option value={null}>Bez projekta</option>{#each projects as project}<option value={project.id}>{project.name}</option>{/each}</select></label>
        <label>Status<select class="field" bind:value={taskForm.status}><option>Otvoren</option><option>U tijeku</option><option>Na čekanju</option><option>Završen</option></select></label>
        <label>Prioritet<select class="field" bind:value={taskForm.priority}><option>Nizak</option><option>Normalan</option><option>Visok</option><option>Hitan</option></select></label>
        <label>Rok<input class="field" type="date" bind:value={taskForm.due_date} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={taskForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveTask}><Icon name={editingTaskId ? 'save' : 'plus'} size={15} />{editingTaskId ? 'Spremi izmjene' : 'Dodaj zadatak'}</button>
          {#if editingTaskId}<button class="button" onclick={resetTaskForm}>Odustani</button>{/if}
        </div>
      </div>
      <div class="toolbar"><strong>{tasks.length} zadataka</strong></div>
      <div class="list">
        {#each tasks as task}
          <div class="list-item">
            <div>
              <h3>{task.title}</h3>
              <p>{task.status} · {task.project_name || task.client_name || 'bez veze'} · {task.due_date || 'bez roka'}</p>
            </div>
            <div class="row-actions">
              <span class="chip">{task.priority}</span>
              <button class="button" onclick={() => editTask(task)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeTask(task)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema zadataka.</div>
        {/each}
      </div>
    {:else if section === 'notes'}
      <h1 class="page-title">Bilješke</h1>
      <p class="page-subtitle">Markdown sadržaj, tagovi i povezivanje s poslovnim zapisima.</p>
      <div class="card form-grid">
        <label class="wide">Naslov<input class="field" bind:value={noteForm.title} /></label>
        <label>Klijent<select class="field" bind:value={noteForm.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Projekt<select class="field" bind:value={noteForm.project_id}><option value={null}>Bez projekta</option>{#each projects as project}<option value={project.id}>{project.name}</option>{/each}</select></label>
        <label class="wide">Tagovi<input class="field" bind:value={noteForm.tags} placeholder="ugovor, sastanak, hitno" /></label>
        <label class="wide">Sadržaj<textarea class="field note-editor" bind:value={noteForm.body_markdown}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveNote}><Icon name={editingNoteId ? 'save' : 'plus'} size={15} />{editingNoteId ? 'Spremi izmjene' : 'Dodaj bilješku'}</button>
          {#if editingNoteId}<button class="button" onclick={resetNoteForm}>Odustani</button>{/if}
        </div>
      </div>
      <div class="toolbar"><strong>{notes.length} bilješki</strong></div>
      <div class="list">
        {#each notes as note}
          <div class="list-item">
            <div><h3>{note.title}</h3><p>{note.tags || 'bez tagova'} · {formatDate(note.updated_at)}</p></div>
            <div class="row-actions">
              <button class="button" onclick={() => editNote(note)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeNote(note)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema bilješki.</div>
        {/each}
      </div>
    {:else if section === 'documents'}
      <h1 class="page-title">Dokumenti</h1>
      <p class="page-subtitle">Dokumenti povezani s klijentima i projektima.</p>

      <div class="card form-grid compact-form">
        <label>Klijent<select class="field" bind:value={documentClientId}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Projekt<select class="field" bind:value={documentProjectId}><option value={null}>Bez projekta</option>{#each projects as project}<option value={project.id}>{project.name}</option>{/each}</select></label>
        {#if documentEditId}
          <label class="wide">Naziv dokumenta<input class="field" bind:value={documentTitle} /></label>
          <div class="wide form-actions">
            <button class="button primary" onclick={saveDocumentMetadata}><Icon name="save" size={15} />Spremi podatke dokumenta</button>
            <button class="button" onclick={() => { documentEditId = null; documentTitle = ''; documentClientId = null; documentProjectId = null; }}>Odustani</button>
          </div>
        {:else}
          <div class="wide form-actions"><button class="button primary" disabled={busy} onclick={addDocument}><Icon name="upload" size={15} />Uvezi dokument</button></div>
        {/if}
      </div>

      <div class="toolbar"><strong>{documents.length} dokumenata</strong></div>
      <div class="list">
        {#each documents as document}
          <div class="list-item">
            <div><h3>{document.title}</h3><p>{document.file_name} · {formatDate(document.created_at)}</p></div>
            <div class="row-actions">
              <button class="button primary-soft" onclick={() => openDocument(document)}><Icon name="open" size={15} />Otvori</button>
              <button class="button" onclick={() => editDocument(document)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeDocument(document)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema dokumenata.</div>
        {/each}
      </div>
    {:else if section === 'activities'}
      <h1 class="page-title">Aktivnosti</h1>
      <p class="page-subtitle">Kronologija poziva, sastanaka, e-mailova i drugih poslovnih događaja.</p>
      <div class="card form-grid">
        <label>Vrsta<select class="field" bind:value={activityForm.kind}><option>Bilješka</option><option>Poziv</option><option>Sastanak</option><option>E-mail</option><option>Ugovor</option><option>Drugo</option></select></label>
        <label>Datum i vrijeme<input class="field" type="datetime-local" bind:value={activityForm.happened_at} /></label>
        <label class="wide">Naslov<input class="field" bind:value={activityForm.title} /></label>
        <label>Klijent<select class="field" bind:value={activityForm.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Projekt<select class="field" bind:value={activityForm.project_id}><option value={null}>Bez projekta</option>{#each projects as project}<option value={project.id}>{project.name}</option>{/each}</select></label>
        <label class="wide">Detalji<textarea class="field" bind:value={activityForm.details}></textarea></label>
        <div class="wide form-actions"><button class="button primary" disabled={busy} onclick={saveActivity}><Icon name="plus" size={15} />Dodaj aktivnost</button></div>
      </div>
      <div class="timeline">
        {#each activities as activity}
          <article class="timeline-item">
            <div class="timeline-dot"></div>
            <div class="card">
              <div class="section-heading">
                <div><strong>{activity.title}</strong><span>{activity.kind} · {formatDate(activity.happened_at || activity.created_at)}</span></div>
                <button class="button danger" onclick={() => removeActivity(activity)}><Icon name="trash" size={15} />Ukloni</button>
              </div>
              {#if activity.client_name || activity.project_name}<p class="muted">{activity.client_name || ''}{activity.client_name && activity.project_name ? ' · ' : ''}{activity.project_name || ''}</p>{/if}
              {#if activity.details}<p class="detail-text">{activity.details}</p>{/if}
            </div>
          </article>
        {:else}
          <div class="empty">Još nema aktivnosti.</div>
        {/each}
      </div>
    {:else if section === 'finance'}
      <h1 class="page-title">Financije</h1>
      <p class="page-subtitle">Ponude, računi i troškovi povezani s klijentima i poslovanjem.</p>
      <div class="card form-grid">
        <label>Vrsta<select class="field" bind:value={financeForm.kind}><option>Ponuda</option><option>Račun</option><option>Trošak</option><option>Ostalo</option></select></label>
        <label>Broj / oznaka<input class="field" bind:value={financeForm.number} /></label>
        <label class="wide">Naziv<input class="field" bind:value={financeForm.title} /></label>
        <label>Klijent<select class="field" bind:value={financeForm.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Status<select class="field" bind:value={financeForm.status}><option>Nacrt</option><option>Poslano</option><option>Prihvaćeno</option><option>Plaćeno</option><option>Dospjelo</option><option>Otkazano</option></select></label>
        <label>Iznos ({financeForm.currency})<input class="field" type="number" min="0" step="0.01" bind:value={financeAmount} /></label>
        <label>Valuta<select class="field" bind:value={financeForm.currency}><option>EUR</option><option>USD</option><option>GBP</option><option>CHF</option></select></label>
        <label>Datum izdavanja<input class="field" type="date" bind:value={financeForm.issue_date} /></label>
        <label>Datum dospijeća<input class="field" type="date" bind:value={financeForm.due_date} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={financeForm.notes}></textarea></label>
        <div class="wide form-actions">
          <button class="button primary" disabled={busy} onclick={saveFinance}><Icon name={editingFinanceId ? 'save' : 'plus'} size={15} />{editingFinanceId ? 'Spremi izmjene' : 'Dodaj zapis'}</button>
          {#if editingFinanceId}<button class="button" onclick={resetFinanceForm}>Odustani</button>{/if}
        </div>
      </div>
      <div class="toolbar"><strong>{finance.length} financijskih zapisa</strong></div>
      <div class="list">
        {#each finance as record}
          <div class="list-item">
            <div>
              <h3>{record.title}</h3>
              <p>{record.kind} {record.number ? `· ${record.number}` : ''} · {record.status} · {record.client_name || 'bez klijenta'}</p>
            </div>
            <div class="row-actions">
              <strong class="money">{money(record.amount_cents, record.currency)}</strong>
              <button class="button" onclick={() => editFinance(record)}><Icon name="edit" size={15} />Uredi</button>
              <button class="button danger" onclick={() => removeFinance(record)}><Icon name="trash" size={15} />Ukloni</button>
            </div>
          </div>
        {:else}
          <div class="empty">Još nema financijskih zapisa.</div>
        {/each}
      </div>
    {:else if section === 'settings'}
      <h1 class="page-title">Sigurnosne kopije</h1>
      <p class="page-subtitle">Spremi sigurnosnu kopiju ili izvezi poslovne podatke kada god želiš.</p>

      <div class="settings-grid export-grid">
        <section class="card export-card featured-export">
          <div class="export-icon"><Icon name="archive" size={20} /></div>
          <h2>Potpuna Virelo arhiva</h2>
          <p class="muted">Standardni ZIP s bazom, čitljivim JSON podacima i svim uvezenim dokumentima.</p>
          <button class="button primary" disabled={busy} onclick={exportArchive}><Icon name="archive" size={15} />Spremi ZIP arhivu</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="backup" size={20} /></div>
          <h2>Sigurnosna kopija</h2>
          <p class="muted">Cjelovita kopija baze za povrat podataka.</p>
          <button class="button primary" disabled={busy} onclick={backupDatabase}><Icon name="backup" size={15} />Spremi kopiju</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="export" size={20} /></div>
          <h2>JSON</h2>
          <p class="muted">Strukturirani potpuni izvoz za prijenos i automatizaciju.</p>
          <button class="button" disabled={busy} onclick={exportJson}><Icon name="export" size={15} />Izvezi JSON</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="export" size={20} /></div>
          <h2>CSV</h2>
          <p class="muted">Tablični izvoz prikladan za Excel, Numbers i druge alate.</p>
          <button class="button" disabled={busy} onclick={exportCsv}><Icon name="export" size={15} />Izvezi CSV</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="note" size={20} /></div>
          <h2>Markdown</h2>
          <p class="muted">Čitljiv tekstualni format za arhivu, bilješke i druge editore.</p>
          <button class="button" disabled={busy} onclick={exportMarkdown}><Icon name="export" size={15} />Izvezi Markdown</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="file" size={20} /></div>
          <h2>HTML izvještaj</h2>
          <p class="muted">Samostalni izvještaj koji se otvara u svakom modernom pregledniku.</p>
          <button class="button" disabled={busy} onclick={exportHtml}><Icon name="export" size={15} />Izvezi HTML</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="file" size={20} /></div>
          <h2>YAML</h2>
          <p class="muted">Čitljiv strukturirani format za prijenos, konfiguracije i automatizaciju.</p>
          <button class="button" disabled={busy} onclick={exportYaml}><Icon name="export" size={15} />Izvezi YAML</button>
        </section>
        <section class="card export-card">
          <div class="export-icon"><Icon name="file" size={20} /></div>
          <h2>XML</h2>
          <p class="muted">Standardni razmjenski format za druge poslovne sustave i arhivu.</p>
          <button class="button" disabled={busy} onclick={exportXml}><Icon name="export" size={15} />Izvezi XML</button>
        </section>
      </div>

      <section class="card data-info">
        <h2>O aplikaciji</h2>
        <dl>
          <div><dt>Proizvod</dt><dd>Virelo</dd></div>
          <div><dt>Verzija</dt><dd>{appInfo?.version || '—'}</dd></div>
          <div><dt>Podaci</dt><dd>Sigurnosna kopija i izvoz dostupni su iz ovog izbornika.</dd></div>
        </dl>
      </section>
    {/if}
  </main>
</div>
