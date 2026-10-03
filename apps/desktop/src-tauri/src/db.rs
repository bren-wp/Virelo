use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::models::{
    Client, CompanyProfile, DashboardStats, DocumentInput, DocumentRecord, Note, Project,
};

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub database_path: PathBuf,
    pub documents_dir: PathBuf,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        let documents_dir = data_dir.join("documents");
        fs::create_dir_all(&documents_dir).map_err(|error| error.to_string())?;

        let database_path = data_dir.join("virelo.sqlite3");
        let conn = Connection::open(&database_path).map_err(|error| error.to_string())?;
        configure_database(&conn)?;
        migrate(&conn)?;
        crate::extras::migrate(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            database_path,
            documents_dir,
        })
    }
}

pub(crate) fn configure_database(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS company_profile (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            name TEXT NOT NULL DEFAULT '',
            tax_id TEXT NOT NULL DEFAULT '',
            registration_id TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',
            city TEXT NOT NULL DEFAULT '',
            postal_code TEXT NOT NULL DEFAULT '',
            country TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            website TEXT NOT NULL DEFAULT '',
            iban TEXT NOT NULL DEFAULT '',
            bic TEXT NOT NULL DEFAULT '',
            bank_name TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            tax_id TEXT NOT NULL DEFAULT '',
            registration_id TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            website TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',
            city TEXT NOT NULL DEFAULT '',
            country TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'Aktivan',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Aktivan',
            priority TEXT NOT NULL DEFAULT 'Normalan',
            due_date TEXT NOT NULL DEFAULT '',
            value_cents INTEGER NOT NULL DEFAULT 0,
            currency TEXT NOT NULL DEFAULT 'EUR',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            body_markdown TEXT NOT NULL DEFAULT '',
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            tags TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            category TEXT NOT NULL DEFAULT 'Ostalo',
            tags TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_clients_name ON clients(name);
        CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
        CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
        CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title);
        "#,
    )
    .map_err(|error| error.to_string())
}

fn lock(state: &AppState) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
    state
        .conn
        .lock()
        .map_err(|_| "Baza podataka nije dostupna.".to_string())
}

pub fn dashboard_stats(state: &AppState) -> Result<DashboardStats, String> {
    let conn = lock(state)?;
    let count = |table: &str, filter: Option<&str>| -> Result<i64, String> {
        let sql = match filter {
            Some(filter) => format!("SELECT COUNT(*) FROM {table} WHERE {filter}"),
            None => format!("SELECT COUNT(*) FROM {table}"),
        };
        conn.query_row(&sql, [], |row| row.get(0))
            .map_err(|error| error.to_string())
    };

    Ok(DashboardStats {
        clients: count("clients", None)?,
        active_projects: count("projects", Some("status != 'Završen'"))?,
        notes: count("notes", None)?,
        documents: count("documents", None)?,
    })
}

pub fn get_company_profile(state: &AppState) -> Result<CompanyProfile, String> {
    let conn = lock(state)?;
    conn.query_row(
        "SELECT name, tax_id, registration_id, address, city, postal_code, country,
                email, phone, website, iban, bic, bank_name, notes
         FROM company_profile WHERE id = 1",
        [],
        |row| {
            Ok(CompanyProfile {
                name: row.get(0)?,
                tax_id: row.get(1)?,
                registration_id: row.get(2)?,
                address: row.get(3)?,
                city: row.get(4)?,
                postal_code: row.get(5)?,
                country: row.get(6)?,
                email: row.get(7)?,
                phone: row.get(8)?,
                website: row.get(9)?,
                iban: row.get(10)?,
                bic: row.get(11)?,
                bank_name: row.get(12)?,
                notes: row.get(13)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
    .map(|profile| profile.unwrap_or_default())
}

pub fn save_company_profile(state: &AppState, profile: CompanyProfile) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute(
        r#"INSERT INTO company_profile
           (id, name, tax_id, registration_id, address, city, postal_code, country, email,
            phone, website, iban, bic, bank_name, notes, updated_at)
           VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)
           ON CONFLICT(id) DO UPDATE SET
             name=excluded.name, tax_id=excluded.tax_id, registration_id=excluded.registration_id,
             address=excluded.address, city=excluded.city, postal_code=excluded.postal_code,
             country=excluded.country, email=excluded.email, phone=excluded.phone,
             website=excluded.website, iban=excluded.iban, bic=excluded.bic,
             bank_name=excluded.bank_name, notes=excluded.notes, updated_at=CURRENT_TIMESTAMP"#,
        params![
            profile.name,
            profile.tax_id,
            profile.registration_id,
            profile.address,
            profile.city,
            profile.postal_code,
            profile.country,
            profile.email,
            profile.phone,
            profile.website,
            profile.iban,
            profile.bic,
            profile.bank_name,
            profile.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_clients(state: &AppState) -> Result<Vec<Client>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, tax_id, registration_id, email, phone, website, address, city,
                country, status, notes, created_at FROM clients ORDER BY name COLLATE NOCASE",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                tax_id: row.get(2)?,
                registration_id: row.get(3)?,
                email: row.get(4)?,
                phone: row.get(5)?,
                website: row.get(6)?,
                address: row.get(7)?,
                city: row.get(8)?,
                country: row.get(9)?,
                status: row.get(10)?,
                notes: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_client(state: &AppState, client: Client) -> Result<i64, String> {
    if client.name.trim().is_empty() {
        return Err("Naziv klijenta je obavezan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        "INSERT INTO clients
         (name, tax_id, registration_id, email, phone, website, address, city, country, status, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            client.name.trim(),
            client.tax_id.trim(),
            client.registration_id.trim(),
            client.email.trim(),
            client.phone.trim(),
            client.website.trim(),
            client.address.trim(),
            client.city.trim(),
            client.country.trim(),
            client.status.trim(),
            client.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_client(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM clients WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_projects(state: &AppState) -> Result<Vec<Project>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.client_id, c.name, p.name, p.status, p.priority, p.due_date,
                p.value_cents, p.currency, p.notes, p.created_at
         FROM projects p LEFT JOIN clients c ON c.id = p.client_id
         ORDER BY CASE WHEN p.status = 'Završen' THEN 1 ELSE 0 END, p.due_date, p.name",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                client_id: row.get(1)?,
                client_name: row.get(2)?,
                name: row.get(3)?,
                status: row.get(4)?,
                priority: row.get(5)?,
                due_date: row.get(6)?,
                value_cents: row.get(7)?,
                currency: row.get(8)?,
                notes: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_project(state: &AppState, project: Project) -> Result<i64, String> {
    if project.name.trim().is_empty() {
        return Err("Naziv projekta je obavezan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        "INSERT INTO projects (client_id, name, status, priority, due_date, value_cents, currency, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            project.client_id,
            project.name.trim(),
            project.status.trim(),
            project.priority.trim(),
            project.due_date.trim(),
            project.value_cents,
            project.currency.trim(),
            project.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_project(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_notes(state: &AppState) -> Result<Vec<Note>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, body_markdown, client_id, project_id, tags, updated_at
         FROM notes ORDER BY updated_at DESC, id DESC",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                body_markdown: row.get(2)?,
                client_id: row.get(3)?,
                project_id: row.get(4)?,
                tags: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_note(state: &AppState, note: Note) -> Result<i64, String> {
    if note.title.trim().is_empty() {
        return Err("Naslov bilješke je obavezan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        "INSERT INTO notes (title, body_markdown, client_id, project_id, tags)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            note.title.trim(),
            note.body_markdown,
            note.client_id,
            note.project_id,
            note.tags.trim()
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_note(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM notes WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_documents(state: &AppState) -> Result<Vec<DocumentRecord>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.title, d.file_name, d.file_path, d.client_id, c.name,
                    d.project_id, p.name, d.category, d.tags, d.description, d.created_at
             FROM documents d
             LEFT JOIN clients c ON c.id = d.client_id
             LEFT JOIN projects p ON p.id = d.project_id
             ORDER BY d.created_at DESC, d.id DESC",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(DocumentRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                client_id: row.get(4)?,
                client_name: row.get(5)?,
                project_id: row.get(6)?,
                project_name: row.get(7)?,
                category: row.get(8)?,
                tags: row.get(9)?,
                description: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn import_document(
    state: &AppState,
    source_path: String,
    document: DocumentInput,
) -> Result<i64, String> {
    let source = Path::new(&source_path);
    if !source.is_file() {
        return Err("Odabrana datoteka ne postoji.".into());
    }

    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Naziv datoteke nije valjan.".to_string())?;

    let stored_name = format!("{}_{}", Uuid::new_v4(), file_name);
    let destination = state.documents_dir.join(stored_name);
    fs::copy(source, &destination).map_err(|error| error.to_string())?;

    let conn = lock(state)?;
    let destination_text = destination.to_string_lossy().to_string();
    let result = conn.execute(
        "INSERT INTO documents
         (title, file_name, file_path, client_id, project_id, category, tags, description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            if document.title.trim().is_empty() {
                file_name
            } else {
                document.title.trim()
            },
            file_name,
            destination_text,
            document.client_id,
            document.project_id,
            if document.category.trim().is_empty() {
                "Ostalo"
            } else {
                document.category.trim()
            },
            document.tags.trim(),
            document.description.trim()
        ],
    );

    if let Err(error) = result {
        let _ = fs::remove_file(&destination);
        return Err(error.to_string());
    }

    Ok(conn.last_insert_rowid())
}

pub fn delete_document(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    let file_path: Option<String> = conn
        .query_row(
            "SELECT file_path FROM documents WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    conn.execute("DELETE FROM documents WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    drop(conn);

    if let Some(file_path) = file_path {
        let path = PathBuf::from(file_path);
        if path.starts_with(&state.documents_dir) {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        extras,
        models::{
            ActivityRecord, BankAccount, ClientContact, ContractRecord, FinanceRecord, Note,
            Project, TaskRecord,
        },
    };

    fn test_state() -> AppState {
        let data_dir = std::env::temp_dir().join(format!("virelo-test-{}", Uuid::new_v4()));
        let documents_dir = data_dir.join("documents");
        fs::create_dir_all(&documents_dir).expect("test documents directory");

        let database_path = data_dir.join("virelo-test.sqlite3");
        let conn = Connection::open(&database_path).expect("test database");
        configure_database(&conn).expect("database pragmas");
        migrate(&conn).expect("core migrations");
        extras::migrate(&conn).expect("extended migrations");

        AppState {
            conn: Mutex::new(conn),
            database_path,
            documents_dir,
        }
    }

    #[test]
    fn production_crud_flow_works() {
        let state = test_state();

        let client_id = create_client(
            &state,
            Client {
                id: 0,
                name: "Test klijent".into(),
                tax_id: "12345678901".into(),
                registration_id: String::new(),
                email: "test@example.com".into(),
                phone: String::new(),
                website: String::new(),
                address: String::new(),
                city: "Rijeka".into(),
                country: "Hrvatska".into(),
                status: "Aktivan".into(),
                notes: "Važna napomena".into(),
                created_at: String::new(),
            },
        )
        .expect("create client");

        extras::create_client_contact(
            &state,
            ClientContact {
                id: 0,
                client_id,
                client_name: None,
                name: "Ana Test".into(),
                role: "Računovodstvo".into(),
                email: "ana@example.com".into(),
                phone: "+385 51 123 456".into(),
                notes: "Primarni financijski kontakt".into(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .expect("create client contact");

        extras::create_bank_account(
            &state,
            BankAccount {
                id: 0,
                label: "Glavni račun".into(),
                iban: "HR1223600001101234565".into(),
                bic: "ZABAHR2X".into(),
                bank_name: "Test banka".into(),
                currency: "EUR".into(),
                is_default: true,
                notes: "Primarni račun".into(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .expect("create bank account");

        let project_id = create_project(
            &state,
            Project {
                id: 0,
                client_id: Some(client_id),
                client_name: None,
                name: "Virelo QA projekt".into(),
                status: "Aktivan".into(),
                priority: "Visok".into(),
                due_date: "2026-12-31".into(),
                value_cents: 125_000,
                currency: "EUR".into(),
                notes: "Test projekta".into(),
                created_at: String::new(),
            },
        )
        .expect("create project");

        create_note(
            &state,
            Note {
                id: 0,
                title: "Sastanak".into(),
                body_markdown: "Dogovoreni sljedeći koraci.".into(),
                client_id: Some(client_id),
                project_id: Some(project_id),
                tags: "sastanak".into(),
                updated_at: String::new(),
            },
        )
        .expect("create note");

        extras::create_task(
            &state,
            TaskRecord {
                id: 0,
                title: "Pripremi ponudu".into(),
                client_id: Some(client_id),
                client_name: None,
                project_id: Some(project_id),
                project_name: None,
                status: "Otvoren".into(),
                priority: "Visok".into(),
                due_date: "2026-12-15".into(),
                notes: "Test zadatka".into(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .expect("create task");

        extras::create_activity(
            &state,
            ActivityRecord {
                id: 0,
                kind: "Poziv".into(),
                title: "Poziv klijentu".into(),
                details: "Potvrđeni detalji projekta.".into(),
                client_id: Some(client_id),
                client_name: None,
                project_id: Some(project_id),
                project_name: None,
                happened_at: "2026-10-02T12:00".into(),
                created_at: String::new(),
            },
        )
        .expect("create activity");

        extras::create_finance_record(
            &state,
            FinanceRecord {
                id: 0,
                kind: "Ponuda".into(),
                number: "P-001".into(),
                title: "Ponuda za projekt".into(),
                client_id: Some(client_id),
                client_name: None,
                amount_cents: 125_000,
                currency: "EUR".into(),
                status: "Nacrt".into(),
                issue_date: "2026-10-02".into(),
                due_date: "2026-10-15".into(),
                notes: String::new(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .expect("create finance record");

        let source = state.documents_dir.join("virelo-test-document.txt");
        fs::write(&source, b"Virelo QA").expect("write source document");
        let document_id = import_document(
            &state,
            source.to_string_lossy().into_owned(),
            DocumentInput {
                title: "QA dokument".into(),
                client_id: Some(client_id),
                project_id: Some(project_id),
                category: "Ugovor".into(),
                tags: "qa, potpisano".into(),
                description: "Dokument za provjeru arhivskog modula.".into(),
            },
        )
        .expect("import document");

        extras::create_contract(
            &state,
            ContractRecord {
                id: 0,
                number: "UG-001".into(),
                title: "Virelo QA ugovor".into(),
                client_id: Some(client_id),
                client_name: None,
                project_id: Some(project_id),
                project_name: None,
                document_id: Some(document_id),
                document_title: None,
                status: "Aktivan".into(),
                signed_date: "2026-10-02".into(),
                start_date: "2026-10-02".into(),
                end_date: "2027-10-02".into(),
                value_cents: 125_000,
                currency: "EUR".into(),
                notes: "Test ugovora".into(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .expect("create contract");

        assert_eq!(list_clients(&state).expect("list clients").len(), 1);
        assert_eq!(
            extras::list_client_contacts(&state)
                .expect("list client contacts")
                .len(),
            1
        );
        assert_eq!(
            extras::list_bank_accounts(&state)
                .expect("list bank accounts")
                .len(),
            1
        );
        assert_eq!(
            extras::list_contracts(&state)
                .expect("list contracts")
                .len(),
            1
        );
        assert_eq!(list_projects(&state).expect("list projects").len(), 1);
        assert_eq!(list_notes(&state).expect("list notes").len(), 1);
        let listed_documents = list_documents(&state).expect("list documents");
        assert_eq!(listed_documents.len(), 1);
        assert_eq!(listed_documents[0].category, "Ugovor");
        assert_eq!(
            listed_documents[0].client_name.as_deref(),
            Some("Test klijent")
        );
        assert_eq!(
            listed_documents[0].project_name.as_deref(),
            Some("Virelo QA projekt")
        );
        assert!(listed_documents[0].tags.contains("potpisano"));
        assert_eq!(extras::list_tasks(&state).expect("list tasks").len(), 1);
        assert_eq!(
            extras::list_activities(&state)
                .expect("list activities")
                .len(),
            1
        );
        assert_eq!(
            extras::list_finance_records(&state)
                .expect("list finance")
                .len(),
            1
        );

        let search = extras::global_search(&state, "Virelo QA".into()).expect("global search");
        assert!(search.iter().any(|hit| hit.kind == "project"));

        let contact_search =
            extras::global_search(&state, "Ana Test".into()).expect("contact search");
        assert!(contact_search.iter().any(|hit| hit.kind == "contact"));

        let bank_search =
            extras::global_search(&state, "Glavni račun".into()).expect("bank account search");
        assert!(bank_search.iter().any(|hit| hit.kind == "bank"));

        let contract_search =
            extras::global_search(&state, "QA ugovor".into()).expect("contract search");
        assert!(contract_search.iter().any(|hit| hit.kind == "contract"));

        let document_search =
            extras::global_search(&state, "potpisano".into()).expect("document metadata search");
        assert!(document_search.iter().any(|hit| hit.kind == "document"));

        let export_root = state
            .documents_dir
            .parent()
            .expect("test export directory")
            .to_path_buf();

        let json_path = export_root.join("export.json");
        let csv_path = export_root.join("export.csv");
        let md_path = export_root.join("export.md");
        let html_path = export_root.join("export.html");
        let yaml_path = export_root.join("export.yaml");
        let xml_path = export_root.join("export.xml");
        let archive_path = export_root.join("Virelo-arhiva.zip");

        extras::export_workspace_json(&state, json_path.to_string_lossy().into_owned())
            .expect("json export");
        extras::export_workspace_csv(&state, csv_path.to_string_lossy().into_owned())
            .expect("csv export");
        extras::export_workspace_markdown(&state, md_path.to_string_lossy().into_owned())
            .expect("markdown export");
        extras::export_workspace_html(&state, html_path.to_string_lossy().into_owned())
            .expect("html export");
        extras::export_workspace_yaml(&state, yaml_path.to_string_lossy().into_owned())
            .expect("yaml export");
        extras::export_workspace_xml(&state, xml_path.to_string_lossy().into_owned())
            .expect("xml export");
        extras::export_workspace_archive(&state, archive_path.to_string_lossy().into_owned())
            .expect("archive export");

        assert!(fs::read_to_string(json_path)
            .expect("read json")
            .contains("Ana Test"));
        assert!(fs::read_to_string(csv_path)
            .expect("read csv")
            .contains("Ana Test"));
        assert!(fs::read_to_string(md_path)
            .expect("read markdown")
            .contains("Ana Test"));
        let html_export = fs::read_to_string(html_path).expect("read html");
        assert!(html_export.contains("Virelo QA ugovor"));
        assert!(html_export.contains("potpisano"));
        assert!(fs::read_to_string(yaml_path)
            .expect("read yaml")
            .contains("Glavni račun"));
        assert!(fs::read_to_string(xml_path)
            .expect("read xml")
            .contains("Virelo QA ugovor"));

        let archive_file = fs::File::open(&archive_path).expect("open archive");
        let mut archive = zip::ZipArchive::new(archive_file).expect("read archive");
        let archive_names: Vec<String> = archive.file_names().map(str::to_string).collect();
        assert!(archive_names.iter().any(|name| name == "virelo-data.json"));
        assert!(archive_names.iter().any(|name| name == "virelo.sqlite3"));
        assert!(archive_names
            .iter()
            .any(|name| name.starts_with("documents/")));
        assert!(archive.by_name("README.txt").is_ok());

        let stats = dashboard_stats(&state).expect("dashboard stats");
        assert_eq!(stats.clients, 1);
        assert_eq!(stats.active_projects, 1);
        assert_eq!(stats.notes, 1);
        assert_eq!(stats.documents, 1);

        let data_dir = state
            .documents_dir
            .parent()
            .expect("test data directory")
            .to_path_buf();
        drop(state);
        let _ = fs::remove_dir_all(data_dir);
    }
}
