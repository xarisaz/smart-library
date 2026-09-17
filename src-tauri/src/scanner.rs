use crate::classifier;
use crate::database;
use crate::extractor;
use crate::models::{FileInsert, ScanSummary};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::{Instant, SystemTime};
use walkdir::{DirEntry, WalkDir};

const MAX_INDEXED_FILES: usize = 50_000;
const MAX_FULL_HASH_BYTES: u64 = 64 * 1024 * 1024;
const DATABASE_BATCH_SIZE: usize = 250;

pub fn scan_directory(root: &Path, database_path: &Path) -> Result<ScanSummary, String> {
    let started = Instant::now();
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("Δεν μπορώ να ανοίξω τον φάκελο: {error}"))?;
    if !canonical_root.is_dir() {
        return Err("Η επιλεγμένη διαδρομή δεν είναι φάκελος.".to_string());
    }

    let source_root = canonical_root.to_string_lossy().to_string();
    let scan_id = format!("{}-{}", Utc::now().timestamp_millis(), std::process::id());
    let mut connection = database::open(database_path)?;
    let enabled_extensions = database::enabled_extension_set(&connection)?;
    database::start_scan(
        &connection,
        &scan_id,
        &source_root,
        &Utc::now().to_rfc3339(),
    )?;

    let mut records = Vec::new();
    let mut ignored = 0usize;
    let mut unreadable = 0usize;
    let mut reached_limit = false;
    let mut categories = BTreeMap::new();
    let mut indexed = 0usize;
    let mut text_extracted = 0usize;
    let mut needs_ocr = 0usize;
    let mut extraction_failed = 0usize;

    let walker = WalkDir::new(&canonical_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_visit);

    for entry_result in walker {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => {
                unreadable += 1;
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        if indexed >= MAX_INDEXED_FILES {
            reached_limit = true;
            break;
        }

        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !classifier::is_indexable(&extension) || !enabled_extensions.contains(&extension) {
            ignored += 1;
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                unreadable += 1;
                continue;
            }
        };
        let modified_utc =
            system_time_to_rfc3339(metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH));
        let extraction = extractor::extract(path, &extension, metadata.len());
        match extraction.status.as_str() {
            "extracted" => text_extracted += 1,
            "needs_ocr" => needs_ocr += 1,
            "failed" => extraction_failed += 1,
            _ => {}
        }
        let classification =
            classifier::classify(path, &extension, &modified_utc, &extraction.text);
        let sha256 = if metadata.len() <= MAX_FULL_HASH_BYTES {
            hash_file(path).ok()
        } else {
            None
        };
        let category = classification.category.clone();

        records.push(FileInsert {
            path: path.to_string_lossy().to_string(),
            name: entry.file_name().to_string_lossy().to_string(),
            extension,
            mime_type: mime_guess::from_path(path)
                .first_or_octet_stream()
                .essence_str()
                .to_string(),
            size_bytes: metadata.len().min(i64::MAX as u64) as i64,
            modified_utc,
            sha256,
            category,
            subcategory: classification.subcategory,
            confidence: classification.confidence,
            proposed_name: classification.proposed_name,
            proposed_relative_path: classification.proposed_relative_path,
            source_root: source_root.clone(),
            extracted_text: extraction.text,
            extraction_status: extraction.status,
        });
        indexed += 1;

        *categories.entry(classification.category).or_insert(0usize) += 1;

        if records.len() >= DATABASE_BATCH_SIZE {
            database::store_scan_batch(&mut connection, &scan_id, &records)?;
            records.clear();
        }
    }

    if !records.is_empty() {
        database::store_scan_batch(&mut connection, &scan_id, &records)?;
    }
    database::finalize_scan_storage(&mut connection, &scan_id, &source_root)?;
    database::finish_scan(&connection, &scan_id, indexed, ignored, unreadable)?;
    let duplicates = database::duplicate_count(&connection, &source_root)?;

    Ok(ScanSummary {
        source_root,
        indexed,
        ignored,
        unreadable,
        duplicates,
        text_extracted,
        needs_ocr,
        extraction_failed,
        reached_limit,
        elapsed_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
        categories,
    })
}

fn should_visit(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || !entry.file_type().is_dir()
        || !classifier::should_skip_directory(entry.path())
}

fn system_time_to_rfc3339(value: SystemTime) -> String {
    let timestamp: DateTime<Utc> = value.into();
    timestamp.to_rfc3339()
}

fn hash_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}
