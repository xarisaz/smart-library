use encoding_rs::WINDOWS_1253;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

const MAX_PDF_BYTES: u64 = 50 * 1024 * 1024;
const MAX_OFFICE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 512 * 1024;
const MAX_XML_ENTRY_BYTES: u64 = 32 * 1024 * 1024;
const MAX_SELECTED_XML_BYTES: u64 = 64 * 1024 * 1024;
const MAX_EXTRACTED_CHARACTERS: usize = 250_000;
const MIN_USEFUL_PDF_CHARACTERS: usize = 40;

#[derive(Debug)]
pub struct Extraction {
    pub text: String,
    pub status: String,
}

pub fn extract(path: &Path, extension: &str, size_bytes: u64) -> Extraction {
    match extension {
        "pdf" => extract_pdf(path, size_bytes),
        "docx" | "xlsx" | "pptx" => extract_office(path, extension, size_bytes),
        "txt" | "csv" | "json" | "xml" | "html" | "htm" | "md" | "markdown" | "yaml" | "yml"
        | "gpx" | "kml" | "nmea" => extract_plain_text(path),
        _ => Extraction {
            text: String::new(),
            status: "unsupported".to_string(),
        },
    }
}

fn extract_pdf(path: &Path, size_bytes: u64) -> Extraction {
    if size_bytes > MAX_PDF_BYTES {
        return Extraction {
            text: String::new(),
            status: "too_large".to_string(),
        };
    }

    match std::panic::catch_unwind(|| pdf_extract::extract_text(path)) {
        Ok(Ok(raw_text)) => finish_text(raw_text, "needs_ocr", MIN_USEFUL_PDF_CHARACTERS),
        Ok(Err(_)) | Err(_) => Extraction {
            text: String::new(),
            status: "failed".to_string(),
        },
    }
}

fn extract_office(path: &Path, extension: &str, size_bytes: u64) -> Extraction {
    if size_bytes > MAX_OFFICE_BYTES {
        return Extraction {
            text: String::new(),
            status: "too_large".to_string(),
        };
    }

    match std::panic::catch_unwind(|| extract_office_inner(path, extension)) {
        Ok(Ok(raw_text)) => finish_text(raw_text, "empty", 1),
        Ok(Err(_)) | Err(_) => Extraction {
            text: String::new(),
            status: "failed".to_string(),
        },
    }
}

fn extract_office_inner(path: &Path, extension: &str) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    let mut selected_entries = Vec::new();

    for index in 0..archive.len() {
        if let Some(name) = archive.name_for_index(index) {
            let normalized_name = name.replace('\\', "/");
            if is_relevant_office_xml(&normalized_name, extension) {
                selected_entries.push((normalized_name, index));
            }
        }
    }

    selected_entries.sort_by(|left, right| left.0.cmp(&right.0));
    if selected_entries.is_empty() {
        return Err("Δεν βρέθηκε αναγνώσιμο περιεχόμενο Office.".to_string());
    }

    let mut total_xml_bytes = 0u64;
    let mut extracted = String::new();
    for (_, index) in selected_entries {
        let entry = archive.by_index(index).map_err(|error| error.to_string())?;
        if entry.size() > MAX_XML_ENTRY_BYTES {
            return Err("Ένα εσωτερικό XML ξεπερνά το ασφαλές όριο.".to_string());
        }

        total_xml_bytes = total_xml_bytes.saturating_add(entry.size());
        if total_xml_bytes > MAX_SELECTED_XML_BYTES {
            return Err("Το αποσυμπιεσμένο έγγραφο ξεπερνά το ασφαλές όριο.".to_string());
        }

        let mut xml = String::with_capacity(entry.size().min(MAX_XML_ENTRY_BYTES) as usize);
        let mut limited_reader = entry.take(MAX_XML_ENTRY_BYTES + 1);
        limited_reader
            .read_to_string(&mut xml)
            .map_err(|error| error.to_string())?;
        if xml.len() as u64 > MAX_XML_ENTRY_BYTES {
            return Err("Ένα εσωτερικό XML ξεπερνά το ασφαλές όριο.".to_string());
        }

        append_office_xml_text(&xml, extension, &mut extracted)?;
        if extracted.len() >= MAX_EXTRACTED_CHARACTERS * 4 {
            break;
        }
    }

    Ok(extracted)
}

fn is_relevant_office_xml(name: &str, extension: &str) -> bool {
    if !name.ends_with(".xml") {
        return false;
    }

    let is_core_properties = name == "docProps/core.xml" || name == "docProps/app.xml";
    match extension {
        "docx" => {
            is_core_properties
                || name == "word/document.xml"
                || name.starts_with("word/header")
                || name.starts_with("word/footer")
                || matches!(
                    name,
                    "word/footnotes.xml"
                        | "word/endnotes.xml"
                        | "word/comments.xml"
                        | "word/glossary/document.xml"
                )
        }
        "xlsx" => {
            is_core_properties
                || matches!(name, "xl/sharedStrings.xml" | "xl/workbook.xml")
                || name.starts_with("xl/worksheets/")
                || name.starts_with("xl/tables/")
                || name.starts_with("xl/comments")
        }
        "pptx" => {
            is_core_properties
                || name == "ppt/presentation.xml"
                || name.starts_with("ppt/slides/slide")
                || name.starts_with("ppt/notesSlides/notesSlide")
                || name.starts_with("ppt/comments/comment")
        }
        _ => false,
    }
}

fn append_office_xml_text(xml: &str, extension: &str, output: &mut String) -> Result<(), String> {
    let document = roxmltree::Document::parse(xml).map_err(|error| error.to_string())?;

    for node in document.descendants().filter(|node| node.is_element()) {
        let local_name = node.tag_name().name();
        let include_text = match extension {
            "docx" => matches!(
                local_name,
                "t" | "instrText" | "title" | "subject" | "creator" | "keywords" | "description"
            ),
            "xlsx" => matches!(
                local_name,
                "t" | "v" | "f" | "title" | "subject" | "creator" | "keywords" | "description"
            ),
            "pptx" => matches!(
                local_name,
                "t" | "title" | "subject" | "creator" | "keywords" | "description"
            ),
            _ => false,
        };

        if include_text {
            if let Some(value) = node.text() {
                push_fragment(output, value);
            }
        }

        if extension == "xlsx" && matches!(local_name, "sheet" | "table" | "tableColumn") {
            if let Some(value) = node
                .attribute("name")
                .or_else(|| node.attribute("displayName"))
            {
                push_fragment(output, value);
            }
        }
    }

    Ok(())
}

fn push_fragment(output: &mut String, value: &str) {
    let value = value.trim();
    if value.is_empty() || output.len() >= MAX_EXTRACTED_CHARACTERS * 4 {
        return;
    }

    if !output.is_empty() {
        output.push(' ');
    }
    let remaining_bytes = (MAX_EXTRACTED_CHARACTERS * 4).saturating_sub(output.len());
    if value.len() <= remaining_bytes {
        output.push_str(value);
    } else {
        for character in value.chars() {
            if character.len_utf8() > (MAX_EXTRACTED_CHARACTERS * 4).saturating_sub(output.len()) {
                break;
            }
            output.push(character);
        }
    }
}

fn extract_plain_text(path: &Path) -> Extraction {
    match fs::read(path) {
        Ok(bytes) => {
            let capped = &bytes[..bytes.len().min(MAX_TEXT_BYTES)];
            finish_text(decode_plain_text_bytes(capped), "empty", 1)
        }
        Err(_) => Extraction {
            text: String::new(),
            status: "failed".to_string(),
        },
    }
}

pub fn decode_plain_text_bytes(bytes: &[u8]) -> String {
    if let Some(payload) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(payload).into_owned();
    }

    if let Some(payload) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return decode_utf16(payload, true);
    }

    if let Some(payload) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return decode_utf16(payload, false);
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }

    let (decoded, _, _) = WINDOWS_1253.decode(bytes);
    decoded.into_owned()
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let units = bytes.chunks_exact(2).map(|pair| {
        if little_endian {
            u16::from_le_bytes([pair[0], pair[1]])
        } else {
            u16::from_be_bytes([pair[0], pair[1]])
        }
    });
    String::from_utf16_lossy(&units.collect::<Vec<_>>())
}

fn finish_text(raw_text: String, empty_status: &str, minimum_characters: usize) -> Extraction {
    let text = normalize_and_cap(&raw_text);
    let useful_characters = text
        .chars()
        .filter(|character| character.is_alphanumeric())
        .count();
    let status = if useful_characters >= minimum_characters {
        "extracted"
    } else {
        empty_status
    };

    Extraction {
        text,
        status: status.to_string(),
    }
}

fn normalize_and_cap(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len().min(MAX_EXTRACTED_CHARACTERS));
    let mut previous_was_space = true;
    let mut character_count = 0usize;

    for character in value.chars() {
        if character_count >= MAX_EXTRACTED_CHARACTERS {
            break;
        }

        if character.is_whitespace() {
            if !previous_was_space {
                normalized.push(' ');
                previous_was_space = true;
                character_count += 1;
            }
        } else if !character.is_control() {
            normalized.push(character);
            previous_was_space = false;
            character_count += 1;
        }
    }

    normalized.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_document_whitespace() {
        let result = normalize_and_cap("  invoice\n\n  total\t42  ");
        assert_eq!(result, "invoice total 42");
    }

    #[test]
    fn reads_text_nodes_from_word_xml() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Curriculum Vitae</w:t></w:r></w:p><w:p><w:r><w:t>Work Experience</w:t></w:r></w:p></w:body></w:document>"#;
        let mut result = String::new();
        append_office_xml_text(xml, "docx", &mut result).unwrap();
        assert_eq!(result, "Curriculum Vitae Work Experience");
    }

    #[test]
    fn recognizes_only_relevant_office_parts() {
        assert!(is_relevant_office_xml("word/document.xml", "docx"));
        assert!(is_relevant_office_xml("xl/worksheets/sheet1.xml", "xlsx"));
        assert!(is_relevant_office_xml("ppt/slides/slide1.xml", "pptx"));
        assert!(!is_relevant_office_xml("word/media/image1.png", "docx"));
    }

    #[test]
    fn decodes_common_csv_encodings() {
        assert_eq!(decode_plain_text_bytes(b"name,amount"), "name,amount");
        assert_eq!(
            decode_plain_text_bytes(&[0xFF, 0xFE, 0x91, 0x03, 0x92, 0x03]),
            "ΑΒ"
        );
        assert_eq!(decode_plain_text_bytes(&[0xC1, 0xC2, 0xC3]), "ΑΒΓ");
    }
}
