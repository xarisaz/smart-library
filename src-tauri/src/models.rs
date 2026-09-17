use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct FileInsert {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub modified_utc: String,
    pub sha256: Option<String>,
    pub category: String,
    pub subcategory: String,
    pub confidence: f64,
    pub proposed_name: String,
    pub proposed_relative_path: String,
    pub source_root: String,
    pub extracted_text: String,
    pub extraction_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub extension: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub modified_utc: String,
    pub sha256: Option<String>,
    pub category: String,
    pub subcategory: String,
    pub confidence: f64,
    pub proposed_name: String,
    pub proposed_relative_path: String,
    pub source_root: String,
    pub review_status: String,
    pub extraction_status: String,
    pub text_preview: String,
    pub is_duplicate: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub source_root: String,
    pub indexed: usize,
    pub ignored: usize,
    pub unreadable: usize,
    pub duplicates: usize,
    pub text_extracted: usize,
    pub needs_ocr: usize,
    pub extraction_failed: usize,
    pub reached_limit: bool,
    pub elapsed_ms: i64,
    pub categories: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub safe_mode: bool,
    pub indexed_files: i64,
    pub pending_review: i64,
    pub duplicates: i64,
    pub source_count: i64,
    pub smart_library_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPayload {
    pub kind: String,
    pub mime_type: String,
    pub text: Option<String>,
    pub data_url: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFilterSettings {
    pub supported_extensions: Vec<String>,
    pub enabled_extensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GmailAccountRecord {
    pub account_email: String,
    pub connected_utc: String,
    pub last_sync_utc: Option<String>,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub struct EmailAttachmentInsert {
    pub provider: String,
    pub account_email: String,
    pub message_id: String,
    pub thread_id: String,
    pub part_id: String,
    pub sender: String,
    pub subject: String,
    pub message_date: String,
    pub original_filename: String,
    pub stored_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub import_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailStatus {
    pub platform_supported: bool,
    pub oauth_configured: bool,
    pub connected: bool,
    pub auth_method: Option<String>,
    pub account_email: Option<String>,
    pub connected_utc: Option<String>,
    pub last_sync_utc: Option<String>,
    pub imported_attachments: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailImportSummary {
    pub account_email: String,
    pub messages_scanned: usize,
    pub attachments_found: usize,
    pub imported: usize,
    pub duplicates: usize,
    pub skipped: usize,
    pub failed: usize,
    pub indexed_after_sync: usize,
    pub source_root: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailProvenance {
    pub provider: String,
    pub account_email: String,
    pub message_id: String,
    pub sender: String,
    pub subject: String,
    pub message_date: String,
    pub original_filename: String,
    pub import_status: String,
}
