// llama.cpp / Metal inference interface.
//
// C-46: model/runtime memory budget <= 2.5 GB inclusive of KV cache at max
//        operational context and grammar buffers.
// C-47: structured inference p95 per proposal <= 2.0 seconds.
// C-146: per-call wall-clock timeout with abort callbacks; task abort cancels
//        in-flight generation.

use crate::core::reasoning::schema::ModelAction;
use crate::core::reasoning::abort::AbortSignal;
use crate::core::reasoning::timeout::InferenceTimeout;

pub struct LlamaSession {
    pub model_path: String,
    pub kv_cache_bytes: u64,
}

pub struct InferenceEngine;

impl InferenceEngine {
    pub fn load_from_installed_models_dir(_path: &str, _expected_sha256: &str) -> Result<LlamaSession, crate::Error> {
        Err(crate::Error::NotImplemented("InferenceEngine::load_from_installed_models_dir is scheduled".into()))
    }

    pub fn generate(
        &self,
        _session: &LlamaSession,
        _abort: &AbortSignal,
        _timeout: &InferenceTimeout,
        _prompt: &str,
    ) -> Result<ModelAction, crate::Error> {
        Err(crate::Error::NotImplemented("InferenceEngine::generate is scheduled".into()))
    }
}
