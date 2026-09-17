use crate::app_paths;
use crate::classifier;
use crate::credential_store;
use crate::database;
use crate::gmail_imap;
use crate::models::{EmailAttachmentInsert, GmailImportSummary, GmailStatus};
use crate::scanner;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::{Datelike, TimeZone, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use url::Url;

const OAUTH_CONFIG_TARGET: &str = "gr.smartlibrary.desktop.gmail.oauth-client";
const TOKEN_TARGET_PREFIX: &str = "gr.smartlibrary.desktop.gmail.token";
const GMAIL_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";
const GMAIL_API_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
const MAX_OAUTH_FILE_BYTES: u64 = 1024 * 1024;
pub(crate) const MAX_ATTACHMENT_BYTES: usize = 32 * 1024 * 1024;
const MAX_GMAIL_QUERY_CHARS: usize = 512;
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthClientConfig {
    client_id: String,
    client_secret: String,
    auth_uri: String,
    token_uri: String,
}

#[derive(Debug, Deserialize)]
struct OAuthClientFile {
    installed: Option<OAuthClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredToken {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    scope: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
    scope: Option<String>,
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailProfile {
    email_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailMessageList {
    #[serde(default)]
    messages: Vec<GmailMessageRef>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailMessageRef {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailMessage {
    id: String,
    #[serde(default)]
    thread_id: String,
    internal_date: Option<String>,
    payload: Option<GmailPart>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct GmailPart {
    part_id: Option<String>,
    #[serde(default)]
    mime_type: String,
    #[serde(default)]
    filename: String,
    #[serde(default)]
    headers: Vec<GmailHeader>,
    body: Option<GmailPartBody>,
    #[serde(default)]
    parts: Vec<GmailPart>,
}

#[derive(Debug, Clone, Deserialize)]
struct GmailHeader {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct GmailPartBody {
    attachment_id: Option<String>,
    size: Option<usize>,
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GmailAttachmentResponse {
    #[serde(default)]
    size: usize,
    data: String,
}

#[derive(Debug, Clone)]
struct AttachmentPart {
    part_id: String,
    filename: String,
    mime_type: String,
    attachment_id: Option<String>,
    inline_data: Option<String>,
    declared_size: usize,
}

pub fn configure_oauth_from_file(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Δεν μπορώ να διαβάσω το αρχείο OAuth: {error}"))?;
    if !metadata.is_file() || metadata.len() > MAX_OAUTH_FILE_BYTES {
        return Err("Το αρχείο OAuth δεν είναι έγκυρο αρχείο JSON.".to_string());
    }

    let bytes =
        fs::read(path).map_err(|error| format!("Δεν μπορώ να διαβάσω το αρχείο OAuth: {error}"))?;
    let parsed: OAuthClientFile = serde_json::from_slice(&bytes)
        .map_err(|_| "Το JSON δεν είναι αρχείο OAuth Desktop app της Google.".to_string())?;
    let config = parsed.installed.ok_or_else(|| {
        "Χρειάζεται OAuth Client τύπου Desktop app, όχι Web application.".to_string()
    })?;
    validate_oauth_config(&config)?;
    let secret = serde_json::to_vec(&config).map_err(|error| error.to_string())?;
    credential_store::set(OAUTH_CONFIG_TARGET, &secret)
}

pub fn status(database_path: &Path) -> Result<GmailStatus, String> {
    let oauth_configured = credential_store::get(OAUTH_CONFIG_TARGET)?.is_some();
    let account = database::active_gmail_account(database_path)?;

    if let Some(account) = account {
        let token_exists = credential_store::get(&token_target(&account.account_email))?.is_some();
        let imap_exists = gmail_imap::credential_exists(&account.account_email)?;
        let imported_attachments =
            database::imported_attachment_count(database_path, &account.account_email)?;
        return Ok(GmailStatus {
            platform_supported: credential_store::is_supported(),
            oauth_configured,
            connected: account.is_connected && (imap_exists || token_exists),
            auth_method: if imap_exists {
                Some("imap".to_string())
            } else if token_exists {
                Some("oauth".to_string())
            } else {
                None
            },
            account_email: Some(account.account_email),
            connected_utc: Some(account.connected_utc),
            last_sync_utc: account.last_sync_utc,
            imported_attachments,
        });
    }

    Ok(GmailStatus {
        platform_supported: credential_store::is_supported(),
        oauth_configured,
        connected: false,
        auth_method: None,
        account_email: None,
        connected_utc: None,
        last_sync_utc: None,
        imported_attachments: 0,
    })
}

pub fn connect_imap(
    database_path: &Path,
    email: &str,
    app_password: &str,
) -> Result<GmailStatus, String> {
    gmail_imap::connect(database_path, email, app_password)?;
    status(database_path)
}

pub async fn connect(app: &AppHandle, database_path: &Path) -> Result<GmailStatus, String> {
    if !credential_store::is_supported() {
        return Err(
            "Η ασφαλής σύνδεση Gmail αυτής της έκδοσης υποστηρίζεται σε Windows και Linux.".to_string(),
        );
    }

    let config = load_oauth_config()?;
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| format!("Δεν άνοιξε το ασφαλές τοπικό callback: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");
    let verifier = random_token(72);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state_value = random_token(40);
    let authorization_url = authorization_url(&config, &redirect_uri, &state_value, &challenge)?;

    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Δεν ρυθμίστηκε το τοπικό callback: {error}"))?;
    app.opener()
        .open_url(authorization_url.as_str(), None::<&str>)
        .map_err(|error| format!("Δεν άνοιξε το πρόγραμμα περιήγησης: {error}"))?;

    let expected_state = state_value.clone();
    let authorization_code =
        tauri::async_runtime::spawn_blocking(move || wait_for_callback(listener, &expected_state))
            .await
            .map_err(|error| format!("Η σύνδεση Gmail διακόπηκε: {error}"))??;

    let client = http_client()?;
    let token = exchange_authorization_code(
        &client,
        &config,
        &authorization_code,
        &redirect_uri,
        &verifier,
    )
    .await?;
    let profile: GmailProfile = gmail_get_json(
        &client,
        &format!("{GMAIL_API_BASE}/profile"),
        &token.access_token,
    )
    .await?;
    let account_email = profile.email_address.trim().to_ascii_lowercase();
    if account_email.is_empty() || !account_email.contains('@') {
        return Err("Η Google δεν επέστρεψε έγκυρη διεύθυνση Gmail.".to_string());
    }

    let previous_account =
        database::active_gmail_account(database_path)?.map(|account| account.account_email);
    save_token(&account_email, &token)?;
    if let Err(error) = database::upsert_gmail_account(database_path, &account_email) {
        if previous_account.as_deref() != Some(account_email.as_str()) {
            let _ = credential_store::delete(&token_target(&account_email));
        }
        return Err(error);
    }
    if let Some(previous_email) = previous_account {
        if previous_email != account_email {
            credential_store::delete(&token_target(&previous_email)).map_err(|error| {
                format!("Ο νέος λογαριασμός συνδέθηκε, αλλά δεν αφαιρέθηκε το παλιό token: {error}")
            })?;
        }
    }
    status(database_path)
}

pub fn disconnect(database_path: &Path) -> Result<GmailStatus, String> {
    if let Some(account) = database::active_gmail_account(database_path)? {
        credential_store::delete(&token_target(&account.account_email))?;
        gmail_imap::delete_credentials(&account.account_email)?;
        database::disconnect_gmail_account(database_path, &account.account_email)?;
    }
    status(database_path)
}

pub async fn import_attachments(
    database_path: &Path,
    query: &str,
    max_messages: usize,
) -> Result<GmailImportSummary, String> {
    let account = database::active_gmail_account(database_path)?
        .ok_or_else(|| "Σύνδεσε πρώτα έναν λογαριασμό Gmail.".to_string())?;
    if gmail_imap::credential_exists(&account.account_email)? {
        let database_path = database_path.to_path_buf();
        let account_email = account.account_email;
        let query = query.to_string();
        return tauri::async_runtime::spawn_blocking(move || {
            gmail_imap::import_attachments(&database_path, &account_email, &query, max_messages)
        })
        .await
        .map_err(|error| format!("Η λήψη Gmail διακόπηκε: {error}"))?;
    }

    let config = load_oauth_config()?;
    let client = http_client()?;
    let mut token = load_token(&account.account_email)?;
    if token.expires_at <= Utc::now().timestamp() + 90 {
        token = refresh_access_token(&client, &config, &token).await?;
        save_token(&account.account_email, &token)?;
    }

    let connection = database::open(database_path)?;
    let enabled_extensions = database::enabled_extension_set(&connection)?;
    drop(connection);

    let query = if query.trim().is_empty() {
        "has:attachment"
    } else {
        query.trim()
    };
    if query.chars().count() > MAX_GMAIL_QUERY_CHARS {
        return Err(format!(
            "Το Gmail query δεν μπορεί να ξεπερνά τους {MAX_GMAIL_QUERY_CHARS} χαρακτήρες."
        ));
    }
    let message_limit = max_messages.clamp(1, 500);
    let account_root = gmail_inbox_root(&account.account_email);
    fs::create_dir_all(&account_root)
        .map_err(|error| format!("Δεν δημιουργήθηκε το Gmail Inbox: {error}"))?;

    let mut summary = GmailImportSummary {
        account_email: account.account_email.clone(),
        messages_scanned: 0,
        attachments_found: 0,
        imported: 0,
        duplicates: 0,
        skipped: 0,
        failed: 0,
        indexed_after_sync: 0,
        source_root: account_root.to_string_lossy().to_string(),
    };
    let mut page_token: Option<String> = None;

    while summary.messages_scanned < message_limit {
        let remaining = message_limit - summary.messages_scanned;
        let mut url =
            Url::parse(&format!("{GMAIL_API_BASE}/messages")).map_err(|error| error.to_string())?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("q", query);
            pairs.append_pair("includeSpamTrash", "false");
            pairs.append_pair("maxResults", &remaining.min(100).to_string());
            if let Some(value) = page_token.as_deref() {
                pairs.append_pair("pageToken", value);
            }
        }
        let page: GmailMessageList =
            gmail_get_json(&client, url.as_str(), &token.access_token).await?;
        if page.messages.is_empty() {
            break;
        }

        for message_ref in page.messages {
            if summary.messages_scanned >= message_limit {
                break;
            }
            summary.messages_scanned += 1;
            let message_url = format!("{GMAIL_API_BASE}/messages/{}?format=full", message_ref.id);
            let message: GmailMessage =
                match gmail_get_json(&client, &message_url, &token.access_token).await {
                    Ok(message) => message,
                    Err(_) => {
                        summary.failed += 1;
                        continue;
                    }
                };
            process_message(
                &client,
                &token.access_token,
                database_path,
                &account.account_email,
                &account_root,
                &enabled_extensions,
                &message,
                &mut summary,
            )
            .await;
        }

        page_token = page.next_page_token;
        if page_token.is_none() {
            break;
        }
    }

    database::finish_gmail_sync(database_path, &account.account_email)?;
    let scan_root = account_root.clone();
    let scan_database = database_path.to_path_buf();
    let scan = tauri::async_runtime::spawn_blocking(move || {
        scanner::scan_directory(&scan_root, &scan_database)
    })
    .await
    .map_err(|error| format!("Η ενημέρωση του index διακόπηκε: {error}"))??;
    summary.indexed_after_sync = scan.indexed;

    Ok(summary)
}

async fn process_message(
    client: &Client,
    access_token: &str,
    database_path: &Path,
    account_email: &str,
    account_root: &Path,
    enabled_extensions: &HashSet<String>,
    message: &GmailMessage,
    summary: &mut GmailImportSummary,
) {
    let Some(payload) = message.payload.as_ref() else {
        summary.failed += 1;
        return;
    };
    let sender = header_value(payload, "From");
    let subject = header_value(payload, "Subject");
    let header_date = header_value(payload, "Date");
    let message_time = message_timestamp(message.internal_date.as_deref());
    let message_date = if header_date.is_empty() {
        message_time.to_rfc3339()
    } else {
        header_date
    };
    let mut attachments = Vec::new();
    collect_attachment_parts(payload, "0", &mut attachments);
    summary.attachments_found += attachments.len();

    for attachment in attachments {
        let result = process_attachment(
            client,
            access_token,
            database_path,
            account_email,
            account_root,
            enabled_extensions,
            message,
            &sender,
            &subject,
            &message_date,
            &message_time,
            &attachment,
        )
        .await;
        match result {
            Ok(AttachmentOutcome::Imported) => summary.imported += 1,
            Ok(AttachmentOutcome::Duplicate) => summary.duplicates += 1,
            Ok(AttachmentOutcome::Skipped) => summary.skipped += 1,
            Err(_) => summary.failed += 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AttachmentOutcome {
    Imported,
    Duplicate,
    Skipped,
}

#[allow(clippy::too_many_arguments)]
async fn process_attachment(
    client: &Client,
    access_token: &str,
    database_path: &Path,
    account_email: &str,
    account_root: &Path,
    enabled_extensions: &HashSet<String>,
    message: &GmailMessage,
    sender: &str,
    subject: &str,
    message_date: &str,
    message_time: &chrono::DateTime<Utc>,
    attachment: &AttachmentPart,
) -> Result<AttachmentOutcome, String> {
    if database::email_attachment_exists(
        database_path,
        account_email,
        &message.id,
        &attachment.part_id,
    )? {
        return Ok(AttachmentOutcome::Skipped);
    }

    let safe_filename = sanitize_filename(&attachment.filename);
    let extension = Path::new(&safe_filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !classifier::is_indexable(&extension) || !enabled_extensions.contains(&extension) {
        return Ok(AttachmentOutcome::Skipped);
    }
    if attachment.declared_size > MAX_ATTACHMENT_BYTES {
        return Ok(AttachmentOutcome::Skipped);
    }

    let bytes = if let Some(data) = attachment.inline_data.as_deref() {
        decode_base64url(data)?
    } else if let Some(attachment_id) = attachment.attachment_id.as_deref() {
        let url = format!(
            "{GMAIL_API_BASE}/messages/{}/attachments/{attachment_id}",
            message.id
        );
        let response: GmailAttachmentResponse = gmail_get_json(client, &url, access_token).await?;
        if response.size > MAX_ATTACHMENT_BYTES {
            return Ok(AttachmentOutcome::Skipped);
        }
        decode_base64url(&response.data)?
    } else {
        return Err("Το Gmail attachment δεν είχε αναγνώσιμα δεδομένα.".to_string());
    };
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Ok(AttachmentOutcome::Skipped);
    }

    persist_attachment(
        database_path,
        account_email,
        account_root,
        enabled_extensions,
        &message.id,
        &message.thread_id,
        &attachment.part_id,
        sender,
        subject,
        message_date,
        message_time,
        &attachment.filename,
        &attachment.mime_type,
        &bytes,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn persist_attachment(
    database_path: &Path,
    account_email: &str,
    account_root: &Path,
    enabled_extensions: &HashSet<String>,
    message_id: &str,
    thread_id: &str,
    part_id: &str,
    sender: &str,
    subject: &str,
    message_date: &str,
    message_time: &chrono::DateTime<Utc>,
    filename: &str,
    mime_type: &str,
    bytes: &[u8],
) -> Result<AttachmentOutcome, String> {
    if database::email_attachment_exists(database_path, account_email, message_id, part_id)? {
        return Ok(AttachmentOutcome::Skipped);
    }

    let safe_filename = sanitize_filename(filename);
    let extension = Path::new(&safe_filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !classifier::is_indexable(&extension) || !enabled_extensions.contains(&extension) {
        return Ok(AttachmentOutcome::Skipped);
    }
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Ok(AttachmentOutcome::Skipped);
    }

    let sha256 = hex::encode(Sha256::digest(bytes));
    if let Some(existing_path) = database::existing_path_for_hash(database_path, &sha256)? {
        if Path::new(&existing_path).is_file() {
            database::record_email_attachment(
                database_path,
                &attachment_record(
                    account_email,
                    message_id,
                    thread_id,
                    part_id,
                    sender,
                    subject,
                    message_date,
                    filename,
                    mime_type,
                    &existing_path,
                    bytes.len(),
                    &sha256,
                    "duplicate",
                ),
            )?;
            return Ok(AttachmentOutcome::Duplicate);
        }
    }

    let destination_directory = account_root
        .join(format!("{:04}", message_time.year()))
        .join(format!("{:02}", message_time.month()));
    fs::create_dir_all(&destination_directory)
        .map_err(|error| format!("Δεν δημιουργήθηκε φάκελος Gmail Inbox: {error}"))?;
    let short_message = safe_identifier(message_id, 14);
    let short_part = safe_identifier(part_id, 12);
    let destination_name = format!(
        "{}_{}_{}_{}",
        message_time.format("%Y-%m-%d"),
        short_message,
        short_part,
        safe_filename
    );
    let destination = unique_destination(&destination_directory, &destination_name);
    write_atomic(&destination, bytes)?;
    let stored_path = destination.to_string_lossy().to_string();
    if let Err(error) = database::record_email_attachment(
        database_path,
        &attachment_record(
            account_email,
            message_id,
            thread_id,
            part_id,
            sender,
            subject,
            message_date,
            filename,
            mime_type,
            &stored_path,
            bytes.len(),
            &sha256,
            "imported",
        ),
    ) {
        let _ = fs::remove_file(&destination);
        return Err(error);
    }
    Ok(AttachmentOutcome::Imported)
}

#[allow(clippy::too_many_arguments)]
fn attachment_record(
    account_email: &str,
    message_id: &str,
    thread_id: &str,
    part_id: &str,
    sender: &str,
    subject: &str,
    message_date: &str,
    original_filename: &str,
    mime_type: &str,
    stored_path: &str,
    size_bytes: usize,
    sha256: &str,
    import_status: &str,
) -> EmailAttachmentInsert {
    EmailAttachmentInsert {
        provider: "gmail".to_string(),
        account_email: account_email.to_string(),
        message_id: message_id.to_string(),
        thread_id: thread_id.to_string(),
        part_id: part_id.to_string(),
        sender: sender.to_string(),
        subject: subject.to_string(),
        message_date: message_date.to_string(),
        original_filename: original_filename.to_string(),
        stored_path: stored_path.to_string(),
        mime_type: mime_type.to_string(),
        size_bytes: size_bytes.min(i64::MAX as usize) as i64,
        sha256: sha256.to_string(),
        import_status: import_status.to_string(),
    }
}

fn validate_oauth_config(config: &OAuthClientConfig) -> Result<(), String> {
    if !config.client_id.ends_with(".apps.googleusercontent.com") {
        return Err("Το OAuth Client ID δεν είναι έγκυρο Google Desktop Client.".to_string());
    }
    validate_google_endpoint(
        &config.auth_uri,
        "accounts.google.com",
        &["/o/oauth2/auth", "/o/oauth2/v2/auth"],
    )?;
    validate_google_endpoint(&config.token_uri, "oauth2.googleapis.com", &["/token"])?;
    Ok(())
}

fn validate_google_endpoint(value: &str, host: &str, paths: &[&str]) -> Result<(), String> {
    let url = Url::parse(value).map_err(|_| "Μη έγκυρο Google OAuth endpoint.".to_string())?;
    if url.scheme() != "https"
        || url.host_str() != Some(host)
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !paths.contains(&url.path())
    {
        return Err("Το OAuth JSON περιέχει μη αναμενόμενο endpoint.".to_string());
    }
    Ok(())
}

fn validate_token_scope(scope: &str) -> Result<(), String> {
    let scopes: HashSet<&str> = scope.split_whitespace().collect();
    if scopes.len() == 1 && scopes.contains(GMAIL_READONLY_SCOPE) {
        Ok(())
    } else {
        Err("Η Google επέστρεψε διαφορετική άδεια από το gmail.readonly.".to_string())
    }
}

fn authorization_url(
    config: &OAuthClientConfig,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> Result<Url, String> {
    let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|error| error.to_string())?;
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", GMAIL_READONLY_SCOPE)
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(url)
}

fn wait_for_callback(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    let started = Instant::now();
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(connection) => break connection,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if started.elapsed() >= CALLBACK_TIMEOUT {
                    return Err("Η εξουσιοδότηση Gmail έληξε μετά από 3 λεπτά.".to_string());
                }
                std::thread::sleep(Duration::from_millis(160));
            }
            Err(error) => return Err(format!("Απέτυχε το τοπικό OAuth callback: {error}")),
        }
    };
    stream
        .set_nonblocking(false)
        .map_err(|error| format!("Δεν ρυθμίστηκε το OAuth callback: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut buffer = Vec::with_capacity(2 * 1024);
    let mut header_complete = false;
    loop {
        let mut chunk = [0u8; 2 * 1024];
        let bytes_read = stream
            .read(&mut chunk)
            .map_err(|error| format!("Δεν διαβάστηκε η απάντηση Google: {error}"))?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            header_complete = true;
            break;
        }
        if buffer.len() >= 16 * 1024 {
            return Err("Η απάντηση OAuth ήταν μεγαλύτερη από το ασφαλές όριο.".to_string());
        }
    }
    if !header_complete {
        return Err("Η απάντηση OAuth ήταν ελλιπής.".to_string());
    }
    let request = String::from_utf8_lossy(&buffer);
    let mut request_parts = request
        .lines()
        .next()
        .map(str::split_whitespace)
        .ok_or_else(|| "Μη έγκυρη απάντηση OAuth.".to_string())?;
    if request_parts.next() != Some("GET") {
        return Err("Το OAuth callback δεν ήταν ασφαλές HTTP GET.".to_string());
    }
    let request_target = request_parts
        .next()
        .ok_or_else(|| "Μη έγκυρη απάντηση OAuth.".to_string())?;
    let callback_url = Url::parse(&format!("http://127.0.0.1{request_target}"))
        .map_err(|_| "Μη έγκυρη διεύθυνση OAuth callback.".to_string())?;
    let query: std::collections::HashMap<String, String> =
        callback_url.query_pairs().into_owned().collect();
    let state_matches = query
        .get("state")
        .is_some_and(|value| value == expected_state);
    let error = query.get("error").cloned();
    let code = query.get("code").cloned();
    let success = state_matches && error.is_none() && code.is_some();
    let body = if success {
        "<!doctype html><meta charset=\"utf-8\"><title>Smart Library</title><body style=\"font-family:Segoe UI,sans-serif;background:#071019;color:#eef6f9;padding:48px\"><h1>Το Gmail συνδέθηκε</h1><p>Μπορείς να κλείσεις αυτή την καρτέλα και να επιστρέψεις στο Smart Library.</p></body>"
    } else {
        "<!doctype html><meta charset=\"utf-8\"><title>Smart Library</title><body style=\"font-family:Segoe UI,sans-serif;background:#071019;color:#eef6f9;padding:48px\"><h1>Η σύνδεση δεν ολοκληρώθηκε</h1><p>Επέστρεψε στο Smart Library για περισσότερες πληροφορίες.</p></body>"
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());

    if !state_matches {
        return Err("Η απάντηση OAuth απέτυχε στον έλεγχο ασφαλείας state.".to_string());
    }
    if let Some(error) = error {
        return Err(format!("Η Google δεν έδωσε άδεια σύνδεσης: {error}"));
    }
    code.ok_or_else(|| "Η Google δεν επέστρεψε authorization code.".to_string())
}

async fn exchange_authorization_code(
    client: &Client,
    config: &OAuthClientConfig,
    code: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<StoredToken, String> {
    let mut form = vec![
        ("client_id", config.client_id.as_str()),
        ("code", code),
        ("code_verifier", verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];
    if !config.client_secret.is_empty() {
        form.push(("client_secret", config.client_secret.as_str()));
    }
    let response = client
        .post(&config.token_uri)
        .form(&form)
        .send()
        .await
        .map_err(|error| format!("Δεν απάντησε η υπηρεσία OAuth της Google: {error}"))?;
    let token_response: OAuthTokenResponse = parse_json_response(response, "Google OAuth").await?;
    if let Some(scope) = token_response.scope.as_deref() {
        validate_token_scope(scope)?;
    }
    let refresh_token = token_response.refresh_token.ok_or_else(|| {
        "Η Google δεν επέστρεψε refresh token. Αφαίρεσε την προηγούμενη άδεια και σύνδεσε ξανά τον λογαριασμό."
            .to_string()
    })?;
    Ok(StoredToken {
        access_token: token_response.access_token,
        refresh_token,
        expires_at: Utc::now().timestamp() + token_response.expires_in.max(60),
        scope: token_response
            .scope
            .unwrap_or_else(|| GMAIL_READONLY_SCOPE.to_string()),
        token_type: token_response
            .token_type
            .unwrap_or_else(|| "Bearer".to_string()),
    })
}

async fn refresh_access_token(
    client: &Client,
    config: &OAuthClientConfig,
    current: &StoredToken,
) -> Result<StoredToken, String> {
    let mut form = vec![
        ("client_id", config.client_id.as_str()),
        ("refresh_token", current.refresh_token.as_str()),
        ("grant_type", "refresh_token"),
    ];
    if !config.client_secret.is_empty() {
        form.push(("client_secret", config.client_secret.as_str()));
    }
    let response = client
        .post(&config.token_uri)
        .form(&form)
        .send()
        .await
        .map_err(|error| format!("Δεν ανανεώθηκε η σύνδεση Gmail: {error}"))?;
    let refreshed: OAuthTokenResponse =
        parse_json_response(response, "Google OAuth refresh").await?;
    if let Some(scope) = refreshed.scope.as_deref() {
        validate_token_scope(scope)?;
    }
    Ok(StoredToken {
        access_token: refreshed.access_token,
        refresh_token: refreshed
            .refresh_token
            .unwrap_or_else(|| current.refresh_token.clone()),
        expires_at: Utc::now().timestamp() + refreshed.expires_in.max(60),
        scope: refreshed.scope.unwrap_or_else(|| current.scope.clone()),
        token_type: refreshed
            .token_type
            .unwrap_or_else(|| current.token_type.clone()),
    })
}

async fn gmail_get_json<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    access_token: &str,
) -> Result<T, String> {
    let response = client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| format!("Δεν απάντησε το Gmail API: {error}"))?;
    parse_json_response(response, "Gmail API").await
}

async fn parse_json_response<T: DeserializeOwned>(
    response: Response,
    service: &str,
) -> Result<T, String> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("Μη αναγνώσιμη απάντηση από {service}: {error}"))?;
    if !status.is_success() {
        let safe_message = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| {
                value
                    .pointer("/error/message")
                    .and_then(|item| item.as_str())
                    .or_else(|| {
                        value
                            .get("error_description")
                            .and_then(|item| item.as_str())
                    })
                    .or_else(|| value.get("error").and_then(|item| item.as_str()))
                    .map(str::to_string)
            })
            .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
        return Err(format!(
            "{service}: {}",
            safe_message.chars().take(500).collect::<String>()
        ));
    }
    serde_json::from_str(&text)
        .map_err(|error| format!("Μη αναμενόμενη απάντηση από {service}: {error}"))
}

fn load_oauth_config() -> Result<OAuthClientConfig, String> {
    let bytes = credential_store::get(OAUTH_CONFIG_TARGET)?
        .ok_or_else(|| "Επίλεξε πρώτα το OAuth JSON του Gmail.".to_string())?;
    let config: OAuthClientConfig = serde_json::from_slice(&bytes)
        .map_err(|_| "Η αποθηκευμένη ρύθμιση OAuth δεν διαβάζεται.".to_string())?;
    validate_oauth_config(&config)?;
    Ok(config)
}

fn save_token(account_email: &str, token: &StoredToken) -> Result<(), String> {
    let bytes = serde_json::to_vec(token).map_err(|error| error.to_string())?;
    credential_store::set(&token_target(account_email), &bytes)
}

fn load_token(account_email: &str) -> Result<StoredToken, String> {
    let bytes = credential_store::get(&token_target(account_email))?.ok_or_else(|| {
        "Η ασφαλής σύνδεση Gmail δεν βρέθηκε. Σύνδεσε ξανά τον λογαριασμό.".to_string()
    })?;
    let token: StoredToken = serde_json::from_slice(&bytes).map_err(|_| {
        "Το αποθηκευμένο Gmail token δεν διαβάζεται. Σύνδεσε ξανά τον λογαριασμό.".to_string()
    })?;
    validate_token_scope(&token.scope)?;
    Ok(token)
}

fn token_target(account_email: &str) -> String {
    format!(
        "{TOKEN_TARGET_PREFIX}.{}",
        account_email.trim().to_ascii_lowercase()
    )
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(45))
        .user_agent("Smart-Library/0.3.4")
        .build()
        .map_err(|error| format!("Δεν δημιουργήθηκε ασφαλής HTTP client: {error}"))
}

fn random_token(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn header_value(payload: &GmailPart, name: &str) -> String {
    payload
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.trim().to_string())
        .unwrap_or_default()
}

fn collect_attachment_parts(part: &GmailPart, fallback_id: &str, output: &mut Vec<AttachmentPart>) {
    let part_id = part
        .part_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback_id)
        .to_string();
    if !part.filename.trim().is_empty() {
        let body = part.body.clone().unwrap_or_default();
        if body.attachment_id.is_some() || body.data.is_some() {
            output.push(AttachmentPart {
                part_id: part_id.clone(),
                filename: part.filename.clone(),
                mime_type: if part.mime_type.is_empty() {
                    "application/octet-stream".to_string()
                } else {
                    part.mime_type.clone()
                },
                attachment_id: body.attachment_id,
                inline_data: body.data,
                declared_size: body.size.unwrap_or(0),
            });
        }
    }
    for (index, child) in part.parts.iter().enumerate() {
        collect_attachment_parts(child, &format!("{part_id}.{index}"), output);
    }
}

fn decode_base64url(value: &str) -> Result<Vec<u8>, String> {
    URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .map_err(|_| "Το Gmail επέστρεψε μη έγκυρα δεδομένα attachment.".to_string())
}

fn message_timestamp(value: Option<&str>) -> chrono::DateTime<Utc> {
    value
        .and_then(|item| item.parse::<i64>().ok())
        .and_then(|milliseconds| Utc.timestamp_millis_opt(milliseconds).single())
        .unwrap_or_else(Utc::now)
}

pub(crate) fn gmail_inbox_root(account_email: &str) -> PathBuf {
    app_paths::documents_dir()
        .join("Smart Library")
        .join("Email Inbox")
        .join("Gmail")
        .join(sanitize_component(account_email))
}

fn sanitize_component(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|character: char| {
            if character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
            {
                '_'
            } else {
                character
            }
        })
        .collect();
    let cleaned = cleaned
        .trim()
        .trim_end_matches(|character: char| character == '.' || character == ' ');
    if cleaned.is_empty() {
        "gmail-account".to_string()
    } else {
        cleaned.chars().take(120).collect()
    }
}

fn sanitize_filename(value: &str) -> String {
    let basename = value
        .rsplit(|character| character == '/' || character == '\\')
        .next()
        .unwrap_or("attachment")
        .trim();
    let mut cleaned = sanitize_component(basename);
    if cleaned.len() > 180 {
        let extension = Path::new(&cleaned)
            .extension()
            .and_then(|item| item.to_str())
            .unwrap_or("")
            .chars()
            .take(16)
            .collect::<String>();
        let stem_limit = 160usize.saturating_sub(extension.len());
        let stem: String = Path::new(&cleaned)
            .file_stem()
            .and_then(|item| item.to_str())
            .unwrap_or("attachment")
            .chars()
            .take(stem_limit)
            .collect();
        cleaned = if extension.is_empty() {
            stem
        } else {
            format!("{stem}.{extension}")
        };
    }

    let reserved_stem = Path::new(&cleaned)
        .file_stem()
        .and_then(|item| item.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    let is_reserved = matches!(
        reserved_stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );
    if is_reserved {
        format!("_{cleaned}")
    } else if cleaned.is_empty() {
        "attachment".to_string()
    } else {
        cleaned
    }
}

fn safe_identifier(value: &str, maximum: usize) -> String {
    let cleaned: String = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(maximum)
        .collect();
    if cleaned.is_empty() {
        "part".to_string()
    } else {
        cleaned
    }
}

fn unique_destination(directory: &Path, file_name: &str) -> PathBuf {
    let initial = directory.join(file_name);
    if !initial.exists() {
        return initial;
    }
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|item| item.to_str())
        .unwrap_or("attachment");
    let extension = path
        .extension()
        .and_then(|item| item.to_str())
        .unwrap_or("");
    for index in 2..10_000 {
        let candidate_name = if extension.is_empty() {
            format!("{stem} ({index})")
        } else {
            format!("{stem} ({index}).{extension}")
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!("{}_{}", random_token(16), file_name))
}

fn write_atomic(destination: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| "Μη έγκυρος φάκελος Gmail Inbox.".to_string())?;
    let temporary = parent.join(format!(".smart-library-{}.download", random_token(20)));
    fs::write(&temporary, bytes)
        .map_err(|error| format!("Δεν γράφτηκε το Gmail attachment: {error}"))?;
    if let Err(error) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "Δεν ολοκληρώθηκε η ασφαλής εγγραφή attachment: {error}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_google_desktop_oauth_config() {
        let config = OAuthClientConfig {
            client_id: "123.apps.googleusercontent.com".to_string(),
            client_secret: "secret".to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
        };
        assert!(validate_oauth_config(&config).is_ok());
    }

    #[test]
    fn rejects_non_google_token_endpoint() {
        let config = OAuthClientConfig {
            client_id: "123.apps.googleusercontent.com".to_string(),
            client_secret: "secret".to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://example.com/token".to_string(),
        };
        assert!(validate_oauth_config(&config).is_err());
    }

    #[test]
    fn accepts_only_the_readonly_gmail_scope() {
        assert!(validate_token_scope(GMAIL_READONLY_SCOPE).is_ok());
        assert!(validate_token_scope("https://mail.google.com/").is_err());
        assert!(validate_token_scope(&format!(
            "{GMAIL_READONLY_SCOPE} https://www.googleapis.com/auth/gmail.modify"
        ))
        .is_err());
    }

    #[test]
    fn sanitizes_windows_file_names() {
        assert_eq!(sanitize_filename("../../CON.pdf"), "_CON.pdf");
        assert_eq!(sanitize_filename("invoice:2026?.pdf"), "invoice_2026_.pdf");
    }

    #[test]
    fn finds_nested_attachment_parts() {
        let payload = GmailPart {
            parts: vec![GmailPart {
                part_id: Some("1".to_string()),
                mime_type: "application/pdf".to_string(),
                filename: "invoice.pdf".to_string(),
                body: Some(GmailPartBody {
                    attachment_id: Some("attachment-1".to_string()),
                    size: Some(42),
                    data: None,
                }),
                ..GmailPart::default()
            }],
            ..GmailPart::default()
        };
        let mut attachments = Vec::new();
        collect_attachment_parts(&payload, "0", &mut attachments);
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "invoice.pdf");
        assert_eq!(attachments[0].part_id, "1");
    }
}
