use std::path::PathBuf;

pub fn documents_dir() -> PathBuf {
    choose_documents_dir(dirs::document_dir(), dirs::home_dir())
}

fn choose_documents_dir(document_dir: Option<PathBuf>, home_dir: Option<PathBuf>) -> PathBuf {
    document_dir
        .or_else(|| home_dir.map(|home| home.join("Documents")))
        .unwrap_or_else(|| PathBuf::from("Documents"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_operating_system_documents_directory() {
        let documents = PathBuf::from("/home/user/Έγγραφα");
        let home = PathBuf::from("/home/user");
        assert_eq!(
            choose_documents_dir(Some(documents.clone()), Some(home)),
            documents
        );
    }

    #[test]
    fn falls_back_to_documents_inside_the_home_directory() {
        assert_eq!(
            choose_documents_dir(None, Some(PathBuf::from("/home/user"))),
            PathBuf::from("/home/user/Documents")
        );
    }
}
