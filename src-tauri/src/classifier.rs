use std::path::Path;

#[derive(Debug)]
pub struct Classification {
    pub category: String,
    pub subcategory: String,
    pub confidence: f64,
    pub proposed_name: String,
    pub proposed_relative_path: String,
}

const INDEXABLE_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "csv", "txt", "rtf", "odt", "ods", "odp", "ppt", "pptx",
    "md", "markdown", "epub", "mobi", "xml", "json", "yaml", "yml", "html", "htm", "jpg", "jpeg",
    "png", "webp", "gif", "avif", "tif", "tiff", "bmp", "heic", "dng", "cr2", "cr3", "nef", "arw",
    "orf", "rw2", "zip", "rar", "7z", "eml", "msg", "dwg", "dxf", "dwf", "dwt", "step", "stp",
    "iges", "igs", "stl", "3mf", "obj", "mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "wma",
    "mp4", "m4v", "mov", "mkv", "avi", "webm", "gpx", "kml", "kmz", "nmea",
];

const SKIPPED_DIRECTORIES: &[&str] = &[
    "$recycle.bin",
    "system volume information",
    "windows",
    "program files",
    "program files (x86)",
    "programdata",
    "appdata",
    "steamapps",
    "steamlibrary",
    "epic games",
    "xboxgames",
    "node_modules",
    ".git",
    ".cache",
    ".config",
    ".local",
    ".mozilla",
    ".steam",
    ".var",
    "temp",
];

pub fn should_skip_directory(path: &Path) -> bool {
    #[cfg(target_os = "linux")]
    if is_linux_root_system_path(path) {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| SKIPPED_DIRECTORIES.contains(&name.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn is_linux_root_system_path(path: &Path) -> bool {
    use std::path::Component;

    let mut components = path.components();
    if !matches!(components.next(), Some(Component::RootDir)) {
        return false;
    }

    matches!(
        components
            .next()
            .and_then(|component| component.as_os_str().to_str()),
        Some(
            "bin"
                | "boot"
                | "dev"
                | "etc"
                | "lib"
                | "lib64"
                | "lost+found"
                | "opt"
                | "proc"
                | "root"
                | "run"
                | "sbin"
                | "snap"
                | "sys"
                | "tmp"
                | "usr"
                | "var"
        )
    )
}

pub fn is_indexable(extension: &str) -> bool {
    INDEXABLE_EXTENSIONS.contains(&extension)
}

pub fn indexable_extensions() -> &'static [&'static str] {
    INDEXABLE_EXTENSIONS
}

pub fn classify(
    path: &Path,
    extension: &str,
    modified_date: &str,
    extracted_text: &str,
) -> Classification {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("untitled")
        .to_string();
    let path_text = path.to_string_lossy().to_lowercase();
    let content_text = extracted_text.to_lowercase();
    let searchable = format!("{path_text}\n{content_text}");

    let file_stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    let cv_content_signals = count_matches(
        &content_text,
        &[
            "curriculum vitae",
            "work experience",
            "professional experience",
            "employment history",
            "education",
            "skills",
            "personal details",
            "βιογραφικό",
            "βιογραφικο",
            "εργασιακή εμπειρία",
            "εργασιακη εμπειρια",
            "εκπαίδευση",
            "εκπαιδευση",
            "δεξιότητες",
            "δεξιοτητες",
        ],
    );
    let is_cv_name = file_stem == "cv"
        || file_stem.starts_with("cv ")
        || file_stem.starts_with("cv_")
        || file_stem.contains("curriculum")
        || file_stem.contains("resume")
        || file_stem.contains("βιογραφ");
    let is_cv = is_cv_name || cv_content_signals >= 2;

    let is_invoice_document = contains_any(
        &searchable,
        &[
            "invoice",
            "receipt",
            "timolog",
            "τιμολ",
            "apodeix",
            "αποδειξ",
            "quotation",
            "προσφορ",
        ],
    );
    let invoice_supporting_signals = count_matches(
        &content_text,
        &[
            "subtotal",
            "total",
            "vat",
            "invoice number",
            "amount due",
            "αφμ",
            "φπα",
            "σύνολο",
            "συνολο",
        ],
    );
    let is_invoice = is_invoice_document || invoice_supporting_signals >= 3;
    let is_yachting = contains_any(
        &searchable,
        &[
            "ferretti",
            "lagoon",
            "yacht",
            "boat",
            "skaf",
            "σκαφ",
            "naut",
            "ναυτ",
            "caterpillar",
            "bessenzoni",
            "lofrans",
            "charter",
            "engine hour",
            "engine_hour",
            "generator hour",
            "generator_hour",
            "main engine",
            "port engine",
            "starboard engine",
            "m/e hours",
            "gen hours",
            "ώρες μηχαν",
            "ωρες μηχαν",
            "γεννήτρι",
            "γεννητρι",
        ],
    );
    let is_manual = contains_any(
        &searchable,
        &[
            "manual",
            "service bulletin",
            "instructions",
            "οδηγι",
            "εγχειριδ",
        ],
    );
    let is_bank_or_tax = contains_any(
        &searchable,
        &[
            "bank",
            "piraeus",
            "alpha bank",
            "eurobank",
            "iban",
            "φορο",
            "eforia",
            "aade",
            "εκκαθαρισ",
            "tax",
        ],
    );
    let is_property = contains_any(
        &searchable,
        &[
            "property",
            "akinito",
            "ακινητ",
            "ktimat",
            "κτηματο",
            "οικοπεδ",
            "rent",
            "lease",
            "μισθω",
        ],
    );
    let is_vehicle = contains_any_token(
        &searchable,
        &["audi", "vehicle", "vehicles", "car", "cars", "kteo"],
    ) || contains_any(&searchable, &["αυτοκι", "κτεο"])
        || searchable.contains("service car");
    let is_legal = contains_any(
        &searchable,
        &[
            "contract",
            "agreement",
            "legal",
            "συμβασ",
            "νομικ",
            "δήλωση",
            "dilosi",
        ],
    );

    let (category, subcategory, confidence, destination) = if is_navigation_file(extension) {
        (
            "Work",
            "Yachting / Navigation & Routes",
            0.9,
            "02_Work/Yachting/Navigation_and_Routes",
        )
    } else if is_cad(extension) {
        (
            "Technical",
            "CAD & Technical Drawings",
            0.96,
            "09_Technical/CAD",
        )
    } else if is_audio(extension) {
        ("Media", "Audio", 0.96, "10_Media/Audio")
    } else if is_video(extension) {
        ("Media", "Video", 0.96, "10_Media/Video")
    } else if is_cv {
        (
            "Personal",
            "CV & Qualifications",
            0.93,
            "01_Personal/CV_and_Qualifications",
        )
    } else if is_yachting && is_invoice {
        (
            "Work",
            "Yachting / Ferretti 80 / Invoices",
            0.94,
            "02_Work/Yachting/Ferretti_80/Invoices",
        )
    } else if is_yachting && is_manual {
        (
            "Work",
            "Yachting / Ferretti 80 / Manuals",
            0.93,
            "02_Work/Yachting/Ferretti_80/Manuals",
        )
    } else if is_yachting {
        ("Work", "Yachting", 0.86, "02_Work/Yachting/_Needs_Review")
    } else if is_invoice {
        ("Finance", "Invoices", 0.91, "03_Finance/Invoices")
    } else if is_bank_or_tax {
        ("Finance", "Banks & Tax", 0.87, "03_Finance/Banks_and_Tax")
    } else if is_property {
        ("Property", "Property documents", 0.84, "04_Property")
    } else if is_vehicle {
        ("Vehicles", "Vehicle documents", 0.84, "05_Vehicles")
    } else if is_legal {
        ("Legal", "Contracts & Legal", 0.82, "06_Contracts_Legal")
    } else if is_manual {
        ("Manuals", "Manuals", 0.86, "08_Manuals")
    } else if is_image(extension) {
        ("Photos", "Unsorted photos", 0.72, "07_Photos/_Needs_Review")
    } else {
        ("Review", "Needs review", 0.55, "_Needs_Review")
    };

    let proposed_name = propose_name(&file_name, is_invoice, modified_date);

    Classification {
        category: category.to_string(),
        subcategory: subcategory.to_string(),
        confidence,
        proposed_name,
        proposed_relative_path: destination.to_string(),
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn contains_any_token(value: &str, needles: &[&str]) -> bool {
    value
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| needles.contains(&token))
}

fn count_matches(value: &str, needles: &[&str]) -> usize {
    needles
        .iter()
        .copied()
        .filter(|needle| value.contains(*needle))
        .count()
}

fn is_image(extension: &str) -> bool {
    matches!(
        extension,
        "jpg"
            | "jpeg"
            | "png"
            | "webp"
            | "gif"
            | "avif"
            | "tif"
            | "tiff"
            | "bmp"
            | "heic"
            | "dng"
            | "cr2"
            | "cr3"
            | "nef"
            | "arw"
            | "orf"
            | "rw2"
    )
}

fn is_cad(extension: &str) -> bool {
    matches!(
        extension,
        "dwg" | "dxf" | "dwf" | "dwt" | "step" | "stp" | "iges" | "igs" | "stl" | "3mf" | "obj"
    )
}

fn is_audio(extension: &str) -> bool {
    matches!(
        extension,
        "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "opus" | "wma"
    )
}

fn is_video(extension: &str) -> bool {
    matches!(extension, "mp4" | "m4v" | "mov" | "mkv" | "avi" | "webm")
}

fn is_navigation_file(extension: &str) -> bool {
    matches!(extension, "gpx" | "kml" | "kmz" | "nmea")
}

fn propose_name(original: &str, is_invoice: bool, modified_date: &str) -> String {
    let cleaned = original
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    let begins_with_date = cleaned
        .chars()
        .take(4)
        .all(|character| character.is_ascii_digit());

    if is_invoice && !begins_with_date && modified_date.len() >= 10 {
        format!("{}_{}", &modified_date[..10], cleaned)
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_a_ferretti_invoice_inside_yachting() {
        let result = classify(
            Path::new(r"C:\Users\User\Downloads\Ferretti_CAT_invoice_44.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "",
        );
        assert_eq!(result.category, "Work");
        assert!(result.subcategory.contains("Invoices"));
        assert!(result.proposed_name.starts_with("2026-09-15"));
    }

    #[test]
    fn identifies_property_documents() {
        let result = classify(
            Path::new(r"D:\Scans\ktimatologio_kalyvia.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "",
        );
        assert_eq!(result.category, "Property");
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn keeps_unknown_scans_for_review() {
        let result = classify(
            Path::new(r"D:\Various\scan00042.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "",
        );
        assert_eq!(result.category, "Review");
        assert!(result.confidence < 0.7);
    }

    #[test]
    fn excludes_game_and_system_directories() {
        assert!(should_skip_directory(Path::new("SteamLibrary")));
        assert!(should_skip_directory(Path::new("Windows")));
        assert!(should_skip_directory(Path::new(".cache")));
        assert!(!should_skip_directory(Path::new("Documents")));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn excludes_only_real_linux_root_system_directories() {
        assert!(should_skip_directory(Path::new("/proc")));
        assert!(should_skip_directory(Path::new("/usr/share")));
        assert!(should_skip_directory(Path::new("/var/lib")));
        assert!(!should_skip_directory(Path::new("/home/user/Documents")));
        assert!(!should_skip_directory(Path::new("/mnt/archive/usr")));
    }

    #[test]
    fn recognizes_cv_from_pdf_content() {
        let result = classify(
            Path::new(r"C:\Users\User\Downloads\document_17.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "Curriculum Vitae Professional Experience Education Skills",
        );
        assert_eq!(result.category, "Personal");
        assert_eq!(result.subcategory, "CV & Qualifications");
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn recognizes_invoice_from_pdf_content() {
        let result = classify(
            Path::new(r"C:\Users\User\Downloads\F1244914.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "INVOICE Number F1244914 VAT Total Amount Due",
        );
        assert_eq!(result.category, "Finance");
        assert_eq!(result.subcategory, "Invoices");
    }

    #[test]
    fn recognizes_engine_hours_spreadsheet_from_content() {
        let result = classify(
            Path::new(r"C:\Users\User\Downloads\log.xlsx"),
            "xlsx",
            "2026-09-15T10:00:00Z",
            "Date Port engine hours Starboard engine hours Generator hours",
        );
        assert_eq!(result.category, "Work");
        assert_eq!(result.subcategory, "Yachting");
    }

    #[test]
    fn classifies_cad_and_media_extensions() {
        let cad = classify(
            Path::new(r"D:\Drawings\general_arrangement.dwg"),
            "dwg",
            "2026-09-15T10:00:00Z",
            "",
        );
        let audio = classify(
            Path::new(r"D:\Audio\voice_note.mp3"),
            "mp3",
            "2026-09-15T10:00:00Z",
            "",
        );
        assert_eq!(cad.category, "Technical");
        assert_eq!(audio.category, "Media");
    }

    #[test]
    fn does_not_confuse_audio_with_audi() {
        let document = classify(
            Path::new(r"D:\Audio\notes.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "",
        );
        let audi_recording = classify(
            Path::new(r"D:\Audio\Audi_engine_sound.mp3"),
            "mp3",
            "2026-09-15T10:00:00Z",
            "",
        );
        let audi_document = classify(
            Path::new(r"D:\Documents\Audi_A3_KTEO.pdf"),
            "pdf",
            "2026-09-15T10:00:00Z",
            "",
        );

        assert_ne!(document.category, "Vehicles");
        assert_eq!(audi_recording.category, "Media");
        assert_eq!(audi_document.category, "Vehicles");
    }

    #[test]
    fn indexes_useful_extended_formats() {
        for extension in ["dwg", "step", "mp3", "flac", "mp4", "gpx", "dng", "epub"] {
            assert!(is_indexable(extension), "missing {extension}");
        }
        assert!(!is_indexable("exe"));
        assert!(!is_indexable("dll"));
    }
}
