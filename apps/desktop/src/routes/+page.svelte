<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';
  import type {
    Client,
    CompanyProfile,
    DashboardStats,
    DocumentRecord,
    Note,
    Project,
    SearchHit,
    Section
  } from '$lib/types';

  const emptyCompany: CompanyProfile = {
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
  };

  let section: Section = 'dashboard';
  let stats: DashboardStats = { clients: 0, active_projects: 0, notes: 0, documents: 0 };
  let company: CompanyProfile = { ...emptyCompany };
  let clients: Client[] = [];
  let projects: Project[] = [];
  let notes: Note[] = [];
  let documents: DocumentRecord[] = [];
  let searchQuery = '';
  let searchResults: SearchHit[] = [];
  let busy = false;
  let message = '';

  let newClient = {
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
  };

  let newProject = {
    client_id: null as number | null,
    name: '',
    status: 'Aktivan',
    priority: 'Normalan',
    due_date: '',
    value_cents: 0,
    currency: 'EUR',
    notes: ''
  };

  let newNote = {
    title: '',
    body_markdown: '',
    client_id: null as number | null,
    project_id: null as number | null,
    tags: ''
  };

  const nav: Array<{ id: Section; label: string }> = [
    { id: 'dashboard', label: 'Pregled' },
    { id: 'company', label: 'Moja firma' },
    { id: 'clients', label: 'Klijenti' },
    { id: 'projects', label: 'Projekti' },
    { id: 'notes', label: 'Bilješke' },
    { id: 'documents', label: 'Dokumenti' }
  ];

  async function refresh() {
    [stats, company, clients, projects, notes, documents] = await Promise.all([
      api.dashboard(),
      api.company(),
      api.clients(),
      api.projects(),
      api.notes(),
      api.documents()
    ]);
  }

  async function run(action: () => Promise<unknown>, success: string) {
    busy = true;
    message = '';
    try {
      await action();
      await refresh();
      message = success;
    } catch (error) {
      message = String(error);
    } finally {
      busy = false;
    }
  }

  async function saveCompany() {
    await run(() => api.saveCompany(company), 'Podaci firme su spremljeni.');
  }

  async function addClient() {
    if (!newClient.name.trim()) return;
    await run(() => api.createClient(newClient), 'Klijent je dodan.');
    newClient = {
      ...newClient,
      name: '',
      tax_id: '',
      registration_id: '',
      email: '',
      phone: '',
      website: '',
      address: '',
      city: '',
      notes: ''
    };
  }

  async function addProject() {
    if (!newProject.name.trim()) return;
    await run(() => api.createProject(newProject), 'Projekt je dodan.');
    newProject = { ...newProject, name: '', due_date: '', value_cents: 0, notes: '' };
  }

  async function addNote() {
    if (!newNote.title.trim()) return;
    await run(() => api.createNote(newNote), 'Bilješka je dodana.');
    newNote = { title: '', body_markdown: '', client_id: null, project_id: null, tags: '' };
  }

  async function addDocument() {
    const selected = await open({ multiple: false, directory: false });
    if (!selected || Array.isArray(selected)) return;
    const title = selected.split(/[\\/]/).pop() || 'Dokument';
    await run(() => api.importDocument(selected, title, null, null), 'Dokument je uvezen u Virelo.');
  }

  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  function queueSearch() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(async () => {
      searchResults = searchQuery.trim().length >= 2 ? await api.search(searchQuery.trim()) : [];
    }, 180);
  }

  function openSearchHit(hit: SearchHit) {
    const target: Record<string, Section> = {
      client: 'clients',
      project: 'projects',
      note: 'notes',
      document: 'documents'
    };
    section = target[hit.kind] ?? 'dashboard';
    searchQuery = '';
    searchResults = [];
  }

  onMount(async () => {
    try {
      await refresh();
    } catch (error) {
      message = String(error);
    }
  });
</script>

<svelte:head>
  <title>Virelo</title>
  <meta name="description" content="Virelo local-first poslovni workspace" />
</svelte:head>

<div class="app-shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">V</div>
      <div>
        <strong>Virelo</strong>
        <span>Local Business Workspace</span>
      </div>
    </div>

    <div class="nav-label">Workspace</div>
    {#each nav as item}
      <button class:active={section === item.id} class="nav-button" onclick={() => (section = item.id)}>
        {item.label}
      </button>
    {/each}
  </aside>

  <main class="main">
    <div class="topbar">
      <div class="search-wrap">
        <input
          class="search-input"
          placeholder="Pretraži klijente, projekte, bilješke i dokumente…"
          bind:value={searchQuery}
          oninput={queueSearch}
        />
        {#if searchResults.length}
          <div class="search-results">
            {#each searchResults as hit}
              <button class="search-result" onclick={() => openSearchHit(hit)}>
                <strong>{hit.title}</strong><br />
                <small>{hit.subtitle}</small>
              </button>
            {/each}
          </div>
        {/if}
      </div>
      <span class="muted">Offline · lokalni podaci</span>
    </div>

    {#if message}<div class="notice">{message}</div>{/if}

    {#if section === 'dashboard'}
      <h1 class="page-title">Pregled</h1>
      <p class="page-subtitle">Sve važne poslovne informacije na jednom lokalnom mjestu.</p>
      <div class="stats-grid">
        <div class="card"><div class="stat-value">{stats.clients}</div><div class="muted">Klijenti</div></div>
        <div class="card"><div class="stat-value">{stats.active_projects}</div><div class="muted">Aktivni projekti</div></div>
        <div class="card"><div class="stat-value">{stats.notes}</div><div class="muted">Bilješke</div></div>
        <div class="card"><div class="stat-value">{stats.documents}</div><div class="muted">Dokumenti</div></div>
      </div>
    {:else if section === 'company'}
      <h1 class="page-title">Moja firma</h1>
      <p class="page-subtitle">Osnovni, porezni, bankovni i kontakt podaci firme.</p>
      <div class="card form-grid">
        <label>Naziv<input class="field" bind:value={company.name} /></label>
        <label>OIB / PIB / DIČ<input class="field" bind:value={company.tax_id} /></label>
        <label>MBO / MBS / IČO<input class="field" bind:value={company.registration_id} /></label>
        <label>Država<input class="field" bind:value={company.country} /></label>
        <label>Adresa<input class="field" bind:value={company.address} /></label>
        <label>Grad<input class="field" bind:value={company.city} /></label>
        <label>Poštanski broj<input class="field" bind:value={company.postal_code} /></label>
        <label>E-mail<input class="field" bind:value={company.email} /></label>
        <label>Telefon<input class="field" bind:value={company.phone} /></label>
        <label>Web<input class="field" bind:value={company.website} /></label>
        <label>IBAN<input class="field" bind:value={company.iban} /></label>
        <label>SWIFT / BIC<input class="field" bind:value={company.bic} /></label>
        <label>Banka<input class="field" bind:value={company.bank_name} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={company.notes}></textarea></label>
        <div class="wide"><button class="button primary" disabled={busy} onclick={saveCompany}>Spremi podatke</button></div>
      </div>
    {:else if section === 'clients'}
      <h1 class="page-title">Klijenti</h1>
      <p class="page-subtitle">Kontakti, identifikacijski podaci i interne bilješke klijenata.</p>
      <div class="card form-grid">
        <label>Naziv<input class="field" bind:value={newClient.name} /></label>
        <label>Status<select class="field" bind:value={newClient.status}><option>Aktivan</option><option>Potencijalni</option><option>Neaktivan</option></select></label>
        <label>OIB / PIB / DIČ<input class="field" bind:value={newClient.tax_id} /></label>
        <label>MBO / MBS / IČO<input class="field" bind:value={newClient.registration_id} /></label>
        <label>E-mail<input class="field" bind:value={newClient.email} /></label>
        <label>Telefon<input class="field" bind:value={newClient.phone} /></label>
        <label>Web<input class="field" bind:value={newClient.website} /></label>
        <label>Grad<input class="field" bind:value={newClient.city} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={newClient.notes}></textarea></label>
        <div class="wide"><button class="button primary" disabled={busy} onclick={addClient}>Dodaj klijenta</button></div>
      </div>
      <div class="toolbar"><strong>{clients.length} klijenata</strong></div>
      <div class="list">
        {#each clients as client}
          <div class="list-item">
            <div><h3>{client.name}</h3><p>{client.status} · {client.tax_id || 'bez poreznog ID-a'} · {client.email || 'bez e-maila'}</p></div>
            <button class="button danger" onclick={() => run(() => api.deleteClient(client.id), 'Klijent je uklonjen.')}>Ukloni</button>
          </div>
        {:else}
          <div class="empty">Još nema klijenata.</div>
        {/each}
      </div>
    {:else if section === 'projects'}
      <h1 class="page-title">Projekti</h1>
      <p class="page-subtitle">Projektni statusi, rokovi, vrijednost i veza s klijentima.</p>
      <div class="card form-grid">
        <label>Naziv<input class="field" bind:value={newProject.name} /></label>
        <label>Klijent<select class="field" bind:value={newProject.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Status<select class="field" bind:value={newProject.status}><option>Aktivan</option><option>Na čekanju</option><option>Završen</option></select></label>
        <label>Prioritet<select class="field" bind:value={newProject.priority}><option>Nizak</option><option>Normalan</option><option>Visok</option></select></label>
        <label>Rok<input class="field" type="date" bind:value={newProject.due_date} /></label>
        <label>Vrijednost u centima<input class="field" type="number" bind:value={newProject.value_cents} /></label>
        <label class="wide">Bilješke<textarea class="field" bind:value={newProject.notes}></textarea></label>
        <div class="wide"><button class="button primary" disabled={busy} onclick={addProject}>Dodaj projekt</button></div>
      </div>
      <div class="toolbar"><strong>{projects.length} projekata</strong></div>
      <div class="list">
        {#each projects as project}
          <div class="list-item">
            <div><h3>{project.name}</h3><p>{project.status} · {project.client_name || 'bez klijenta'} · {project.due_date || 'bez roka'}</p></div>
            <button class="button danger" onclick={() => run(() => api.deleteProject(project.id), 'Projekt je uklonjen.')}>Ukloni</button>
          </div>
        {:else}
          <div class="empty">Još nema projekata.</div>
        {/each}
      </div>
    {:else if section === 'notes'}
      <h1 class="page-title">Bilješke</h1>
      <p class="page-subtitle">Markdown bilješke povezane s klijentima i projektima.</p>
      <div class="card form-grid">
        <label class="wide">Naslov<input class="field" bind:value={newNote.title} /></label>
        <label>Klijent<select class="field" bind:value={newNote.client_id}><option value={null}>Bez klijenta</option>{#each clients as client}<option value={client.id}>{client.name}</option>{/each}</select></label>
        <label>Projekt<select class="field" bind:value={newNote.project_id}><option value={null}>Bez projekta</option>{#each projects as project}<option value={project.id}>{project.name}</option>{/each}</select></label>
        <label class="wide">Tagovi<input class="field" bind:value={newNote.tags} placeholder="ugovor, sastanak, hitno" /></label>
        <label class="wide">Sadržaj<textarea class="field" bind:value={newNote.body_markdown}></textarea></label>
        <div class="wide"><button class="button primary" disabled={busy} onclick={addNote}>Dodaj bilješku</button></div>
      </div>
      <div class="toolbar"><strong>{notes.length} bilješki</strong></div>
      <div class="list">
        {#each notes as note}
          <div class="list-item">
            <div><h3>{note.title}</h3><p>{note.tags || 'bez tagova'} · {note.updated_at}</p></div>
            <button class="button danger" onclick={() => run(() => api.deleteNote(note.id), 'Bilješka je uklonjena.')}>Ukloni</button>
          </div>
        {:else}
          <div class="empty">Još nema bilješki.</div>
        {/each}
      </div>
    {:else if section === 'documents'}
      <h1 class="page-title">Dokumenti</h1>
      <p class="page-subtitle">Datoteke se kopiraju u lokalni Virelo application-data direktorij.</p>
      <div class="toolbar">
        <strong>{documents.length} dokumenata</strong>
        <button class="button primary" disabled={busy} onclick={addDocument}>Uvezi dokument</button>
      </div>
      <div class="list">
        {#each documents as document}
          <div class="list-item">
            <div><h3>{document.title}</h3><p>{document.file_name} · {document.created_at}</p></div>
            <button class="button danger" onclick={() => run(() => api.deleteDocument(document.id), 'Dokument je uklonjen.')}>Ukloni</button>
          </div>
        {:else}
          <div class="empty">Još nema dokumenata.</div>
        {/each}
      </div>
    {/if}
  </main>
</div>
