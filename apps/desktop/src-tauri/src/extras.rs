use std::{fs, path::Path, sync::MutexGuard};

use rusqlite::{params, Connection};
use serde_json::json;

use crate::{
    db::{self, AppState},
    models::{
        ActivityRecord, AppInfo, Client, ClientContact, FinanceRecord, Note, Project, SearchHit,
        TaskRecord,
    },
};

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

        CREATE INDEX IF NOT EXISTS idx_tasks_status_due ON tasks(status, due_date);
        CREATE INDEX IF NOT EXISTS idx_activities_happened_at ON activities(happened_at);
        CREATE INDEX IF NOT EXISTS idx_finance_kind_status ON finance_records(kind, status);
        CREATE INDEX IF NOT EXISTS idx_client_contacts_client ON client_contacts(client_id);
        CREATE INDEX IF NOT EXISTS idx_client_contacts_name ON client_contacts(name);
        "#,
    )
    .map_err(|error| error.to_string())
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
    title: String,
    client_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<(), String> {
    if id <= 0 || title.trim().is_empty() {
        return Err("Dokument nije valjan.".into());
    }
    let conn = lock(state)?;
    conn.execute(
        "UPDATE documents SET title=?2, client_id=?3, project_id=?4 WHERE id=?1",
        params![id, title.trim(), client_id, project_id],
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

fn csv_cell(value: &str) -> String {
    let escaped = value.replace('"', """");
    format!(""{escaped}"")
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

    fs::write(destination, rows.join("\n")).map_err(|error| error.to_string())
}

pub fn export_workspace_markdown(state: &AppState, destination: String) -> Result<(), String> {
    let destination = Path::new(&destination);
    if destination.as_os_str().is_empty() {
        return Err("Odredište nije valjano.".into());
    }

    let company = db::get_company_profile(state)?;
    let clients = db::list_clients(state)?;
    let contacts = list_client_contacts(state)?;
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

    md.push_str("## Aktivnosti\n\n");
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
    let projects = db::list_projects(state)?;
    let tasks = list_tasks(state)?;
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
            SELECT 'project', id, name, status || ' ' || COALESCE(due_date, '')
            FROM projects WHERE name LIKE ?1 OR notes LIKE ?1
            UNION ALL
            SELECT 'note', id, title, tags
            FROM notes WHERE title LIKE ?1 OR body_markdown LIKE ?1 OR tags LIKE ?1
            UNION ALL
            SELECT 'document', id, title, file_name
            FROM documents WHERE title LIKE ?1 OR file_name LIKE ?1
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
