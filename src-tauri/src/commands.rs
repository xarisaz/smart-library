use crate::database;
use crate::extractor;
use crate::gmail;
use crate::models::{
    AppStatus, EmailProvenance, FileFilterSettings, FileRecord, GmailImportSummary, GmailStatus,
    PreviewPayload, ScanSummary,
};
use crate::scanner;
use crate::AppState;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

const MAX_AUDIO_PREVIEW_BYTES: u64 = 32 * 1024 * 1024;
const MAX_VIDEO_PREVIEW_BYTES: u64 = 40 * 1024 * 1024;

fn validate_indexed_file(path: &str, state: &AppState) -> Result<(), String> {
    if !database::is_indexed_path(&state.db_path, path)? {
        return Err("Το αρχείο δεν ανήκει στο ασφαλές index.".to_string());
    }

    let metadata = fs::metadata(path)
        .map_err(|error| format!("Το αρχείο δεν βρέθηκε πλέον στον δίσκο: {error}"))?;

    if !metadata.is_file() {
        return Err("Η επιλεγμένη διαδρομή δεν είναι αρχείο.".to_string());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn shell_compatible_path(path: &str) -> String {
    if let Some(network_path) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{network_path}");
    }

    path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
}

#[cfg(not(target_os = "windows"))]
fn shell_compatible_path(path: &str) -> String {
    path.to_string()
}

#[cfg(target_os = "windows")]
fn is_text_fallback(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "csv"
                    | "txt"
                    | "json"
                    | "xml"
                    | "html"
                    | "htm"
                    | "md"
                    | "markdown"
                    | "yaml"
                    | "yml"
                    | "gpx"
                    | "kml"
                    | "nmea"
            )
        })
}

fn is_audio_preview(extension: &str) -> bool {
    matches!(
        extension,
        "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "opus"
    )
}

fn is_video_preview(extension: &str) -> bool {
    matches!(extension, "mp4" | "m4v" | "mov" | "webm")
}

fn is_cad(extension: &str) -> bool {
    matches!(
        extension,
        "dwg" | "dxf" | "dwf" | "dwt" | "step" | "stp" | "iges" | "igs" | "stl" | "3mf" | "obj"
    )
}

fn is_image_preview(extension: &str) -> bool {
    matches!(
        extension,
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "avif" | "bmp"
    )
}

fn is_external_image(extension: &str) -> bool {
    matches!(
        extension,
        "tif" | "tiff" | "heic" | "dng" | "cr2" | "cr3" | "nef" | "arw" | "orf" | "rw2"
    )
}

#[tauri::command]
pub fn initialize_app(state: State<'_, AppState>) -> Result<AppStatus, String> {
    database::app_status(&state.db_path)
}

#[tauri::command]
pub fn get_file_filter_settings(state: State<'_, AppState>) -> Result<FileFilterSettings, String> {
    database::file_filter_settings(&state.db_path)
}

#[tauri::command]
pub fn save_file_filter_settings(
    enabled_extensions: Vec<String>,
    state: State<'_, AppState>,
) -> Result<FileFilterSettings, String> {
    database::save_file_filter_settings(&state.db_path, &enabled_extensions)
}

#[tauri::command]
pub fn pick_folder() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Επίλεξε φάκελο για ασφαλή σάρωση")
        .pick_folder()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn gmail_status(state: State<'_, AppState>) -> Result<GmailStatus, String> {
    gmail::status(&state.db_path)
}

#[tauri::command]
pub fn select_gmail_oauth_file(state: State<'_, AppState>) -> Result<GmailStatus, String> {
    let path = rfd::FileDialog::new()
        .set_title("Επίλεξε Google OAuth Desktop JSON")
        .add_filter("Google OAuth JSON", &["json"])
        .pick_file()
        .ok_or_else(|| "Δεν επιλέχθηκε αρχείο OAuth.".to_string())?;
    gmail::configure_oauth_from_file(&path)?;
    gmail::status(&state.db_path)
}

#[tauri::command]
pub async fn connect_gmail(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GmailStatus, String> {
    gmail::connect(&app, &state.db_path).await
}

#[tauri::command]
pub async fn connect_gmail_imap(
    email: String,
    app_password: String,
    state: State<'_, AppState>,
) -> Result<GmailStatus, String> {
    let database_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        gmail::connect_imap(&database_path, &email, &app_password)
    })
    .await
    .map_err(|error| format!("Η σύνδεση Gmail διακόπηκε: {error}"))?
}

#[tauri::command]
pub async fn import_gmail_attachments(
    query: Option<String>,
    max_messages: Option<usize>,
    state: State<'_, AppState>,
) -> Result<GmailImportSummary, String> {
    gmail::import_attachments(
        &state.db_path,
        query.as_deref().unwrap_or("has:attachment"),
        max_messages.unwrap_or(100),
    )
    .await
}

#[tauri::command]
pub fn disconnect_gmail(state: State<'_, AppState>) -> Result<GmailStatus, String> {
    gmail::disconnect(&state.db_path)
}

#[tauri::command]
pub fn get_file_provenance(
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<EmailProvenance>, String> {
    validate_indexed_file(&path, &state)?;
    database::email_provenance(&state.db_path, &path)
}

#[tauri::command]
pub async fn scan_folder(path: String, state: State<'_, AppState>) -> Result<ScanSummary, String> {
    let database_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        scanner::scan_directory(Path::new(&path), &database_path)
    })
    .await
    .map_err(|error| format!("Η σάρωση διακόπηκε: {error}"))?
}

#[tauri::command]
pub fn list_files(
    search: Option<String>,
    category: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<FileRecord>, String> {
    database::list_files(
        &state.db_path,
        search.as_deref().unwrap_or(""),
        category.as_deref().unwrap_or("All"),
        limit.unwrap_or(2_000),
    )
}

#[tauri::command]
pub fn preview_file(path: String, state: State<'_, AppState>) -> Result<PreviewPayload, String> {
    validate_indexed_file(&path, &state)?;

    let file_path = Path::new(&path);
    let metadata = fs::metadata(file_path).map_err(|error| error.to_string())?;
    let extension = file_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime_type = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    if matches!(
        extension.as_str(),
        "txt"
            | "csv"
            | "json"
            | "xml"
            | "html"
            | "htm"
            | "md"
            | "markdown"
            | "yaml"
            | "yml"
            | "gpx"
            | "kml"
            | "nmea"
    ) {
        let bytes = fs::read(file_path).map_err(|error| error.to_string())?;
        let capped = &bytes[..bytes.len().min(512 * 1024)];
        return Ok(PreviewPayload {
            kind: "text".to_string(),
            mime_type,
            text: Some(extractor::decode_plain_text_bytes(capped)),
            data_url: None,
            message: (bytes.len() > capped.len())
                .then(|| "Εμφανίζονται οι πρώτοι 512 KB.".to_string()),
        });
    }

    if is_audio_preview(&extension) {
        if metadata.len() <= MAX_AUDIO_PREVIEW_BYTES {
            let bytes = fs::read(file_path).map_err(|error| error.to_string())?;
            return Ok(PreviewPayload {
                kind: "audio".to_string(),
                mime_type: mime_type.clone(),
                text: None,
                data_url: Some(format!(
                    "data:{mime_type};base64,{}",
                    STANDARD.encode(bytes)
                )),
                message: None,
            });
        }

        return Ok(PreviewPayload {
            kind: "metadata".to_string(),
            mime_type,
            text: None,
            data_url: None,
            message: Some(
                "Το αρχείο ήχου είναι μεγαλύτερο από το όριο εσωτερικής αναπαραγωγής των 32 MB. Χρησιμοποίησε «Άνοιγμα αρχείου»."
                    .to_string(),
            ),
        });
    }

    if is_video_preview(&extension) {
        if metadata.len() <= MAX_VIDEO_PREVIEW_BYTES {
            let bytes = fs::read(file_path).map_err(|error| error.to_string())?;
            return Ok(PreviewPayload {
                kind: "video".to_string(),
                mime_type: mime_type.clone(),
                text: None,
                data_url: Some(format!(
                    "data:{mime_type};base64,{}",
                    STANDARD.encode(bytes)
                )),
                message: None,
            });
        }

        return Ok(PreviewPayload {
            kind: "metadata".to_string(),
            mime_type,
            text: None,
            data_url: None,
            message: Some(
                "Το βίντεο είναι μεγαλύτερο από το όριο εσωτερικής αναπαραγωγής των 40 MB. Χρησιμοποίησε «Άνοιγμα αρχείου»."
                    .to_string(),
            ),
        });
    }

    if matches!(extension.as_str(), "docx" | "xlsx" | "pptx") {
        if let Some(text) = database::extracted_text_preview(&state.db_path, &path, 32_000)? {
            return Ok(PreviewPayload {
                kind: "text".to_string(),
                mime_type,
                text: Some(text),
                data_url: None,
                message: Some("Προεπισκόπηση του κειμένου που διαβάστηκε τοπικά.".to_string()),
            });
        }

        return Ok(PreviewPayload {
            kind: "metadata".to_string(),
            mime_type,
            text: None,
            data_url: None,
            message: Some(
                "Δεν βρέθηκε αναγνώσιμο κείμενο. Κάνε ξανά σάρωση του φακέλου.".to_string(),
            ),
        });
    }

    let can_embed = is_image_preview(&extension) || extension == "pdf";
    if can_embed && metadata.len() <= 20 * 1024 * 1024 {
        let bytes = fs::read(file_path).map_err(|error| error.to_string())?;
        return Ok(PreviewPayload {
            kind: if extension == "pdf" { "pdf" } else { "image" }.to_string(),
            mime_type: mime_type.clone(),
            text: None,
            data_url: Some(format!(
                "data:{mime_type};base64,{}",
                STANDARD.encode(bytes)
            )),
            message: None,
        });
    }

    Ok(PreviewPayload {
        kind: "metadata".to_string(),
        mime_type,
        text: None,
        data_url: None,
        message: Some(if is_cad(&extension) {
            "Το σχέδιο μπήκε στο index. Για πραγματική απεικόνιση DWG/CAD χρειάζεται ειδικός renderer· προς το παρόν χρησιμοποίησε «Άνοιγμα αρχείου».".to_string()
        } else if is_external_image(&extension) {
            "Η φωτογραφία μπήκε στο index, αλλά αυτός ο τύπος RAW/HEIC/TIFF δεν υποστηρίζεται από τον ενσωματωμένο viewer. Χρησιμοποίησε «Άνοιγμα αρχείου».".to_string()
        } else if can_embed {
            "Το αρχείο είναι μεγαλύτερο από το όριο προεπισκόπησης των 20 MB.".to_string()
        } else {
            "Η πλήρης προεπισκόπηση αυτού του τύπου θα προστεθεί στην επόμενη έκδοση.".to_string()
        }),
    })
}

#[tauri::command]
pub fn open_indexed_file(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_indexed_file(&path, &state)?;
    let shell_path = shell_compatible_path(&path);
    let default_result = app.opener().open_path(shell_path.as_str(), None::<&str>);

    match default_result {
        Ok(()) => Ok(()),
        #[cfg(target_os = "windows")]
        Err(default_error) if is_text_fallback(&path) => app
            .opener()
            .open_path(shell_path.as_str(), Some("notepad.exe"))
            .map_err(|fallback_error| {
                format!(
                    "Τα Windows δεν μπόρεσαν να ανοίξουν το αρχείο ({default_error}). Απέτυχε και το εφεδρικό άνοιγμα με το Σημειωματάριο: {fallback_error}"
                )
            }),
        Err(error) => Err(format!(
            "Το λειτουργικό σύστημα δεν μπόρεσε να ανοίξει το αρχείο: {error}"
        )),
    }
}

#[tauri::command]
pub fn reveal_indexed_file(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_indexed_file(&path, &state)?;
    let shell_path = shell_compatible_path(&path);
    app.opener()
        .reveal_item_in_dir(Path::new(&shell_path))
        .map_err(|error| format!("Δεν μπόρεσα να εμφανίσω το αρχείο στον φάκελο: {error}"))
}

#[tauri::command]
pub fn set_review_status(
    file_id: i64,
    status: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    database::update_review_status(&state.db_path, file_id, &status)
}

#[cfg(test)]
mod tests {
    use super::shell_compatible_path;

    #[cfg(target_os = "windows")]
    #[test]
    fn prepares_windows_paths_for_shell_actions() {
        assert_eq!(
            shell_compatible_path(r"\\?\C:\Users\xaris\document.csv"),
            r"C:\Users\xaris\document.csv"
        );
        assert_eq!(
            shell_compatible_path(r"\\?\UNC\server\share\document.csv"),
            r"\\server\share\document.csv"
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn preserves_unix_paths_for_shell_actions() {
        assert_eq!(
            shell_compatible_path("/home/user/Έγγραφα/invoice.pdf"),
            "/home/user/Έγγραφα/invoice.pdf"
        );
    }
}
