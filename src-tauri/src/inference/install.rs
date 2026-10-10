use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::inference::pin::ModelPin;

pub struct ModelInstall;

impl ModelInstall {
    pub fn installed_models_dir(app_data: &Path) -> Result<PathBuf, crate::Error> {
        if app_data.as_os_str().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "model app-data directory is required".into(),
            ));
        }
        Ok(app_data.join("models"))
    }

    pub fn verify_manifest_file(
        manifest: &ModelPin,
        model_path: &Path,
    ) -> Result<(), crate::Error> {
        if !model_path.is_file() {
            return Err(crate::Error::Unsupported("MODEL_UNAVAILABLE".into()));
        }
        let metadata =
            std::fs::metadata(model_path).map_err(|e| crate::Error::Internal(e.to_string()))?;
        if metadata.len() != manifest.bytes {
            return Err(crate::Error::PolicyBlocked("MODEL_SIZE_MISMATCH".into()));
        }
        let mut file =
            std::fs::File::open(model_path).map_err(|e| crate::Error::Internal(e.to_string()))?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher).map_err(|e| crate::Error::Internal(e.to_string()))?;
        let actual = format!("{:x}", hasher.finalize());
        if actual != manifest.sha256 {
            return Err(crate::Error::PolicyBlocked("MODEL_HASH_MISMATCH".into()));
        }
        Ok(())
    }

    pub fn load_path_must_be_installed_models_dir(
        path: &Path,
        installed_dir: &Path,
    ) -> Result<(), crate::Error> {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        let canonical_root = installed_dir
            .canonicalize()
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(crate::Error::PolicyBlocked(
                "MODEL_PATH_OUTSIDE_INSTALLED_DIR".into(),
            ));
        }
        Ok(())
    }
}
