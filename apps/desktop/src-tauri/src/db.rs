use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::models::{
    Client, CompanyProfile, DashboardStats, DocumentRecord, Note, Project, SearchHit,
};

pub struct AppState {
    pub conn: Mutex<Connection>,
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
        let conn = Connection::open(database_path).map_err(|error| error.to_string())?;
        configure_database(&conn)?;
        migrate(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            documents_dir,
        })
    }
}

fn configure_database(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )
    .map_err(|error| error.to_string())
}

fn migrate(conn: &Connection) -> Result<(), String> {
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
            "SELECT id, title, file_name, file_path, client_id, project_id, created_at
         FROM documents ORDER BY created_at DESC, id DESC",
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
                project_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn import_document(
    state: &AppState,
    source_path: String,
    title: String,
    client_id: Option<i64>,
    project_id: Option<i64>,
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
        "INSERT INTO documents (title, file_name, file_path, client_id, project_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            if title.trim().is_empty() {
                file_name
            } else {
                title.trim()
            },
            file_name,
            destination_text,
            client_id,
            project_id
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

pub fn global_search(state: &AppState, query: String) -> Result<Vec<SearchHit>, String> {
    let query = query.trim();
    if query.len() < 2 {
        return Ok(Vec::new());
    }

    let pattern = format!("%{query}%");
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"
        SELECT 'client', id, name, COALESCE(email, '') || ' ' || COALESCE(tax_id, '')
        FROM clients WHERE name LIKE ?1 OR email LIKE ?1 OR tax_id LIKE ?1 OR notes LIKE ?1
        UNION ALL
        SELECT 'project', id, name, status || ' ' || COALESCE(due_date, '')
        FROM projects WHERE name LIKE ?1 OR notes LIKE ?1
        UNION ALL
        SELECT 'note', id, title, tags
        FROM notes WHERE title LIKE ?1 OR body_markdown LIKE ?1 OR tags LIKE ?1
        UNION ALL
        SELECT 'document', id, title, file_name
        FROM documents WHERE title LIKE ?1 OR file_name LIKE ?1
        LIMIT 30
        "#,
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([pattern], |row| {
            Ok(SearchHit {
                kind: row.get(0)?,
                id: row.get(1)?,
                title: row.get(2)?,
                subtitle: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}
