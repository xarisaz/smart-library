use crate::app_paths;
use crate::classifier;
use crate::models::{
    AppStatus, EmailAttachmentInsert, EmailProvenance, FileFilterSettings, FileInsert, FileRecord,
    GmailAccountRecord,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

const DISABLED_FILE_EXTENSIONS_KEY: &str = "disabled_file_extensions";

pub fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| error.to_string())?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| error.to_string())?;
    Ok(connection)
}

pub fn initialize(path: &Path) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS scan_runs (
                scan_id TEXT PRIMARY KEY,
                source_root TEXT NOT NULL,
                started_utc TEXT NOT NULL,
                finished_utc TEXT,
                indexed INTEGER NOT NULL DEFAULT 0,
                ignored INTEGER NOT NULL DEFAULT 0,
                unreadable INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                extension TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                modified_utc TEXT NOT NULL,
                sha256 TEXT,
                category TEXT NOT NULL,
                subcategory TEXT NOT NULL,
                confidence REAL NOT NULL,
                proposed_name TEXT NOT NULL,
                proposed_relative_path TEXT NOT NULL,
                source_root TEXT NOT NULL,
                extracted_text TEXT NOT NULL DEFAULT '',
                extraction_status TEXT NOT NULL DEFAULT 'not_attempted',
                review_status TEXT NOT NULL DEFAULT 'pending',
                last_seen_scan TEXT NOT NULL,
                created_utc TEXT NOT NULL,
                updated_utc TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_files_category ON files(category);
            CREATE INDEX IF NOT EXISTS idx_files_hash ON files(sha256);
            CREATE INDEX IF NOT EXISTS idx_files_source ON files(source_root);
            CREATE INDEX IF NOT EXISTS idx_files_review ON files(review_status);

            CREATE VIRTUAL TABLE IF NOT EXISTS file_search USING fts5(
                name,
                path,
                category,
                subcategory,
                content
            );

            CREATE TABLE IF NOT EXISTS review_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL,
                previous_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_utc TEXT NOT NULL,
                FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS email_accounts (
                provider TEXT NOT NULL,
                account_email TEXT NOT NULL,
                connected_utc TEXT NOT NULL,
                last_sync_utc TEXT,
                is_connected INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY(provider, account_email)
            );

            CREATE TABLE IF NOT EXISTS email_attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider TEXT NOT NULL,
                account_email TEXT NOT NULL,
                message_id TEXT NOT NULL,
                thread_id TEXT NOT NULL DEFAULT '',
                part_id TEXT NOT NULL,
                sender TEXT NOT NULL DEFAULT '',
                subject TEXT NOT NULL DEFAULT '',
                message_date TEXT NOT NULL DEFAULT '',
                original_filename TEXT NOT NULL,
                stored_path TEXT NOT NULL,
                mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
                size_bytes INTEGER NOT NULL,
                sha256 TEXT NOT NULL,
                import_status TEXT NOT NULL,
                imported_utc TEXT NOT NULL,
                UNIQUE(provider, account_email, message_id, part_id)
            );

            CREATE INDEX IF NOT EXISTS idx_email_attachments_hash
                ON email_attachments(sha256);
            CREATE INDEX IF NOT EXISTS idx_email_attachments_path
                ON email_attachments(stored_path);
            CREATE INDEX IF NOT EXISTS idx_email_attachments_account
                ON email_attachments(provider, account_email);
            "#,
        )
        .map_err(|error| error.to_string())?;
    ensure_file_column(
        &connection,
        "extracted_text",
        "ALTER TABLE files ADD COLUMN extracted_text TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_file_column(
        &connection,
        "extraction_status",
        "ALTER TABLE files ADD COLUMN extraction_status TEXT NOT NULL DEFAULT 'not_attempted'",
    )?;
    Ok(())
}

pub fn file_filter_settings(path: &Path) -> Result<FileFilterSettings, String> {
    let connection = open(path)?;
    file_filter_settings_from_connection(&connection)
}

pub fn enabled_extension_set(connection: &Connection) -> Result<HashSet<String>, String> {
    Ok(file_filter_settings_from_connection(connection)?
        .enabled_extensions
        .into_iter()
        .collect())
}

pub fn save_file_filter_settings(
    path: &Path,
    enabled_extensions: &[String],
) -> Result<FileFilterSettings, String> {
    let connection = open(path)?;
    let supported: BTreeSet<String> = classifier::indexable_extensions()
        .iter()
        .map(|extension| (*extension).to_string())
        .collect();
    let requested: BTreeSet<String> = enabled_extensions
        .iter()
        .map(|extension| {
            extension
                .trim()
                .trim_start_matches('.')
                .to_ascii_lowercase()
        })
        .filter(|extension| !extension.is_empty())
        .collect();

    if requested.is_empty() {
        return Err("Επίλεξε τουλάχιστον έναν τύπο αρχείου.".to_string());
    }

    if let Some(unsupported) = requested
        .iter()
        .find(|extension| !supported.contains(*extension))
    {
        return Err(format!(
            "Ο τύπος .{unsupported} δεν υποστηρίζεται από αυτή την έκδοση."
        ));
    }

    let disabled: Vec<String> = supported.difference(&requested).cloned().collect();
    let serialized = serde_json::to_string(&disabled).map_err(|error| error.to_string())?;
    connection
        .execute(
            r#"
            INSERT INTO settings(key, value) VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![DISABLED_FILE_EXTENSIONS_KEY, serialized],
        )
        .map_err(|error| error.to_string())?;

    file_filter_settings_from_connection(&connection)
}

fn file_filter_settings_from_connection(
    connection: &Connection,
) -> Result<FileFilterSettings, String> {
    let supported_extensions: Vec<String> = classifier::indexable_extensions()
        .iter()
        .map(|extension| (*extension).to_string())
        .collect();
    let saved_value = connection
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![DISABLED_FILE_EXTENSIONS_KEY],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let disabled: HashSet<String> = saved_value
        .and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|extension| extension.to_ascii_lowercase())
        .collect();
    let enabled_extensions = supported_extensions
        .iter()
        .filter(|extension| !disabled.contains(*extension))
        .cloned()
        .collect();

    Ok(FileFilterSettings {
        supported_extensions,
        enabled_extensions,
    })
}

pub fn start_scan(
    connection: &Connection,
    scan_id: &str,
    source_root: &str,
    started_utc: &str,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO scan_runs(scan_id, source_root, started_utc) VALUES (?1, ?2, ?3)",
            params![scan_id, source_root, started_utc],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn store_scan_batch(
    connection: &mut Connection,
    scan_id: &str,
    records: &[FileInsert],
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    {
        let mut statement = transaction
            .prepare(
                r#"
                INSERT INTO files (
                    path, name, extension, mime_type, size_bytes, modified_utc, sha256,
                    category, subcategory, confidence, proposed_name, proposed_relative_path,
                    source_root, extracted_text, extraction_status, review_status,
                    last_seen_scan, created_utc, updated_utc
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                    ?14, ?15, 'pending', ?16, ?17, ?17
                )
                ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    extension = excluded.extension,
                    mime_type = excluded.mime_type,
                    size_bytes = excluded.size_bytes,
                    modified_utc = excluded.modified_utc,
                    sha256 = excluded.sha256,
                    category = excluded.category,
                    subcategory = excluded.subcategory,
                    confidence = excluded.confidence,
                    proposed_name = excluded.proposed_name,
                    proposed_relative_path = excluded.proposed_relative_path,
                    source_root = excluded.source_root,
                    extracted_text = excluded.extracted_text,
                    extraction_status = excluded.extraction_status,
                    last_seen_scan = excluded.last_seen_scan,
                    updated_utc = excluded.updated_utc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let now = chrono::Utc::now().to_rfc3339();
        for record in records {
            statement
                .execute(params![
                    record.path,
                    record.name,
                    record.extension,
                    record.mime_type,
                    record.size_bytes,
                    record.modified_utc,
                    record.sha256,
                    record.category,
                    record.subcategory,
                    record.confidence,
                    record.proposed_name,
                    record.proposed_relative_path,
                    record.source_root,
                    record.extracted_text,
                    record.extraction_status,
                    scan_id,
                    now,
                ])
                .map_err(|error| error.to_string())?;
        }
    }

    transaction.commit().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn finalize_scan_storage(
    connection: &mut Connection,
    scan_id: &str,
    source_root: &str,
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM files WHERE source_root = ?1 AND last_seen_scan <> ?2",
            params![source_root, scan_id],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute("DELETE FROM file_search", [])
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            r#"
            INSERT INTO file_search(rowid, name, path, category, subcategory, content)
            SELECT id, name, path, category, subcategory, extracted_text FROM files
            "#,
            [],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn finish_scan(
    connection: &Connection,
    scan_id: &str,
    indexed: usize,
    ignored: usize,
    unreadable: usize,
) -> Result<(), String> {
    connection
        .execute(
            r#"
            UPDATE scan_runs
            SET finished_utc = ?2, indexed = ?3, ignored = ?4, unreadable = ?5
            WHERE scan_id = ?1
            "#,
            params![
                scan_id,
                chrono::Utc::now().to_rfc3339(),
                indexed as i64,
                ignored as i64,
                unreadable as i64,
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn duplicate_count(connection: &Connection, source_root: &str) -> Result<usize, String> {
    let count = connection
        .query_row(
            r#"
            SELECT COALESCE(SUM(item_count - 1), 0)
            FROM (
                SELECT COUNT(*) AS item_count
                FROM files
                WHERE source_root = ?1 AND sha256 IS NOT NULL
                GROUP BY sha256
                HAVING COUNT(*) > 1
            )
            "#,
            params![source_root],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(count as usize)
}

pub fn list_files(
    path: &Path,
    search: &str,
    category: &str,
    limit: usize,
) -> Result<Vec<FileRecord>, String> {
    let connection = open(path)?;
    let capped_limit = limit.clamp(1, 50_000) as i64;
    let pattern = format!("%{}%", search.trim());
    let mut statement = connection
        .prepare(
            r#"
            SELECT
                f.id, f.path, f.name, f.extension, f.mime_type, f.size_bytes,
                f.modified_utc, f.sha256, f.category, f.subcategory, f.confidence,
                f.proposed_name, f.proposed_relative_path, f.source_root, f.review_status,
                f.extraction_status, substr(f.extracted_text, 1, 600) AS text_preview,
                CASE WHEN f.sha256 IS NOT NULL AND (
                    SELECT COUNT(*) FROM files duplicate WHERE duplicate.sha256 = f.sha256
                ) > 1 THEN 1 ELSE 0 END AS is_duplicate
            FROM files f
            WHERE (?1 = '%%' OR f.name LIKE ?1 OR f.path LIKE ?1 OR f.subcategory LIKE ?1
                OR f.extracted_text LIKE ?1)
              AND (?2 = 'All' OR f.category = ?2)
            ORDER BY f.updated_utc DESC, f.name COLLATE NOCASE ASC
            LIMIT ?3
            "#,
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map(params![pattern, category, capped_limit], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                extension: row.get(3)?,
                mime_type: row.get(4)?,
                size_bytes: row.get(5)?,
                modified_utc: row.get(6)?,
                sha256: row.get(7)?,
                category: row.get(8)?,
                subcategory: row.get(9)?,
                confidence: row.get(10)?,
                proposed_name: row.get(11)?,
                proposed_relative_path: row.get(12)?,
                source_root: row.get(13)?,
                review_status: row.get(14)?,
                extraction_status: row.get(15)?,
                text_preview: row.get(16)?,
                is_duplicate: row.get::<_, i64>(17)? == 1,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn app_status(path: &Path) -> Result<AppStatus, String> {
    let connection = open(path)?;
    let indexed_files = scalar_count(&connection, "SELECT COUNT(*) FROM files")?;
    let pending_review = scalar_count(
        &connection,
        "SELECT COUNT(*) FROM files WHERE review_status = 'pending'",
    )?;
    let duplicates = scalar_count(
        &connection,
        r#"
        SELECT COALESCE(SUM(item_count - 1), 0)
        FROM (
            SELECT COUNT(*) AS item_count FROM files
            WHERE sha256 IS NOT NULL GROUP BY sha256 HAVING COUNT(*) > 1
        )
        "#,
    )?;
    let source_count = scalar_count(&connection, "SELECT COUNT(DISTINCT source_root) FROM files")?;
    let default_library = app_paths::documents_dir()
        .join("Smart Library")
        .to_string_lossy()
        .to_string();
    let smart_library_path = connection
        .query_row(
            "SELECT value FROM settings WHERE key = 'smart_library_path'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or(default_library);

    Ok(AppStatus {
        safe_mode: true,
        indexed_files,
        pending_review,
        duplicates,
        source_count,
        smart_library_path,
    })
}

pub fn is_indexed_path(path: &Path, file_path: &str) -> Result<bool, String> {
    let connection = open(path)?;
    let count = connection
        .query_row(
            "SELECT COUNT(*) FROM files WHERE path = ?1",
            params![file_path],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(count == 1)
}

pub fn extracted_text_preview(
    path: &Path,
    file_path: &str,
    maximum_characters: usize,
) -> Result<Option<String>, String> {
    let connection = open(path)?;
    let capped_limit = maximum_characters.clamp(1, 100_000) as i64;
    let text = connection
        .query_row(
            "SELECT substr(extracted_text, 1, ?2) FROM files WHERE path = ?1",
            params![file_path, capped_limit],
            |row| row.get::<_, String>(0),
        )
        .map_err(|error| error.to_string())?;

    Ok((!text.trim().is_empty()).then_some(text))
}

pub fn update_review_status(path: &Path, file_id: i64, new_status: &str) -> Result<(), String> {
    if !matches!(new_status, "pending" | "approved" | "skipped") {
        return Err("Μη έγκυρη κατάσταση αξιολόγησης.".to_string());
    }

    let mut connection = open(path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let previous_status = transaction
        .query_row(
            "SELECT review_status FROM files WHERE id = ?1",
            params![file_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "Το αρχείο δεν βρέθηκε στο index.".to_string())?;

    transaction
        .execute(
            "UPDATE files SET review_status = ?2, updated_utc = ?3 WHERE id = ?1",
            params![file_id, new_status, chrono::Utc::now().to_rfc3339()],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            r#"
            INSERT INTO review_events(file_id, previous_status, new_status, changed_utc)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                file_id,
                previous_status,
                new_status,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn active_gmail_account(path: &Path) -> Result<Option<GmailAccountRecord>, String> {
    let connection = open(path)?;
    connection
        .query_row(
            r#"
            SELECT account_email, connected_utc, last_sync_utc, is_connected
            FROM email_accounts
            WHERE provider = 'gmail' AND is_connected = 1
            ORDER BY connected_utc DESC
            LIMIT 1
            "#,
            [],
            |row| {
                Ok(GmailAccountRecord {
                    account_email: row.get(0)?,
                    connected_utc: row.get(1)?,
                    last_sync_utc: row.get(2)?,
                    is_connected: row.get::<_, i64>(3)? == 1,
                })
            },
        )
        .optional()
        .map_err(|error| error.to_string())
}

pub fn upsert_gmail_account(path: &Path, account_email: &str) -> Result<(), String> {
    let mut connection = open(path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE email_accounts SET is_connected = 0 WHERE provider = 'gmail'",
            [],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            r#"
            INSERT INTO email_accounts(provider, account_email, connected_utc, is_connected)
            VALUES ('gmail', ?1, ?2, 1)
            ON CONFLICT(provider, account_email) DO UPDATE SET
                connected_utc = excluded.connected_utc,
                is_connected = 1
            "#,
            params![account_email, chrono::Utc::now().to_rfc3339()],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

pub fn disconnect_gmail_account(path: &Path, account_email: &str) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            UPDATE email_accounts
            SET is_connected = 0
            WHERE provider = 'gmail' AND account_email = ?1
            "#,
            params![account_email],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn finish_gmail_sync(path: &Path, account_email: &str) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            UPDATE email_accounts
            SET last_sync_utc = ?2
            WHERE provider = 'gmail' AND account_email = ?1
            "#,
            params![account_email, chrono::Utc::now().to_rfc3339()],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn imported_attachment_count(path: &Path, account_email: &str) -> Result<i64, String> {
    let connection = open(path)?;
    connection
        .query_row(
            r#"
            SELECT COUNT(*) FROM email_attachments
            WHERE provider = 'gmail' AND account_email = ?1
            "#,
            params![account_email],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn email_attachment_exists(
    path: &Path,
    account_email: &str,
    message_id: &str,
    part_id: &str,
) -> Result<bool, String> {
    let connection = open(path)?;
    let count = connection
        .query_row(
            r#"
            SELECT COUNT(*) FROM email_attachments
            WHERE provider = 'gmail' AND account_email = ?1
              AND message_id = ?2 AND part_id = ?3
            "#,
            params![account_email, message_id, part_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(count > 0)
}

pub fn existing_path_for_hash(path: &Path, sha256: &str) -> Result<Option<String>, String> {
    let connection = open(path)?;
    let indexed_path = connection
        .query_row(
            "SELECT path FROM files WHERE sha256 = ?1 LIMIT 1",
            params![sha256],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    if indexed_path.is_some() {
        return Ok(indexed_path);
    }

    connection
        .query_row(
            r#"
            SELECT stored_path FROM email_attachments
            WHERE sha256 = ?1 AND import_status = 'imported'
            LIMIT 1
            "#,
            params![sha256],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())
}

pub fn record_email_attachment(path: &Path, item: &EmailAttachmentInsert) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            INSERT INTO email_attachments(
                provider, account_email, message_id, thread_id, part_id,
                sender, subject, message_date, original_filename, stored_path,
                mime_type, size_bytes, sha256, import_status, imported_utc
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15
            )
            ON CONFLICT(provider, account_email, message_id, part_id) DO NOTHING
            "#,
            params![
                &item.provider,
                &item.account_email,
                &item.message_id,
                &item.thread_id,
                &item.part_id,
                &item.sender,
                &item.subject,
                &item.message_date,
                &item.original_filename,
                &item.stored_path,
                &item.mime_type,
                item.size_bytes,
                &item.sha256,
                &item.import_status,
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn email_provenance(path: &Path, stored_path: &str) -> Result<Vec<EmailProvenance>, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT provider, account_email, message_id, sender, subject,
                   message_date, original_filename, import_status
            FROM email_attachments
            WHERE stored_path = ?1
            ORDER BY imported_utc DESC
            LIMIT 20
            "#,
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![stored_path], |row| {
            Ok(EmailProvenance {
                provider: row.get(0)?,
                account_email: row.get(1)?,
                message_id: row.get(2)?,
                sender: row.get(3)?,
                subject: row.get(4)?,
                message_date: row.get(5)?,
                original_filename: row.get(6)?,
                import_status: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn scalar_count(connection: &Connection, query: &str) -> Result<i64, String> {
    connection
        .query_row(query, [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

fn ensure_file_column(
    connection: &Connection,
    column_name: &str,
    alter_sql: &str,
) -> Result<(), String> {
    let column_exists = {
        let mut statement = connection
            .prepare("PRAGMA table_info(files)")
            .map_err(|error| error.to_string())?;
        let column_names = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| error.to_string())?;
        let mut found = false;
        for name in column_names {
            if name.map_err(|error| error.to_string())? == column_name {
                found = true;
                break;
            }
        }
        found
    };

    if column_exists {
        return Ok(());
    }

    connection
        .execute(alter_sql, [])
        .map_err(|error| error.to_string())?;
    Ok(())
}
