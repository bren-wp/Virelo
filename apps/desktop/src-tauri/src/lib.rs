mod db;
mod models;

use tauri::{Manager, State};

use db::AppState;
use models::{Client, CompanyProfile, DashboardStats, DocumentRecord, Note, Project, SearchHit};

#[tauri::command]
fn dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    db::dashboard_stats(&state)
}

#[tauri::command]
fn get_company_profile(state: State<'_, AppState>) -> Result<CompanyProfile, String> {
    db::get_company_profile(&state)
}

#[tauri::command]
fn save_company_profile(state: State<'_, AppState>, profile: CompanyProfile) -> Result<(), String> {
    db::save_company_profile(&state, profile)
}

#[tauri::command]
fn list_clients(state: State<'_, AppState>) -> Result<Vec<Client>, String> {
    db::list_clients(&state)
}

#[tauri::command]
fn create_client(state: State<'_, AppState>, client: Client) -> Result<i64, String> {
    db::create_client(&state, client)
}

#[tauri::command]
fn delete_client(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_client(&state, id)
}

#[tauri::command]
fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    db::list_projects(&state)
}

#[tauri::command]
fn create_project(state: State<'_, AppState>, project: Project) -> Result<i64, String> {
    db::create_project(&state, project)
}

#[tauri::command]
fn delete_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_project(&state, id)
}

#[tauri::command]
fn list_notes(state: State<'_, AppState>) -> Result<Vec<Note>, String> {
    db::list_notes(&state)
}

#[tauri::command]
fn create_note(state: State<'_, AppState>, note: Note) -> Result<i64, String> {
    db::create_note(&state, note)
}

#[tauri::command]
fn delete_note(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_note(&state, id)
}

#[tauri::command]
fn list_documents(state: State<'_, AppState>) -> Result<Vec<DocumentRecord>, String> {
    db::list_documents(&state)
}

#[tauri::command]
fn import_document(
    state: State<'_, AppState>,
    source_path: String,
    title: String,
    client_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<i64, String> {
    db::import_document(&state, source_path, title, client_id, project_id)
}

#[tauri::command]
fn delete_document(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_document(&state, id)
}

#[tauri::command]
fn global_search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    db::global_search(&state, query)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::initialize(app.handle()).map_err(std::io::Error::other)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dashboard_stats,
            get_company_profile,
            save_company_profile,
            list_clients,
            create_client,
            delete_client,
            list_projects,
            create_project,
            delete_project,
            list_notes,
            create_note,
            delete_note,
            list_documents,
            import_document,
            delete_document,
            global_search
        ])
        .run(tauri::generate_context!())
        .expect("Virelo nije moguće pokrenuti.");
}
