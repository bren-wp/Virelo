use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::MutexGuard,
};

use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::json;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::{
    db::{self, AppState},
    models::{
        ActivityRecord, AppInfo, BankAccount, Client, ClientContact, CompanyProfile,
        ContractRecord, DocumentInput, DocumentRecord, FinanceRecord, Note, Project, SearchHit,
        TaskRecord,
    },
};

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| error.to_string())?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    if !existing.iter().any(|name| name == column) {
        conn.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {definition}"
        ))
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            status TEXT NOT NULL DEFAULT 'Otvoren',
            priority TEXT NOT NULL DEFAULT 'Normalan',
            due_date TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL DEFAULT 'Bilješka',
            title TEXT NOT NULL,
            details TEXT NOT NULL DEFAULT '',
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            happened_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS finance_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL DEFAULT 'Račun',
            number TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            amount_cents INTEGER NOT NULL DEFAULT 0,
            currency TEXT NOT NULL DEFAULT 'EUR',
            status TEXT NOT NULL DEFAULT 'Nacrt',
            issue_date TEXT NOT NULL DEFAULT '',
            due_date TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS client_contacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS bank_accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label TEXT NOT NULL,
            iban TEXT NOT NULL,
            bic TEXT NOT NULL DEFAULT '',
            bank_name TEXT NOT NULL DEFAULT '',
            currency TEXT NOT NULL DEFAULT 'EUR',
            is_default INTEGER NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS contracts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            number TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            document_id INTEGER REFERENCES documents(id) ON DELETE SET NULL,
            status TEXT NOT NULL DEFAULT 'Aktivan',
            signed_date TEXT NOT NULL DEFAULT '',
            start_date TEXT NOT NULL DEFAULT '',
            end_date TEXT NOT NULL DEFAULT '',
            value_cents INTEGER NOT NULL DEFAULT 0,
            currency TEXT NOT NULL DEFAULT 'EUR',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_status_due ON tasks(status, due_date);
        CREATE INDEX IF NOT EXISTS idx_activities_happened_at ON activities(happened_at);
        CREATE INDEX IF NOT EXISTS idx_finance_kind_status ON finance_records(kind, status);
        CREATE INDEX IF NOT EXISTS idx_client_contacts_client ON client_contacts(client_id);
        CREATE INDEX IF NOT EXISTS idx_client_contacts_name ON client_contacts(name);
        CREATE INDEX IF NOT EXISTS idx_bank_accounts_default ON bank_accounts(is_default);
        CREATE INDEX IF NOT EXISTS idx_contracts_client ON contracts(client_id);
        CREATE INDEX IF NOT EXISTS idx_contracts_project ON contracts(project_id);
        CREATE INDEX IF NOT EXISTS idx_contracts_status_end ON contracts(status, end_date);
        "#,
    )
    .map_err(|error| error.to_string())?;

    ensure_column(
        conn,
        "documents",
        "category",
        "TEXT NOT NULL DEFAULT 'Ostalo'",
    )?;
    ensure_column(conn, "documents", "tags", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(conn, "documents", "description", "TEXT NOT NULL DEFAULT ''")?;
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_documents_category ON documents(category);")
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn lock(state: &AppState) -> Result<MutexGuard<'_, Connection>, String> {
    state
        .conn
        .lock()
        .map_err(|_| "Baza podataka nije dostupna.".to_string())
}

pub fn update_client(state: &AppState, client: Client) -> Result<(), String> {
    if client.id <= 0 || client.name.trim().is_empty() {
        return Err("Klijent nije valjan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE clients SET
           name=?2, tax_id=?3, registration_id=?4, email=?5, phone=?6, website=?7,
           address=?8, city=?9, country=?10, status=?11, notes=?12, updated_at=CURRENT_TIMESTAMP
           WHERE id=?1"#,
        params![
            client.id,
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
    Ok(())
}

pub fn list_client_contacts(state: &AppState) -> Result<Vec<ClientContact>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"SELECT cc.id, cc.client_id, c.name, cc.name, cc.role, cc.email, cc.phone,
                      cc.notes, cc.created_at, cc.updated_at
               FROM client_contacts cc
               INNER JOIN clients c ON c.id = cc.client_id
               ORDER BY c.name COLLATE NOCASE, cc.name COLLATE NOCASE"#,
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ClientContact {
                id: row.get(0)?,
                client_id: row.get(1)?,
                client_name: row.get(2)?,
                name: row.get(3)?,
                role: row.get(4)?,
                email: row.get(5)?,
                phone: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_client_contact(state: &AppState, contact: ClientContact) -> Result<i64, String> {
    if contact.client_id <= 0 || contact.name.trim().is_empty() {
        return Err("Klijent i ime kontakta su obavezni.".into());
    }

    let conn = lock(state)?;
    conn.execute(
        r#"INSERT INTO client_contacts (client_id, name, role, email, phone, notes)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        params![
            contact.client_id,
            contact.name.trim(),
            contact.role.trim(),
            contact.email.trim(),
            contact.phone.trim(),
            contact.notes
        ],
    )
    .map_err(|error| error.to_string())?;

    Ok(conn.last_insert_rowid())
}

pub fn update_client_contact(state: &AppState, contact: ClientContact) -> Result<(), String> {
    if contact.id <= 0 || contact.client_id <= 0 || contact.name.trim().is_empty() {
        return Err("Kontakt nije valjan.".into());
    }

    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE client_contacts
           SET client_id=?2, name=?3, role=?4, email=?5, phone=?6, notes=?7,
               updated_at=CURRENT_TIMESTAMP
           WHERE id=?1"#,
        params![
            contact.id,
            contact.client_id,
            contact.name.trim(),
            contact.role.trim(),
            contact.email.trim(),
            contact.phone.trim(),
            contact.notes
        ],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn delete_client_contact(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM client_contacts WHERE id=?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_bank_accounts(state: &AppState) -> Result<Vec<BankAccount>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"SELECT id, label, iban, bic, bank_name, currency, is_default, notes,
                      created_at, updated_at
               FROM bank_accounts
               ORDER BY is_default DESC, label COLLATE NOCASE, id DESC"#,
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BankAccount {
                id: row.get(0)?,
                label: row.get(1)?,
                iban: row.get(2)?,
                bic: row.get(3)?,
                bank_name: row.get(4)?,
                currency: row.get(5)?,
                is_default: row.get::<_, i64>(6)? != 0,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_bank_account(state: &AppState, account: BankAccount) -> Result<i64, String> {
    if account.label.trim().is_empty() || account.iban.trim().is_empty() {
        return Err("Naziv računa i IBAN su obavezni.".into());
    }

    let mut conn = lock(state)?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    if account.is_default {
        tx.execute("UPDATE bank_accounts SET is_default=0", [])
            .map_err(|error| error.to_string())?;
    }
    tx.execute(
        r#"INSERT INTO bank_accounts
           (label, iban, bic, bank_name, currency, is_default, notes)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
        params![
            account.label.trim(),
            account.iban.trim(),
            account.bic.trim(),
            account.bank_name.trim(),
            account.currency.trim(),
            if account.is_default { 1 } else { 0 },
            account.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    let id = tx.last_insert_rowid();
    tx.commit().map_err(|error| error.to_string())?;
    Ok(id)
}

pub fn update_bank_account(state: &AppState, account: BankAccount) -> Result<(), String> {
    if account.id <= 0 || account.label.trim().is_empty() || account.iban.trim().is_empty() {
        return Err("Bankovni račun nije valjan.".into());
    }

    let mut conn = lock(state)?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    if account.is_default {
        tx.execute(
            "UPDATE bank_accounts SET is_default=0 WHERE id<>?1",
            [account.id],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.execute(
        r#"UPDATE bank_accounts
           SET label=?2, iban=?3, bic=?4, bank_name=?5, currency=?6, is_default=?7,
               notes=?8, updated_at=CURRENT_TIMESTAMP
           WHERE id=?1"#,
        params![
            account.id,
            account.label.trim(),
            account.iban.trim(),
            account.bic.trim(),
            account.bank_name.trim(),
            account.currency.trim(),
            if account.is_default { 1 } else { 0 },
            account.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

pub fn delete_bank_account(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM bank_accounts WHERE id=?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_contracts(state: &AppState) -> Result<Vec<ContractRecord>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"SELECT co.id, co.number, co.title, co.client_id, c.name, co.project_id, p.name,
                      co.document_id, d.title, co.status, co.signed_date, co.start_date,
                      co.end_date, co.value_cents, co.currency, co.notes, co.created_at,
                      co.updated_at
               FROM contracts co
               LEFT JOIN clients c ON c.id=co.client_id
               LEFT JOIN projects p ON p.id=co.project_id
               LEFT JOIN documents d ON d.id=co.document_id
               ORDER BY CASE WHEN co.status='Aktivan' THEN 0 ELSE 1 END,
                        CASE WHEN co.end_date='' THEN 1 ELSE 0 END, co.end_date, co.id DESC"#,
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ContractRecord {
                id: row.get(0)?,
                number: row.get(1)?,
                title: row.get(2)?,
                client_id: row.get(3)?,
                client_name: row.get(4)?,
                project_id: row.get(5)?,
                project_name: row.get(6)?,
                document_id: row.get(7)?,
                document_title: row.get(8)?,
                status: row.get(9)?,
                signed_date: row.get(10)?,
                start_date: row.get(11)?,
                end_date: row.get(12)?,
                value_cents: row.get(13)?,
                currency: row.get(14)?,
                notes: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_contract(state: &AppState, contract: ContractRecord) -> Result<i64, String> {
    if contract.title.trim().is_empty() {
        return Err("Naziv ugovora je obavezan.".into());
    }

    let conn = lock(state)?;
    conn.execute(
        r#"INSERT INTO contracts
           (number, title, client_id, project_id, document_id, status, signed_date,
            start_date, end_date, value_cents, currency, notes)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
        params![
            contract.number.trim(),
            contract.title.trim(),
            contract.client_id,
            contract.project_id,
            contract.document_id,
            contract.status.trim(),
            contract.signed_date.trim(),
            contract.start_date.trim(),
            contract.end_date.trim(),
            contract.value_cents,
            contract.currency.trim(),
            contract.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_contract(state: &AppState, contract: ContractRecord) -> Result<(), String> {
    if contract.id <= 0 || contract.title.trim().is_empty() {
        return Err("Ugovor nije valjan.".into());
    }

    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE contracts
           SET number=?2, title=?3, client_id=?4, project_id=?5, document_id=?6, status=?7,
               signed_date=?8, start_date=?9, end_date=?10, value_cents=?11, currency=?12,
               notes=?13, updated_at=CURRENT_TIMESTAMP
           WHERE id=?1"#,
        params![
            contract.id,
            contract.number.trim(),
            contract.title.trim(),
            contract.client_id,
            contract.project_id,
            contract.document_id,
            contract.status.trim(),
            contract.signed_date.trim(),
            contract.start_date.trim(),
            contract.end_date.trim(),
            contract.value_cents,
            contract.currency.trim(),
            contract.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_contract(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM contracts WHERE id=?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn update_project(state: &AppState, project: Project) -> Result<(), String> {
    if project.id <= 0 || project.name.trim().is_empty() {
        return Err("Projekt nije valjan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE projects SET
           client_id=?2, name=?3, status=?4, priority=?5, due_date=?6,
           value_cents=?7, currency=?8, notes=?9, updated_at=CURRENT_TIMESTAMP
           WHERE id=?1"#,
        params![
            project.id,
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
    Ok(())
}

pub fn update_note(state: &AppState, note: Note) -> Result<(), String> {
    if note.id <= 0 || note.title.trim().is_empty() {
        return Err("Bilješka nije valjana.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE notes SET
           title=?2, body_markdown=?3, client_id=?4, project_id=?5, tags=?6,
           updated_at=CURRENT_TIMESTAMP WHERE id=?1"#,
        params![
            note.id,
            note.title.trim(),
            note.body_markdown,
            note.client_id,
            note.project_id,
            note.tags.trim()
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn update_document(
    state: &AppState,
    id: i64,
    document: DocumentInput,
) -> Result<(), String> {
    if id <= 0 || document.title.trim().is_empty() {
        return Err("Dokument nije valjan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE documents SET
           title=?2, client_id=?3, project_id=?4, category=?5, tags=?6, description=?7
           WHERE id=?1"#,
        params![
            id,
            document.title.trim(),
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
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_tasks(state: &AppState) -> Result<Vec<TaskRecord>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"SELECT t.id, t.title, t.client_id, c.name, t.project_id, p.name,
                      t.status, t.priority, t.due_date, t.notes, t.created_at, t.updated_at
               FROM tasks t
               LEFT JOIN clients c ON c.id=t.client_id
               LEFT JOIN projects p ON p.id=t.project_id
               ORDER BY CASE WHEN t.status='Završen' THEN 1 ELSE 0 END,
                        CASE WHEN t.due_date='' THEN 1 ELSE 0 END, t.due_date, t.id DESC"#,
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TaskRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                client_id: row.get(2)?,
                client_name: row.get(3)?,
                project_id: row.get(4)?,
                project_name: row.get(5)?,
                status: row.get(6)?,
                priority: row.get(7)?,
                due_date: row.get(8)?,
                notes: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_task(state: &AppState, task: TaskRecord) -> Result<i64, String> {
    if task.title.trim().is_empty() {
        return Err("Naslov zadatka je obavezan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"INSERT INTO tasks
           (title, client_id, project_id, status, priority, due_date, notes)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
        params![
            task.title.trim(),
            task.client_id,
            task.project_id,
            task.status.trim(),
            task.priority.trim(),
            task.due_date.trim(),
            task.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_task(state: &AppState, task: TaskRecord) -> Result<(), String> {
    if task.id <= 0 || task.title.trim().is_empty() {
        return Err("Zadatak nije valjan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE tasks SET title=?2, client_id=?3, project_id=?4, status=?5,
           priority=?6, due_date=?7, notes=?8, updated_at=CURRENT_TIMESTAMP WHERE id=?1"#,
        params![
            task.id,
            task.title.trim(),
            task.client_id,
            task.project_id,
            task.status.trim(),
            task.priority.trim(),
            task.due_date.trim(),
            task.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_task(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM tasks WHERE id=?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_activities(state: &AppState) -> Result<Vec<ActivityRecord>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"SELECT a.id, a.kind, a.title, a.details, a.client_id, c.name,
                      a.project_id, p.name, a.happened_at, a.created_at
               FROM activities a
               LEFT JOIN clients c ON c.id=a.client_id
               LEFT JOIN projects p ON p.id=a.project_id
               ORDER BY a.happened_at DESC, a.id DESC"#,
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ActivityRecord {
                id: row.get(0)?,
                kind: row.get(1)?,
                title: row.get(2)?,
                details: row.get(3)?,
                client_id: row.get(4)?,
                client_name: row.get(5)?,
                project_id: row.get(6)?,
                project_name: row.get(7)?,
                happened_at: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_activity(state: &AppState, activity: ActivityRecord) -> Result<i64, String> {
    if activity.title.trim().is_empty() {
        return Err("Naslov aktivnosti je obavezan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"INSERT INTO activities
           (kind, title, details, client_id, project_id, happened_at)
           VALUES (?1, ?2, ?3, ?4, ?5,
             CASE WHEN ?6='' THEN CURRENT_TIMESTAMP ELSE ?6 END)"#,
        params![
            activity.kind.trim(),
            activity.title.trim(),
            activity.details,
            activity.client_id,
            activity.project_id,
            activity.happened_at.trim()
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_activity(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM activities WHERE id=?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_finance_records(state: &AppState) -> Result<Vec<FinanceRecord>, String> {
    let conn = lock(state)?;
    let mut stmt = conn
        .prepare(
            r#"SELECT f.id, f.kind, f.number, f.title, f.client_id, c.name,
                      f.amount_cents, f.currency, f.status, f.issue_date, f.due_date,
                      f.notes, f.created_at, f.updated_at
               FROM finance_records f
               LEFT JOIN clients c ON c.id=f.client_id
               ORDER BY f.issue_date DESC, f.id DESC"#,
        )
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(FinanceRecord {
                id: row.get(0)?,
                kind: row.get(1)?,
                number: row.get(2)?,
                title: row.get(3)?,
                client_id: row.get(4)?,
                client_name: row.get(5)?,
                amount_cents: row.get(6)?,
                currency: row.get(7)?,
                status: row.get(8)?,
                issue_date: row.get(9)?,
                due_date: row.get(10)?,
                notes: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_finance_record(state: &AppState, record: FinanceRecord) -> Result<i64, String> {
    if record.title.trim().is_empty() {
        return Err("Naziv financijskog zapisa je obavezan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"INSERT INTO finance_records
           (kind, number, title, client_id, amount_cents, currency, status, issue_date, due_date, notes)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params![
            record.kind.trim(),
            record.number.trim(),
            record.title.trim(),
            record.client_id,
            record.amount_cents,
            record.currency.trim(),
            record.status.trim(),
            record.issue_date.trim(),
            record.due_date.trim(),
            record.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_finance_record(state: &AppState, record: FinanceRecord) -> Result<(), String> {
    if record.id <= 0 || record.title.trim().is_empty() {
        return Err("Financijski zapis nije valjan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        r#"UPDATE finance_records SET kind=?2, number=?3, title=?4, client_id=?5,
           amount_cents=?6, currency=?7, status=?8, issue_date=?9, due_date=?10,
           notes=?11, updated_at=CURRENT_TIMESTAMP WHERE id=?1"#,
        params![
            record.id,
            record.kind.trim(),
            record.number.trim(),
            record.title.trim(),
            record.client_id,
            record.amount_cents,
            record.currency.trim(),
            record.status.trim(),
            record.issue_date.trim(),
            record.due_date.trim(),
            record.notes
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn delete_finance_record(state: &AppState, id: i64) -> Result<(), String> {
    let conn = lock(state)?;
    conn.execute("DELETE FROM finance_records WHERE id=?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn app_info(_state: &AppState) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[derive(Serialize)]
#[serde(rename = "virelo_export")]
struct ExportBundle {
    version: String,
    company: CompanyProfile,
    clients: Vec<Client>,
    contacts: Vec<ClientContact>,
    bank_accounts: Vec<BankAccount>,
    contracts: Vec<ContractRecord>,
    projects: Vec<Project>,
    notes: Vec<Note>,
    documents: Vec<DocumentRecord>,
    tasks: Vec<TaskRecord>,
    activities: Vec<ActivityRecord>,
    finance: Vec<FinanceRecord>,
}

fn build_export_bundle(state: &AppState) -> Result<ExportBundle, String> {
    Ok(ExportBundle {
        version: env!("CARGO_PKG_VERSION").to_string(),
        company: db::get_company_profile(state)?,
        clients: db::list_clients(state)?,
        contacts: list_client_contacts(state)?,
        bank_accounts: list_bank_accounts(state)?,
        contracts: list_contracts(state)?,
        projects: db::list_projects(state)?,
        notes: db::list_notes(state)?,
        documents: db::list_documents(state)?,
        tasks: list_tasks(state)?,
        activities: list_activities(state)?,
        finance: list_finance_records(state)?,
    })
}

pub fn export_workspace_json(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let payload = json!({
        "format": "virelo-workspace",
        "version": env!("CARGO_PKG_VERSION"),
        "company": db::get_company_profile(state)?,
        "clients": db::list_clients(state)?,
        "contacts": list_client_contacts(state)?,
        "bank_accounts": list_bank_accounts(state)?,
        "contracts": list_contracts(state)?,
        "projects": db::list_projects(state)?,
        "notes": db::list_notes(state)?,
        "documents": db::list_documents(state)?,
        "tasks": list_tasks(state)?,
        "activities": list_activities(state)?,
        "finance": list_finance_records(state)?
    });

    let serialized = serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
    fs::write(destination, serialized).map_err(|error| error.to_string())
}

pub fn export_workspace_yaml(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let bundle = build_export_bundle(state)?;
    let serialized = serde_yaml::to_string(&bundle).map_err(|error| error.to_string())?;
    fs::write(destination, serialized).map_err(|error| error.to_string())
}

pub fn export_workspace_xml(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let bundle = build_export_bundle(state)?;
    let serialized = quick_xml::se::to_string(&bundle).map_err(|error| error.to_string())?;
    let xml = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{serialized}");
    fs::write(destination, xml).map_err(|error| error.to_string())
}

fn csv_cell(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn export_workspace_csv(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let mut rows = vec![
        "vrsta,id,naziv,klijent,status,datum,iznos,valuta,porezni_id,email,telefon,biljeske"
            .to_string(),
    ];

    for client in db::list_clients(state)? {
        rows.push(
            [
                csv_cell("klijent"),
                csv_cell(&client.id.to_string()),
                csv_cell(&client.name),
                csv_cell(""),
                csv_cell(&client.status),
                csv_cell(&client.created_at),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&client.tax_id),
                csv_cell(&client.email),
                csv_cell(&client.phone),
                csv_cell(&client.notes),
            ]
            .join(","),
        );
    }

    for contact in list_client_contacts(state)? {
        rows.push(
            [
                csv_cell("kontakt"),
                csv_cell(&contact.id.to_string()),
                csv_cell(&contact.name),
                csv_cell(contact.client_name.as_deref().unwrap_or("")),
                csv_cell(&contact.role),
                csv_cell(&contact.created_at),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&contact.email),
                csv_cell(&contact.phone),
                csv_cell(&contact.notes),
            ]
            .join(","),
        );
    }

    for account in list_bank_accounts(state)? {
        rows.push(
            [
                csv_cell("bankovni_racun"),
                csv_cell(&account.id.to_string()),
                csv_cell(&account.label),
                csv_cell(""),
                csv_cell(if account.is_default { "Zadani" } else { "" }),
                csv_cell(&account.created_at),
                csv_cell(""),
                csv_cell(&account.currency),
                csv_cell(&account.iban),
                csv_cell(&account.bank_name),
                csv_cell(&account.bic),
                csv_cell(&account.notes),
            ]
            .join(","),
        );
    }

    for contract in list_contracts(state)? {
        rows.push(
            [
                csv_cell("ugovor"),
                csv_cell(&contract.id.to_string()),
                csv_cell(&contract.title),
                csv_cell(contract.client_name.as_deref().unwrap_or("")),
                csv_cell(&contract.status),
                csv_cell(&contract.end_date),
                csv_cell(&(contract.value_cents as f64 / 100.0).to_string()),
                csv_cell(&contract.currency),
                csv_cell(&contract.number),
                csv_cell(contract.project_name.as_deref().unwrap_or("")),
                csv_cell(contract.document_title.as_deref().unwrap_or("")),
                csv_cell(&contract.notes),
            ]
            .join(","),
        );
    }

    for project in db::list_projects(state)? {
        rows.push(
            [
                csv_cell("projekt"),
                csv_cell(&project.id.to_string()),
                csv_cell(&project.name),
                csv_cell(project.client_name.as_deref().unwrap_or("")),
                csv_cell(&project.status),
                csv_cell(&project.due_date),
                csv_cell(&(project.value_cents as f64 / 100.0).to_string()),
                csv_cell(&project.currency),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&project.notes),
            ]
            .join(","),
        );
    }

    for task in list_tasks(state)? {
        rows.push(
            [
                csv_cell("zadatak"),
                csv_cell(&task.id.to_string()),
                csv_cell(&task.title),
                csv_cell(task.client_name.as_deref().unwrap_or("")),
                csv_cell(&task.status),
                csv_cell(&task.due_date),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&task.notes),
            ]
            .join(","),
        );
    }

    for note in db::list_notes(state)? {
        rows.push(
            [
                csv_cell("biljeska"),
                csv_cell(&note.id.to_string()),
                csv_cell(&note.title),
                csv_cell(""),
                csv_cell(&note.tags),
                csv_cell(&note.updated_at),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&note.body_markdown),
            ]
            .join(","),
        );
    }

    for document in db::list_documents(state)? {
        rows.push(
            [
                csv_cell("dokument"),
                csv_cell(&document.id.to_string()),
                csv_cell(&document.title),
                csv_cell(document.client_name.as_deref().unwrap_or("")),
                csv_cell(&document.category),
                csv_cell(&document.created_at),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&document.file_name),
                csv_cell(document.project_name.as_deref().unwrap_or("")),
                csv_cell(&document.tags),
                csv_cell(&document.description),
            ]
            .join(","),
        );
    }

    for activity in list_activities(state)? {
        rows.push(
            [
                csv_cell("aktivnost"),
                csv_cell(&activity.id.to_string()),
                csv_cell(&activity.title),
                csv_cell(activity.client_name.as_deref().unwrap_or("")),
                csv_cell(&activity.kind),
                csv_cell(&activity.happened_at),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&activity.details),
            ]
            .join(","),
        );
    }

    for record in list_finance_records(state)? {
        rows.push(
            [
                csv_cell(&record.kind.to_lowercase()),
                csv_cell(&record.id.to_string()),
                csv_cell(&record.title),
                csv_cell(record.client_name.as_deref().unwrap_or("")),
                csv_cell(&record.status),
                csv_cell(&record.issue_date),
                csv_cell(&(record.amount_cents as f64 / 100.0).to_string()),
                csv_cell(&record.currency),
                csv_cell(""),
                csv_cell(""),
                csv_cell(""),
                csv_cell(&record.notes),
            ]
            .join(","),
        );
    }

    fs::write(destination, format!("\u{feff}{}", rows.join("\n")))
        .map_err(|error| error.to_string())
}

pub fn export_workspace_markdown(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let company = db::get_company_profile(state)?;
    let clients = db::list_clients(state)?;
    let contacts = list_client_contacts(state)?;
    let bank_accounts = list_bank_accounts(state)?;
    let contracts = list_contracts(state)?;
    let projects = db::list_projects(state)?;
    let tasks = list_tasks(state)?;
    let notes = db::list_notes(state)?;
    let activities = list_activities(state)?;
    let finance = list_finance_records(state)?;

    let mut md = format!(
        "# Virelo izvoz\n\n## Firma\n\n**{}**  \n{} {}  \n{} {}  \nIBAN: {}  \n\n",
        company.name,
        company.address,
        company.city,
        company.tax_id,
        company.registration_id,
        company.iban
    );

    md.push_str("## Klijenti\n\n");
    for client in clients {
        md.push_str(&format!(
            "### {}\n- Status: {}\n- Porezni ID: {}\n- E-mail: {}\n- Telefon: {}\n- Grad: {}\n\n{}\n\n",
            client.name,
            client.status,
            client.tax_id,
            client.email,
            client.phone,
            client.city,
            client.notes
        ));
    }

    md.push_str("## Kontakti\n\n");
    for contact in contacts {
        md.push_str(&format!(
            "- **{}** — {} · {} · {} · {}\n",
            contact.name,
            contact.client_name.unwrap_or_default(),
            contact.role,
            contact.email,
            contact.phone
        ));
    }

    md.push_str("\n## Bankovni računi\n\n");
    for account in bank_accounts {
        md.push_str(&format!(
            "- **{}** · {} · {} · {}{}\n",
            account.label,
            account.iban,
            account.bank_name,
            account.currency,
            if account.is_default { " · zadani" } else { "" }
        ));
    }

    md.push_str("\n## Ugovori\n\n");
    for contract in contracts {
        md.push_str(&format!(
            "### {}\n- Broj: {}\n- Klijent: {}\n- Projekt: {}\n- Status: {}\n- Početak: {}\n- Završetak: {}\n- Vrijednost: {:.2} {}\n\n{}\n\n",
            contract.title,
            contract.number,
            contract.client_name.unwrap_or_default(),
            contract.project_name.unwrap_or_default(),
            contract.status,
            contract.start_date,
            contract.end_date,
            contract.value_cents as f64 / 100.0,
            contract.currency,
            contract.notes
        ));
    }

    md.push_str("\n## Projekti\n\n");
    for project in projects {
        md.push_str(&format!(
            "### {}\n- Klijent: {}\n- Status: {}\n- Prioritet: {}\n- Rok: {}\n- Vrijednost: {:.2} {}\n\n{}\n\n",
            project.name,
            project.client_name.unwrap_or_default(),
            project.status,
            project.priority,
            project.due_date,
            project.value_cents as f64 / 100.0,
            project.currency,
            project.notes
        ));
    }

    md.push_str("## Zadaci\n\n");
    for task in tasks {
        md.push_str(&format!(
            "- [{}] **{}** · {} · rok {}\n",
            if task.status == "Završen" { "x" } else { " " },
            task.title,
            task.priority,
            task.due_date
        ));
    }

    md.push_str("\n## Bilješke\n\n");
    for note in notes {
        md.push_str(&format!("### {}\n\n{}\n\n", note.title, note.body_markdown));
    }

    md.push_str("## Dokumenti\n\n");
    for document in db::list_documents(state)? {
        md.push_str(&format!(
            "- **{}** · {} · {} · {}{}{}\n  {}\n",
            document.title,
            document.category,
            document.file_name,
            document.created_at,
            document
                .client_name
                .as_deref()
                .map(|name| format!(" · klijent: {name}"))
                .unwrap_or_default(),
            if document.tags.is_empty() {
                String::new()
            } else {
                format!(" · oznake: {}", document.tags)
            },
            document.description
        ));
    }

    md.push_str("\n## Aktivnosti\n\n");
    for activity in activities {
        md.push_str(&format!(
            "- **{}** · {} · {}\n  {}\n",
            activity.title, activity.kind, activity.happened_at, activity.details
        ));
    }

    md.push_str("\n## Financije\n\n");
    for record in finance {
        md.push_str(&format!(
            "- **{}** · {} · {:.2} {} · {}\n",
            record.title,
            record.kind,
            record.amount_cents as f64 / 100.0,
            record.currency,
            record.status
        ));
    }

    fs::write(destination, md).map_err(|error| error.to_string())
}

pub fn export_workspace_html(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let company = db::get_company_profile(state)?;
    let clients = db::list_clients(state)?;
    let contacts = list_client_contacts(state)?;
    let bank_accounts = list_bank_accounts(state)?;
    let contracts = list_contracts(state)?;
    let projects = db::list_projects(state)?;
    let tasks = list_tasks(state)?;
    let notes = db::list_notes(state)?;
    let documents = db::list_documents(state)?;
    let activities = list_activities(state)?;
    let finance = list_finance_records(state)?;

    let mut html = format!(
        "<!doctype html><html lang=\"hr\"><meta charset=\"utf-8\"><title>Virelo izvoz</title><style>body{{font-family:system-ui;margin:40px;max-width:1100px}}table{{width:100%;border-collapse:collapse;margin:16px 0 32px}}th,td{{border-bottom:1px solid #ddd;padding:8px;text-align:left}}h1,h2{{margin-top:28px}}small{{color:#666}}</style><body><h1>Virelo</h1><h2>{}</h2><p>{}, {}<br>{}<br>{}</p>",
        html_escape(&company.name),
        html_escape(&company.address),
        html_escape(&company.city),
        html_escape(&company.email),
        html_escape(&company.phone)
    );

    html.push_str("<h2>Klijenti</h2><table><tr><th>Naziv</th><th>Status</th><th>ID</th><th>E-mail</th><th>Telefon</th></tr>");
    for client in clients {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&client.name),
            html_escape(&client.status),
            html_escape(&client.tax_id),
            html_escape(&client.email),
            html_escape(&client.phone)
        ));
    }
    html.push_str("</table><h2>Kontakti</h2><table><tr><th>Ime</th><th>Klijent</th><th>Uloga</th><th>E-mail</th><th>Telefon</th></tr>");
    for contact in contacts {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&contact.name),
            html_escape(contact.client_name.as_deref().unwrap_or("")),
            html_escape(&contact.role),
            html_escape(&contact.email),
            html_escape(&contact.phone)
        ));
    }
    html.push_str("</table><h2>Bankovni računi</h2><table><tr><th>Naziv</th><th>IBAN</th><th>Banka</th><th>Valuta</th><th>Zadani</th></tr>");
    for account in bank_accounts {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&account.label),
            html_escape(&account.iban),
            html_escape(&account.bank_name),
            html_escape(&account.currency),
            if account.is_default { "Da" } else { "Ne" }
        ));
    }

    html.push_str("</table><h2>Ugovori</h2><table><tr><th>Ugovor</th><th>Klijent</th><th>Status</th><th>Završetak</th><th>Vrijednost</th></tr>");
    for contract in contracts {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2} {}</td></tr>",
            html_escape(&contract.title),
            html_escape(contract.client_name.as_deref().unwrap_or("")),
            html_escape(&contract.status),
            html_escape(&contract.end_date),
            contract.value_cents as f64 / 100.0,
            html_escape(&contract.currency)
        ));
    }

    html.push_str("</table><h2>Projekti</h2><table><tr><th>Projekt</th><th>Klijent</th><th>Status</th><th>Rok</th><th>Vrijednost</th></tr>");
    for project in projects {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2} {}</td></tr>",
            html_escape(&project.name),
            html_escape(project.client_name.as_deref().unwrap_or("")),
            html_escape(&project.status),
            html_escape(&project.due_date),
            project.value_cents as f64 / 100.0,
            html_escape(&project.currency)
        ));
    }
    html.push_str("</table><h2>Zadaci</h2><table><tr><th>Zadatak</th><th>Status</th><th>Prioritet</th><th>Rok</th></tr>");
    for task in tasks {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&task.title),
            html_escape(&task.status),
            html_escape(&task.priority),
            html_escape(&task.due_date)
        ));
    }
    html.push_str(
        "</table><h2>Bilješke</h2><table><tr><th>Naslov</th><th>Oznake</th><th>Sadržaj</th></tr>",
    );
    for note in notes {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&note.title),
            html_escape(&note.tags),
            html_escape(&note.body_markdown)
        ));
    }

    html.push_str(
        "</table><h2>Dokumenti</h2><table><tr><th>Naziv</th><th>Kategorija</th><th>Klijent</th><th>Projekt</th><th>Oznake</th><th>Datoteka</th><th>Opis</th><th>Datum</th></tr>",
    );
    for document in documents {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&document.title),
            html_escape(&document.category),
            html_escape(document.client_name.as_deref().unwrap_or("")),
            html_escape(document.project_name.as_deref().unwrap_or("")),
            html_escape(&document.tags),
            html_escape(&document.file_name),
            html_escape(&document.description),
            html_escape(&document.created_at)
        ));
    }

    html.push_str("</table><h2>Aktivnosti</h2><table><tr><th>Naslov</th><th>Vrsta</th><th>Klijent</th><th>Datum</th><th>Detalji</th></tr>");
    for activity in activities {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&activity.title),
            html_escape(&activity.kind),
            html_escape(activity.client_name.as_deref().unwrap_or("")),
            html_escape(&activity.happened_at),
            html_escape(&activity.details)
        ));
    }

    html.push_str("</table><h2>Financije</h2><table><tr><th>Naziv</th><th>Vrsta</th><th>Status</th><th>Iznos</th></tr>");
    for record in finance {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.2} {}</td></tr>",
            html_escape(&record.title),
            html_escape(&record.kind),
            html_escape(&record.status),
            record.amount_cents as f64 / 100.0,
            html_escape(&record.currency)
        ));
    }
    html.push_str("</table></body></html>");

    fs::write(destination, html).map_err(|error| error.to_string())
}

pub fn export_workspace_archive(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    {
        let conn = lock(state)?;
        conn.execute_batch("PRAGMA wal_checkpoint(FULL);")
            .map_err(|error| error.to_string())?;
    }

    let payload = json!({
        "format": "virelo-archive",
        "version": env!("CARGO_PKG_VERSION"),
        "company": db::get_company_profile(state)?,
        "clients": db::list_clients(state)?,
        "contacts": list_client_contacts(state)?,
        "bank_accounts": list_bank_accounts(state)?,
        "contracts": list_contracts(state)?,
        "projects": db::list_projects(state)?,
        "notes": db::list_notes(state)?,
        "documents": db::list_documents(state)?,
        "tasks": list_tasks(state)?,
        "activities": list_activities(state)?,
        "finance": list_finance_records(state)?
    });

    let json_data = serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?;

    let file = fs::File::create(destination).map_err(|error| error.to_string())?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    archive
        .start_file("virelo-data.json", options)
        .map_err(|error| error.to_string())?;
    archive
        .write_all(&json_data)
        .map_err(|error| error.to_string())?;

    archive
        .start_file("README.txt", options)
        .map_err(|error| error.to_string())?;
    archive
        .write_all(
            b"Virelo arhiva\n\nSadrzaj:\n- virelo-data.json: citljiv strukturirani izvoz\n- virelo.sqlite3: kompletna baza podataka\n- documents/: kopije uvezenih dokumenata\n\nArhiva je standardni ZIP i moze se otvoriti bez Virela.\n",
        )
        .map_err(|error| error.to_string())?;

    archive
        .start_file("virelo.sqlite3", options)
        .map_err(|error| error.to_string())?;
    let mut database = fs::File::open(&state.database_path).map_err(|error| error.to_string())?;
    std::io::copy(&mut database, &mut archive).map_err(|error| error.to_string())?;

    if state.documents_dir.is_dir() {
        for entry in fs::read_dir(&state.documents_dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = entry
                .file_name()
                .to_str()
                .ok_or_else(|| "Naziv dokumenta nije valjan.".to_string())?
                .to_string();

            archive
                .start_file(format!("documents/{name}"), options)
                .map_err(|error| error.to_string())?;
            let mut source = fs::File::open(&path).map_err(|error| error.to_string())?;
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let read = source
                    .read(&mut buffer)
                    .map_err(|error| error.to_string())?;
                if read == 0 {
                    break;
                }
                archive
                    .write_all(&buffer[..read])
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    archive.finish().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn backup_database(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }
    if destination == state.database_path {
        return Err("Backup mora biti spremljen na drugu lokaciju.".into());
    }

    {
        let conn = lock(state)?;
        conn.execute_batch("PRAGMA wal_checkpoint(FULL);")
            .map_err(|error| error.to_string())?;
    }

    fs::copy(&state.database_path, destination)
        .map(|_| ())
        .map_err(|error| error.to_string())
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
            SELECT 'contact', cc.id, cc.name, c.name || ' ' || cc.role || ' ' || cc.email
            FROM client_contacts cc
            INNER JOIN clients c ON c.id=cc.client_id
            WHERE cc.name LIKE ?1 OR cc.role LIKE ?1 OR cc.email LIKE ?1 OR cc.phone LIKE ?1 OR cc.notes LIKE ?1
            UNION ALL
            SELECT 'bank', id, label, iban || ' ' || bank_name || ' ' || currency
            FROM bank_accounts WHERE label LIKE ?1 OR iban LIKE ?1 OR bic LIKE ?1 OR bank_name LIKE ?1 OR notes LIKE ?1
            UNION ALL
            SELECT 'contract', co.id, co.title, co.number || ' ' || co.status || ' ' || COALESCE(c.name, '')
            FROM contracts co
            LEFT JOIN clients c ON c.id=co.client_id
            WHERE co.title LIKE ?1 OR co.number LIKE ?1 OR co.notes LIKE ?1
            UNION ALL
            SELECT 'project', id, name, status || ' ' || COALESCE(due_date, '')
            FROM projects WHERE name LIKE ?1 OR notes LIKE ?1
            UNION ALL
            SELECT 'note', id, title, tags
            FROM notes WHERE title LIKE ?1 OR body_markdown LIKE ?1 OR tags LIKE ?1
            UNION ALL
            SELECT 'document', d.id, d.title,
                   d.category || ' ' || d.file_name || ' ' ||
                   COALESCE(c.name, '') || ' ' || COALESCE(p.name, '')
            FROM documents d
            LEFT JOIN clients c ON c.id=d.client_id
            LEFT JOIN projects p ON p.id=d.project_id
            WHERE d.title LIKE ?1 OR d.file_name LIKE ?1 OR d.category LIKE ?1
               OR d.tags LIKE ?1 OR d.description LIKE ?1
               OR c.name LIKE ?1 OR p.name LIKE ?1
            UNION ALL
            SELECT 'task', id, title, status || ' ' || priority
            FROM tasks WHERE title LIKE ?1 OR notes LIKE ?1
            UNION ALL
            SELECT 'activity', id, title, kind || ' ' || details
            FROM activities WHERE title LIKE ?1 OR details LIKE ?1
            UNION ALL
            SELECT 'finance', id, title, kind || ' ' || number || ' ' || status
            FROM finance_records WHERE title LIKE ?1 OR number LIKE ?1 OR notes LIKE ?1
            LIMIT 50
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
