mod db;
mod extras;
mod models;

use tauri::{Manager, State};

use db::AppState;
use models::{
    ActivityRecord, AppInfo, Client, ClientContact, CompanyProfile, DashboardStats, DocumentRecord,
    FinanceRecord, Note, Project, SearchHit, TaskRecord,
};

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
fn update_client(state: State<'_, AppState>, client: Client) -> Result<(), String> {
    extras::update_client(&state, client)
}

#[tauri::command]
fn delete_client(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_client(&state, id)
}

#[tauri::command]
fn list_client_contacts(state: State<'_, AppState>) -> Result<Vec<ClientContact>, String> {
    extras::list_client_contacts(&state)
}

#[tauri::command]
fn create_client_contact(
    state: State<'_, AppState>,
    contact: ClientContact,
) -> Result<i64, String> {
    extras::create_client_contact(&state, contact)
}

#[tauri::command]
fn update_client_contact(state: State<'_, AppState>, contact: ClientContact) -> Result<(), String> {
    extras::update_client_contact(&state, contact)
}

#[tauri::command]
fn delete_client_contact(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    extras::delete_client_contact(&state, id)
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
fn update_project(state: State<'_, AppState>, project: Project) -> Result<(), String> {
    extras::update_project(&state, project)
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
fn update_note(state: State<'_, AppState>, note: Note) -> Result<(), String> {
    extras::update_note(&state, note)
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
fn update_document(
    state: State<'_, AppState>,
    id: i64,
    title: String,
    client_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<(), String> {
    extras::update_document(&state, id, title, client_id, project_id)
}

#[tauri::command]
fn delete_document(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_document(&state, id)
}

#[tauri::command]
fn list_tasks(state: State<'_, AppState>) -> Result<Vec<TaskRecord>, String> {
    extras::list_tasks(&state)
}

#[tauri::command]
fn create_task(state: State<'_, AppState>, task: TaskRecord) -> Result<i64, String> {
    extras::create_task(&state, task)
}

#[tauri::command]
fn update_task(state: State<'_, AppState>, task: TaskRecord) -> Result<(), String> {
    extras::update_task(&state, task)
}

#[tauri::command]
fn delete_task(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    extras::delete_task(&state, id)
}

#[tauri::command]
fn list_activities(state: State<'_, AppState>) -> Result<Vec<ActivityRecord>, String> {
    extras::list_activities(&state)
}

#[tauri::command]
fn create_activity(state: State<'_, AppState>, activity: ActivityRecord) -> Result<i64, String> {
    extras::create_activity(&state, activity)
}

#[tauri::command]
fn delete_activity(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    extras::delete_activity(&state, id)
}

#[tauri::command]
fn list_finance_records(state: State<'_, AppState>) -> Result<Vec<FinanceRecord>, String> {
    extras::list_finance_records(&state)
}

#[tauri::command]
fn create_finance_record(state: State<'_, AppState>, record: FinanceRecord) -> Result<i64, String> {
    extras::create_finance_record(&state, record)
}

#[tauri::command]
fn update_finance_record(state: State<'_, AppState>, record: FinanceRecord) -> Result<(), String> {
    extras::update_finance_record(&state, record)
}

#[tauri::command]
fn delete_finance_record(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    extras::delete_finance_record(&state, id)
}

#[tauri::command]
fn app_info(state: State<'_, AppState>) -> AppInfo {
    extras::app_info(&state)
}

#[tauri::command]
fn export_workspace_json(state: State<'_, AppState>, destination: String) -> Result<(), String> {
    extras::export_workspace_json(&state, destination)
}

#[tauri::command]
fn export_workspace_csv(state: State<'_, AppState>, destination: String) -> Result<(), String> {
    extras::export_workspace_csv(&state, destination)
}

#[tauri::command]
fn export_workspace_markdown(
    state: State<'_, AppState>,
    destination: String,
) -> Result<(), String> {
    extras::export_workspace_markdown(&state, destination)
}

#[tauri::command]
fn export_workspace_html(state: State<'_, AppState>, destination: String) -> Result<(), String> {
    extras::export_workspace_html(&state, destination)
}

#[tauri::command]
fn backup_database(state: State<'_, AppState>, destination: String) -> Result<(), String> {
    extras::backup_database(&state, destination)
}

#[tauri::command]
fn global_search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    extras::global_search(&state, query)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
            update_client,
            delete_client,
            list_client_contacts,
            create_client_contact,
            update_client_contact,
            delete_client_contact,
            list_projects,
            create_project,
            update_project,
            delete_project,
            list_notes,
            create_note,
            update_note,
            delete_note,
            list_documents,
            import_document,
            update_document,
            delete_document,
            list_tasks,
            create_task,
            update_task,
            delete_task,
            list_activities,
            create_activity,
            delete_activity,
            list_finance_records,
            create_finance_record,
            update_finance_record,
            delete_finance_record,
            app_info,
            export_workspace_json,
            export_workspace_csv,
            export_workspace_markdown,
            export_workspace_html,
            backup_database,
            global_search
        ])
        .run(tauri::generate_context!())
        .expect("Virelo nije moguće pokrenuti.");
}
