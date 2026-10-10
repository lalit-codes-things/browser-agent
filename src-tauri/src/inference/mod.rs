// Inference subsystem.
//
// C-41: SHA-256 verified at every model load.
// C-42: application never downloads a model at runtime from Hugging Face,
//        OpenAI, Anthropic, or any external service.
// C-43: Phase 1 uses provisional model; Phase 6 promotes benchmark winner into
//        models/manifests/production.json.
// C-44: model-file change gated by models/canary/behavioral-tests.json.
// C-45: Phase-1 provisional model is qwen3-1.7b Q4 GGUF (~1.1 GB) unless
//        replaced by preregistered benchmark winner.
// C-46: model/runtime memory budget <= 2.5 GB inclusive of KV cache at maximum
//        operational context and grammar buffers.
// C-47: structured inference p95 per proposal <= 2.0 seconds.
// C-146: per-call wall-clock timeout with abort callbacks; task abort cancels
//        in-flight generation.
// C-147: model-residency policy handles external memory pressure: pressure-keyed
//        unload, task parking, budgeted reload, jetsam avoidance.

pub mod install;
pub mod llama;
pub mod memory;
pub mod pin;
pub mod status;
