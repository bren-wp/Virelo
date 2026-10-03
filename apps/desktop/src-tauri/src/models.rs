use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub clients: i64,
    pub active_projects: i64,
    pub notes: i64,
    pub documents: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub name: String,
    pub tax_id: String,
    pub registration_id: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub email: String,
    pub phone: String,
    pub website: String,
    pub iban: String,
    pub bic: String,
    pub bank_name: String,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    pub tax_id: String,
    pub registration_id: String,
    pub email: String,
    pub phone: String,
    pub website: String,
    pub address: String,
    pub city: String,
    pub country: String,
    pub status: String,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientContact {
    #[serde(default)]
    pub id: i64,
    pub client_id: i64,
    #[serde(default)]
    pub client_name: Option<String>,
    pub name: String,
    pub role: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccount {
    #[serde(default)]
    pub id: i64,
    pub label: String,
    pub iban: String,
    pub bic: String,
    pub bank_name: String,
    pub currency: String,
    pub is_default: bool,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractRecord {
    #[serde(default)]
    pub id: i64,
    pub number: String,
    pub title: String,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub client_name: Option<String>,
    pub project_id: Option<i64>,
    #[serde(default)]
    pub project_name: Option<String>,
    pub document_id: Option<i64>,
    #[serde(default)]
    pub document_title: Option<String>,
    pub status: String,
    pub signed_date: String,
    pub start_date: String,
    pub end_date: String,
    pub value_cents: i64,
    pub currency: String,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    #[serde(default)]
    pub id: i64,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub client_name: Option<String>,
    pub name: String,
    pub status: String,
    pub priority: String,
    pub due_date: String,
    pub value_cents: i64,
    pub currency: String,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    #[serde(default)]
    pub id: i64,
    pub title: String,
    pub body_markdown: String,
    pub client_id: Option<i64>,
    pub project_id: Option<i64>,
    pub tags: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: i64,
    pub title: String,
    pub file_name: String,
    pub file_path: String,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub client_name: Option<String>,
    pub project_id: Option<i64>,
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default = "default_document_category")]
    pub category: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub description: String,
    pub created_at: String,
}

fn default_document_category() -> String {
    "Ostalo".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRecord {
    #[serde(default)]
    pub id: i64,
    pub title: String,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub client_name: Option<String>,
    pub project_id: Option<i64>,
    #[serde(default)]
    pub project_name: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: String,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityRecord {
    #[serde(default)]
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub details: String,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub client_name: Option<String>,
    pub project_id: Option<i64>,
    #[serde(default)]
    pub project_name: Option<String>,
    pub happened_at: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinanceRecord {
    #[serde(default)]
    pub id: i64,
    pub kind: String,
    pub number: String,
    pub title: String,
    pub client_id: Option<i64>,
    #[serde(default)]
    pub client_name: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub issue_date: String,
    pub due_date: String,
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub kind: String,
    pub id: i64,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub version: String,
}
