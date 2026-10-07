// Install.
//
// C-39: installed models copied at install time to
//        ~/Library/Application Support/<app>/models/.
// C-40: runtime loads models only from the app-data models directory, never
//        from the Git checkout.
// C-42: application never downloads a model at runtime from Hugging Face,
//        OpenAI, Anthropic, or any external service.

pub struct ModelInstall;

impl ModelInstall {
    pub fn installed_models_dir() -> Result<String, crate::Error> {
        // Placeholder for app-data models directory path.
        Err(crate::Error::NotImplemented("ModelInstall::installed_models_dir is scheduled".into()))
    }

    pub fn copy_from_bundle_to_installed(_bundle_path: &str) -> Result<String, crate::Error> {
        Err(crate::Error::NotImplemented("ModelInstall::copy_from_bundle_to_installed is scheduled".into()))
    }

    pub fn load_path_must_be_installed_models_dir(_path: &str) -> Result<(), crate::Error> {
        // Runtime load path must not be the git checkout.
        Err(crate::Error::NotImplemented("ModelInstall::load_path_must_be_installed_models_dir is scheduled".into()))
    }
}
