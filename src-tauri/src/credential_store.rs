const CREDENTIAL_USERNAME: &str = "Smart Library";

pub fn is_supported() -> bool {
    cfg!(any(target_os = "windows", target_os = "linux"))
}

#[cfg(target_os = "windows")]
mod platform {
    use super::CREDENTIAL_USERNAME;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn set(target: &str, secret: &[u8]) -> Result<(), String> {
        if secret.len() > 2_560 {
            return Err(
                "Το ασφαλές διαπιστευτήριο είναι μεγαλύτερο από το όριο των Windows.".to_string(),
            );
        }

        let mut target_wide = wide(target);
        let mut username_wide = wide(CREDENTIAL_USERNAME);
        let mut blob = secret.to_vec();
        let mut credential: CREDENTIALW = unsafe { std::mem::zeroed() };
        credential.Type = CRED_TYPE_GENERIC;
        credential.TargetName = target_wide.as_mut_ptr();
        credential.CredentialBlobSize = blob.len() as u32;
        credential.CredentialBlob = blob.as_mut_ptr();
        credential.Persist = CRED_PERSIST_LOCAL_MACHINE;
        credential.UserName = username_wide.as_mut_ptr();

        let success = unsafe { CredWriteW(&credential, 0) };
        if success == 0 {
            return Err(format!(
                "Το Windows Credential Manager δεν αποθήκευσε το διαπιστευτήριο: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }

    pub fn get(target: &str) -> Result<Option<Vec<u8>>, String> {
        let target_wide = wide(target);
        let mut credential_ptr: *mut CREDENTIALW = null_mut();
        let success = unsafe {
            CredReadW(
                target_wide.as_ptr(),
                CRED_TYPE_GENERIC,
                0,
                &mut credential_ptr,
            )
        };
        if success == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(1_168) {
                return Ok(None);
            }
            return Err(format!(
                "Το Windows Credential Manager δεν μπόρεσε να διαβάσει το διαπιστευτήριο: {error}"
            ));
        }
        if credential_ptr.is_null() {
            return Ok(None);
        }

        let value = unsafe {
            let credential = &*credential_ptr;
            if credential.CredentialBlob.is_null() || credential.CredentialBlobSize == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(
                    credential.CredentialBlob,
                    credential.CredentialBlobSize as usize,
                )
                .to_vec()
            }
        };
        unsafe { CredFree(credential_ptr.cast::<c_void>()) };
        Ok(Some(value))
    }

    pub fn delete(target: &str) -> Result<(), String> {
        let target_wide = wide(target);
        let success = unsafe { CredDeleteW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if success == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(1_168) {
                return Ok(());
            }
            return Err(format!(
                "Το Windows Credential Manager δεν διέγραψε το διαπιστευτήριο: {error}"
            ));
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::CREDENTIAL_USERNAME;
    use keyring::{Entry, Error};

    fn entry(target: &str) -> Result<Entry, String> {
        Entry::new(CREDENTIAL_USERNAME, target)
            .map_err(|error| format!("Δεν ήταν διαθέσιμο το ασφαλές Linux keyring: {error}"))
    }

    pub fn set(target: &str, secret: &[u8]) -> Result<(), String> {
        entry(target)?
            .set_secret(secret)
            .map_err(|error| format!("Το Linux keyring δεν αποθήκευσε το διαπιστευτήριο: {error}"))
    }

    pub fn get(target: &str) -> Result<Option<Vec<u8>>, String> {
        match entry(target)?.get_secret() {
            Ok(value) => Ok(Some(value)),
            Err(Error::NoEntry) => Ok(None),
            Err(error) => Err(format!(
                "Το Linux keyring δεν μπόρεσε να διαβάσει το διαπιστευτήριο: {error}"
            )),
        }
    }

    pub fn delete(target: &str) -> Result<(), String> {
        match entry(target)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(format!(
                "Το Linux keyring δεν μπόρεσε να διαγράψει το διαπιστευτήριο: {error}"
            )),
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod platform {
    pub fn set(_target: &str, _secret: &[u8]) -> Result<(), String> {
        Err("Η ασφαλής αποθήκευση Gmail υποστηρίζεται σε Windows και Linux.".to_string())
    }

    pub fn get(_target: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }

    pub fn delete(_target: &str) -> Result<(), String> {
        Ok(())
    }
}

pub use platform::{delete, get, set};
