use crate::credential_store;
use crate::database;
use crate::gmail::{gmail_inbox_root, persist_attachment, AttachmentOutcome, MAX_ATTACHMENT_BYTES};
use crate::models::GmailImportSummary;
use crate::scanner;
use chrono::{TimeZone, Utc};
use imap::types::NameAttribute;
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use native_tls::{TlsConnector, TlsStream};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

const IMAP_HOST: &str = "imap.gmail.com";
const IMAP_PORT: u16 = 993;
const IMAP_CREDENTIAL_PREFIX: &str = "gr.smartlibrary.desktop.gmail.imap";
const MAX_GMAIL_QUERY_CHARS: usize = 512;
const MAX_IMAP_MESSAGE_BYTES: usize = MAX_ATTACHMENT_BYTES + 16 * 1024 * 1024;
const MAX_ATTACHMENTS_PER_MESSAGE: usize = 250;
const NETWORK_TIMEOUT: Duration = Duration::from_secs(45);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

type ImapSession = imap::Session<TlsStream<TcpStream>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredImapCredential {
    email: String,
    app_password: String,
}

#[derive(Debug)]
struct DecodedAttachment {
    part_id: String,
    filename: String,
    mime_type: String,
    bytes: Vec<u8>,
}

pub(crate) fn connect(
    database_path: &Path,
    email: &str,
    app_password: &str,
) -> Result<String, String> {
    if !credential_store::is_supported() {
        return Err(
            "Η ασφαλής σύνδεση Gmail υποστηρίζεται σε Windows και Linux.".to_string(),
        );
    }

    let credential = normalized_credential(email, app_password)?;
    let mut session = open_session(&credential)?;
    let mailbox = all_mailbox_name(&mut session)?;
    session.examine(&mailbox).map_err(|error| {
        format!("Το Gmail δεν άνοιξε το γραμματοκιβώτιο μόνο για ανάγνωση: {error}")
    })?;
    let _ = session.logout();

    let target = credential_target(&credential.email);
    let previous_secret = credential_store::get(&target)?;
    let previous_account =
        database::active_gmail_account(database_path)?.map(|account| account.account_email);
    let secret = serde_json::to_vec(&credential).map_err(|error| error.to_string())?;
    credential_store::set(&target, &secret)?;

    if let Err(error) = database::upsert_gmail_account(database_path, &credential.email) {
        if let Some(previous) = previous_secret {
            let _ = credential_store::set(&target, &previous);
        } else {
            let _ = credential_store::delete(&target);
        }
        return Err(error);
    }

    if let Some(previous_email) = previous_account {
        if previous_email != credential.email {
            credential_store::delete(&credential_target(&previous_email)).map_err(|error| {
                format!(
                    "Ο νέος λογαριασμός συνδέθηκε, αλλά δεν αφαιρέθηκε ο παλιός κωδικός εφαρμογής: {error}"
                )
            })?;
        }
    }

    Ok(credential.email)
}

pub(crate) fn credential_exists(account_email: &str) -> Result<bool, String> {
    Ok(credential_store::get(&credential_target(account_email))?.is_some())
}

pub(crate) fn delete_credentials(account_email: &str) -> Result<(), String> {
    credential_store::delete(&credential_target(account_email))
}

pub(crate) fn import_attachments(
    database_path: &Path,
    account_email: &str,
    query: &str,
    max_messages: usize,
) -> Result<GmailImportSummary, String> {
    let credential = load_credential(account_email)?;
    let connection = database::open(database_path)?;
    let enabled_extensions = database::enabled_extension_set(&connection)?;
    drop(connection);

    let account_root = gmail_inbox_root(account_email);
    fs::create_dir_all(&account_root)
        .map_err(|error| format!("Δεν δημιουργήθηκε το Gmail Inbox: {error}"))?;
    let mut summary = GmailImportSummary {
        account_email: account_email.to_string(),
        messages_scanned: 0,
        attachments_found: 0,
        imported: 0,
        duplicates: 0,
        skipped: 0,
        failed: 0,
        indexed_after_sync: 0,
        source_root: account_root.to_string_lossy().to_string(),
    };

    let mut session = open_session(&credential)?;
    let mailbox = all_mailbox_name(&mut session)?;
    session.examine(&mailbox).map_err(|error| {
        format!("Το Gmail δεν άνοιξε το γραμματοκιβώτιο μόνο για ανάγνωση: {error}")
    })?;

    let search = gmail_raw_search(query)?;
    let mut uids: Vec<u32> = session
        .uid_search(search)
        .map_err(|error| format!("Η αναζήτηση Gmail απέτυχε: {error}"))?
        .into_iter()
        .collect();
    uids.sort_unstable_by(|left, right| right.cmp(left));
    uids.truncate(max_messages.clamp(1, 500));

    for uid in uids {
        summary.messages_scanned += 1;
        match process_message(
            &mut session,
            uid,
            database_path,
            account_email,
            &account_root,
            &enabled_extensions,
            &mut summary,
        ) {
            Ok(()) => {}
            Err(MessageFailure::Skipped) => summary.skipped += 1,
            Err(MessageFailure::Failed) => summary.failed += 1,
        }
    }
    let _ = session.logout();

    database::finish_gmail_sync(database_path, account_email)?;
    let scan = scanner::scan_directory(&account_root, database_path)?;
    summary.indexed_after_sync = scan.indexed;
    Ok(summary)
}

#[derive(Debug, Clone, Copy)]
enum MessageFailure {
    Skipped,
    Failed,
}

#[allow(clippy::too_many_arguments)]
fn process_message(
    session: &mut ImapSession,
    uid: u32,
    database_path: &Path,
    account_email: &str,
    account_root: &Path,
    enabled_extensions: &std::collections::HashSet<String>,
    summary: &mut GmailImportSummary,
) -> Result<(), MessageFailure> {
    let uid_text = uid.to_string();
    let metadata = session
        .uid_fetch(&uid_text, "(UID RFC822.SIZE)")
        .map_err(|_| MessageFailure::Failed)?;
    let size = metadata
        .iter()
        .find(|item| item.uid == Some(uid))
        .and_then(|item| item.size)
        .ok_or(MessageFailure::Failed)? as usize;
    drop(metadata);
    if size > MAX_IMAP_MESSAGE_BYTES {
        return Err(MessageFailure::Skipped);
    }

    let fetches = session
        .uid_fetch(&uid_text, "(UID INTERNALDATE RFC822.SIZE BODY.PEEK[])")
        .map_err(|_| MessageFailure::Failed)?;
    let fetch = fetches
        .iter()
        .find(|item| item.uid == Some(uid))
        .ok_or(MessageFailure::Failed)?;
    let body = fetch.body().ok_or(MessageFailure::Failed)?;
    if body.len() > MAX_IMAP_MESSAGE_BYTES {
        return Err(MessageFailure::Skipped);
    }

    let parsed = parse_mail(body).map_err(|_| MessageFailure::Failed)?;
    let sender = limited_header(&parsed, "From", 2_048);
    let subject = limited_header(&parsed, "Subject", 4_096);
    let header_date = limited_header(&parsed, "Date", 512);
    let message_time = fetch
        .internal_date()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            mailparse::dateparse(&header_date)
                .ok()
                .and_then(|timestamp| Utc.timestamp_opt(timestamp, 0).single())
        })
        .unwrap_or_else(Utc::now);
    let message_date = if header_date.is_empty() {
        message_time.to_rfc3339()
    } else {
        header_date
    };
    let message_id = limited_header(&parsed, "Message-ID", 1_024);
    let message_id = if message_id.is_empty() {
        format!("imap-all-{uid}")
    } else {
        message_id
    };

    let (attachments, decode_failures, capped) = collect_attachments(&parsed);
    summary.attachments_found += attachments.len() + decode_failures;
    summary.failed += decode_failures;
    if capped {
        summary.skipped += 1;
    }

    for attachment in attachments {
        let result = persist_attachment(
            database_path,
            account_email,
            account_root,
            enabled_extensions,
            &message_id,
            "",
            &attachment.part_id,
            &sender,
            &subject,
            &message_date,
            &message_time,
            &attachment.filename,
            &attachment.mime_type,
            &attachment.bytes,
        );
        match result {
            Ok(AttachmentOutcome::Imported) => summary.imported += 1,
            Ok(AttachmentOutcome::Duplicate) => summary.duplicates += 1,
            Ok(AttachmentOutcome::Skipped) => summary.skipped += 1,
            Err(_) => summary.failed += 1,
        }
    }
    Ok(())
}

fn collect_attachments(message: &ParsedMail<'_>) -> (Vec<DecodedAttachment>, usize, bool) {
    let mut attachments = Vec::new();
    let mut failures = 0;
    let mut capped = false;

    for (index, part) in message.parts().enumerate() {
        let disposition = part.get_content_disposition();
        let filename = disposition
            .params
            .get("filename")
            .or_else(|| part.ctype.params.get("name"))
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let Some(filename) = filename else {
            continue;
        };
        if attachments.len() >= MAX_ATTACHMENTS_PER_MESSAGE {
            capped = true;
            break;
        }
        match part.get_body_raw() {
            Ok(bytes) => attachments.push(DecodedAttachment {
                part_id: format!("mime-{index}"),
                filename: filename.to_string(),
                mime_type: if part.ctype.mimetype.trim().is_empty() {
                    "application/octet-stream".to_string()
                } else {
                    part.ctype.mimetype.clone()
                },
                bytes,
            }),
            Err(_) => failures += 1,
        }
    }

    (attachments, failures, capped)
}

fn open_session(credential: &StoredImapCredential) -> Result<ImapSession, String> {
    let mut last_error = None;
    let addresses = (IMAP_HOST, IMAP_PORT)
        .to_socket_addrs()
        .map_err(|error| format!("Δεν βρέθηκε ο διακομιστής Gmail: {error}"))?;
    let mut stream = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) {
            Ok(candidate) => {
                stream = Some(candidate);
                break;
            }
            Err(error) => last_error = Some(error),
        }
    }
    let stream = stream.ok_or_else(|| {
        format!(
            "Δεν έγινε σύνδεση στο Gmail: {}",
            last_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "δεν βρέθηκε διαθέσιμη διεύθυνση".to_string())
        )
    })?;
    stream
        .set_read_timeout(Some(NETWORK_TIMEOUT))
        .map_err(|error| format!("Δεν ρυθμίστηκε το χρονικό όριο Gmail: {error}"))?;
    stream
        .set_write_timeout(Some(NETWORK_TIMEOUT))
        .map_err(|error| format!("Δεν ρυθμίστηκε το χρονικό όριο Gmail: {error}"))?;

    let connector = TlsConnector::builder()
        .build()
        .map_err(|error| format!("Δεν δημιουργήθηκε ασφαλής σύνδεση TLS: {error}"))?;
    let tls = connector
        .connect(IMAP_HOST, stream)
        .map_err(|error| format!("Απέτυχε ο έλεγχος ασφαλείας TLS του Gmail: {error}"))?;
    let mut client = imap::Client::new(tls);
    client
        .read_greeting()
        .map_err(|error| format!("Το Gmail δεν απάντησε σωστά: {error}"))?;
    client
        .login(&credential.email, &credential.app_password)
        .map_err(|(error, _)| {
            let detail: String = error.to_string().chars().take(240).collect();
            format!(
                "Το Gmail απέρριψε τη σύνδεση. Έλεγξε τη διεύθυνση και τον κωδικό εφαρμογής 16 χαρακτήρων. ({detail})"
            )
        })
}

fn all_mailbox_name(session: &mut ImapSession) -> Result<String, String> {
    let names = session
        .list(None, Some("*"))
        .map_err(|error| format!("Δεν διαβάστηκε η λίστα φακέλων Gmail: {error}"))?;
    Ok(names
        .iter()
        .find(|name| {
            name.attributes().iter().any(|attribute| {
                matches!(
                    attribute,
                    NameAttribute::Custom(value) if value.eq_ignore_ascii_case("\\All")
                )
            })
        })
        .map(|name| name.name().to_string())
        .unwrap_or_else(|| "INBOX".to_string()))
}

fn gmail_raw_search(query: &str) -> Result<String, String> {
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
    if query
        .chars()
        .any(|character| matches!(character, '\r' | '\n' | '\0'))
    {
        return Err("Το Gmail query περιέχει μη επιτρεπτούς χαρακτήρες.".to_string());
    }
    let escaped = query.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!("X-GM-RAW \"{escaped}\""))
}

fn normalized_credential(email: &str, app_password: &str) -> Result<StoredImapCredential, String> {
    let email = email.trim().to_ascii_lowercase();
    let mut pieces = email.split('@');
    let local = pieces.next().unwrap_or("");
    let domain = pieces.next().unwrap_or("");
    if local.is_empty()
        || domain.is_empty()
        || pieces.next().is_some()
        || !email.is_ascii()
        || email
            .chars()
            .any(|character| character.is_ascii_whitespace() || character.is_ascii_control())
        || !domain.contains('.')
    {
        return Err("Γράψε ολόκληρη και έγκυρη διεύθυνση Gmail.".to_string());
    }

    let app_password: String = app_password
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    if app_password.len() != 16
        || !app_password
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return Err(
            "Χρειάζεται ο κωδικός εφαρμογής 16 χαρακτήρων της Google, όχι ο κανονικός κωδικός Gmail."
                .to_string(),
        );
    }

    Ok(StoredImapCredential {
        email,
        app_password,
    })
}

fn load_credential(account_email: &str) -> Result<StoredImapCredential, String> {
    let bytes = credential_store::get(&credential_target(account_email))?.ok_or_else(|| {
        "Ο κωδικός εφαρμογής Gmail δεν βρέθηκε. Σύνδεσε ξανά τον λογαριασμό.".to_string()
    })?;
    let credential: StoredImapCredential = serde_json::from_slice(&bytes).map_err(|_| {
        "Η ασφαλής ρύθμιση Gmail δεν διαβάζεται. Σύνδεσε ξανά τον λογαριασμό.".to_string()
    })?;
    if credential.email != account_email.trim().to_ascii_lowercase() {
        return Err("Η ασφαλής ρύθμιση Gmail δεν αντιστοιχεί στον λογαριασμό.".to_string());
    }
    normalized_credential(&credential.email, &credential.app_password)
}

fn credential_target(account_email: &str) -> String {
    let normalized_email = account_email.trim().to_ascii_lowercase();
    let account_hash = hex::encode(Sha256::digest(normalized_email.as_bytes()));
    format!("{IMAP_CREDENTIAL_PREFIX}.{}", account_hash)
}

fn limited_header(message: &ParsedMail<'_>, name: &str, maximum: usize) -> String {
    message
        .headers
        .get_first_value(name)
        .unwrap_or_default()
        .trim()
        .chars()
        .take(maximum)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_google_app_password() {
        let value = normalized_credential(" User@Gmail.com ", "abcd efgh ijkl mnop").unwrap();
        assert_eq!(value.email, "user@gmail.com");
        assert_eq!(value.app_password, "abcdefghijklmnop");
    }

    #[test]
    fn rejects_a_regular_password() {
        assert!(normalized_credential("user@gmail.com", "ordinary-password").is_err());
    }

    #[test]
    fn quotes_gmail_raw_search_safely() {
        assert_eq!(
            gmail_raw_search("has:attachment subject:\"invoice\"").unwrap(),
            "X-GM-RAW \"has:attachment subject:\\\"invoice\\\"\""
        );
        assert!(gmail_raw_search("has:attachment\r\nLOGOUT").is_err());
    }

    #[test]
    fn decodes_a_mime_attachment() {
        let raw = concat!(
            "From: sender@example.com\r\n",
            "Subject: Test\r\n",
            "MIME-Version: 1.0\r\n",
            "Content-Type: multipart/mixed; boundary=boundary\r\n",
            "\r\n",
            "--boundary\r\n",
            "Content-Type: text/plain\r\n\r\nHello\r\n",
            "--boundary\r\n",
            "Content-Type: application/pdf; name=invoice.pdf\r\n",
            "Content-Disposition: attachment; filename=invoice.pdf\r\n",
            "Content-Transfer-Encoding: base64\r\n\r\n",
            "UERG\r\n",
            "--boundary--\r\n"
        );
        let parsed = parse_mail(raw.as_bytes()).unwrap();
        let (attachments, failures, capped) = collect_attachments(&parsed);
        assert_eq!(failures, 0);
        assert!(!capped);
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "invoice.pdf");
        assert_eq!(attachments[0].bytes, b"PDF");
    }

    #[test]
    fn attachment_limit_is_not_larger_than_storage_limit() {
        assert!(MAX_ATTACHMENT_BYTES < MAX_IMAP_MESSAGE_BYTES);
    }
}
