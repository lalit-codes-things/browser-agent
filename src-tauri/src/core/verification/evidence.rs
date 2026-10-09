// Independent evidence.
//
// C-93: VERIFIED_SUCCESS requires sufficiently independent evidence; page-
//        controlled banners are not independent truth.
// C-158: verification is independent of the model; model self-report of
//         success is never verification evidence.
//
// Evidence is backend-collected and typed.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub source: EvidenceSource,
    pub independent: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum EvidenceSource {
    LifecycleState,
    DomAccessibilityState,
    TargetFrameLoaderState,
    NetworkEvidence,
    TrustedRuntimeState,
    OutOfBandEvidence,
    DownloadVerification,
    PageBanner,      // not independent
    ModelSelfReport, // not independent
}

impl EvidenceSource {
    pub fn independent(&self) -> bool {
        !matches!(self, Self::PageBanner | Self::ModelSelfReport)
    }
}
