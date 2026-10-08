// QR region handling.
//
// The production agent NO LONGER decodes UPI QR codes and NO LONGER parses
// QR payloads or `upi://` URIs. That is enforced here by type and by the
// absence of any reachable decode path.
//
// What remains:
//   - QR region identification (rendered region geometry, frame/loader
//     binding);
//   - cryptographic hash of the rendered pixel region for that QR during the
//     handoff;
//   - monitoring for region/frame/loader/epoch/target changes and
//     invalidation into QR_CHANGED_REVERIFY;
//   - handoff state (QR_HANDOFF, WaitingForExternalAuth, PAYMENT_READY);
//   - native task-authority prompt using task authority only;
//   - external-auth waiting without claiming QR verification/decoding.
//
// No Touch ID is required merely to display a QR or start the dwell timer.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QrHandoffState {
    NotInHandoff,
    /// QR region identified; displayed; dwell period not yet elapsed or
    /// external auth not yet claimed by user in their UPI app.
    QrHandoff,
    /// Handoff complete from the agent's perspective; waiting for external
    /// UPI authentication in the user's UPI app.
    WaitingForExternalAuth,
    /// QR region changed while waiting; invalidate and require fresh
    /// handoff. Do not attempt to determine "better" or "equivalent".
    QrChangedReverify,
}

#[derive(Debug, Clone)]
pub struct QrRegion {
    pub region_id: String,
    pub frame_id: String,
    pub loader_id: String,
    pub epoch_value: u64,
    pub bounds: Option<crate::core::perception::graph::GeometryBounds>,
}

impl QrRegion {
    pub fn new(region_id: String, frame_id: String, loader_id: String, epoch_value: u64, bounds: Option<crate::core::perception::graph::GeometryBounds>) -> Self {
        Self { region_id, frame_id, loader_id, epoch_value, bounds }
    }

    /// Whether this region matches a given frame/loader/epoch identity.
    pub fn matches_epoch(&self, frame_id: &str, loader_id: &str, epoch_value: u64) -> bool {
        self.frame_id == frame_id && self.loader_id == loader_id && self.epoch_value == epoch_value
    }
}

/// A pixel-region hash computed over the rendered QR region for the
/// current handoff.
///
/// This is a region hash, not a QR decode. We deliberately do not extract
/// the QR payload or `upi://` URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrRegionHash {
    pub hash: [u8; 32],
    pub region_id: String,
    pub frame_id: String,
    pub loader_id: String,
    pub epoch_value: u64,
}

impl QrRegionHash {
    pub fn new(region: &QrRegion, hash: [u8; 32]) -> Self {
        Self {
            hash,
            region_id: region.region_id.clone(),
            frame_id: region.frame_id.clone(),
            loader_id: region.loader_id.clone(),
            epoch_value: region.epoch_value,
        }
    }
}

/// Handoff context while waiting for external auth.
#[derive(Debug, Clone)]
pub struct QrHandoffContext {
    pub state: QrHandoffState,
    pub region: Option<QrRegion>,
    pub region_hash_at_handoff: Option<QrRegionHash>,
    pub task_authority_summary: Option<String>,
    /// Dwell timer not yet elapsed.
    pub dwell_remaining_ms: Option<u64>,
    /// When the handoff became invalid due to region/frame/loader/epoch
    /// change.
    pub invalidated_at: Option<String>,
}

impl QrHandoffContext {
    /// Create a handoff context for a QR region using trusted task
    /// authority only. Do NOT extract payee/amount from page text/QR.
    pub fn begin(
        region: QrRegion,
        region_hash: QrRegionHash,
        task_authority_summary: String,
    ) -> Self {
        Self {
            state: QrHandoffState::QrHandoff,
            region: Some(region),
            region_hash_at_handoff: Some(region_hash),
            task_authority_summary: Some(task_authority_summary),
            dwell_remaining_ms: None,
            invalidated_at: None,
        }
    }

    /// The user-facing prompt text is composed from task authority only.
    pub fn task_authority_prompt(&self) -> Option<String> {
        self.task_authority_summary.clone()
    }
}

/// Decide whether a QR handoff must be invalidated due to a change in the
/// rendered region, frame, loader, or epoch.
///
/// If the region changed, frame changed, loader changed, or the epoch
/// changed, invalidate into QrChangedReverify.
pub fn invalidate_if_changed(
    ctx: &QrHandoffContext,
    current_region: Option<&QrRegion>,
    current_hash: Option<QrRegionHash>,
) -> QrHandoffState {
    if let Some(h) = current_hash {
        if ctx.region_hash_at_handoff.as_ref().map(|r| r.hash) != Some(h.hash) {
            return QrHandoffState::QrChangedReverify;
        }
    }
    if let Some(r) = current_region {
        if let Some(cur) = ctx.region.as_ref() {
            if !cur.matches_epoch(&r.frame_id, &r.loader_id, r.epoch_value) {
                return QrHandoffState::QrChangedReverify;
            }
        }
    }
    ctx.state.clone()
}

/// Handoff is invalid if the region hash differs or the epoch/frame/loader
/// identity differs from what was bound at handoff.
pub fn is_handoff_valid_for(
    ctx: &QrHandoffContext,
    region: &QrRegion,
    region_hash: &QrRegionHash,
) -> bool {
    ctx.region.as_ref().map(|r| r.matches_epoch(&region.frame_id, &region.loader_id, region.epoch_value)) == Some(true)
        && ctx.region_hash_at_handoff.as_ref().map(|h| h.hash == region_hash.hash) == Some(true)
}

/// The application does NOT decode QR images or parse QR payloads.
///
/// Any code path that attempts to decode a QR image or parse `upi://` is
/// either removed or unreachable in the production flow. This function exists
/// only as an explicit guard point; a call here that actually tries to decode
/// is a defect.
pub fn attempt_qr_decode_pixels(_pixels: &[u8]) -> Result<(), crate::Error> {
    Err(crate::Error::Unsupported(
        "QR decoding is disabled in the production agent".into()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    #[test]
    fn handoff_is_invalidated_on_epoch_change() {
        let region = QrRegion::new("qr-1".into(), "F1".into(), "L1".into(), 7, None);
        let hash = [1u8; 32];
        let ctx = QrHandoffContext::begin(region.clone(), QrRegionHash::new(&region, hash), "task authority".into());
        let changed = QrRegion::new("qr-1".into(), "F1".into(), "L1".into(), 8, None);
        let changed_hash = QrRegionHash::new(&changed, hash);
        assert_eq!(
            invalidate_if_changed(&ctx, Some(&changed), Some(changed_hash)),
            QrHandoffState::QrChangedReverify
        );
    }

    #[test]
    fn handoff_is_invalidated_on_region_hash_change() {
        let region = QrRegion::new("qr-1".into(), "F1".into(), "L1".into(), 7, None);
        let hash_a = [1u8; 32];
        let hash_b = [2u8; 32];
        let ctx = QrHandoffContext::begin(region.clone(), QrRegionHash::new(&region, hash_a), "task authority".into());
        let changed_hash = QrRegionHash::new(&region, hash_b);
        assert_eq!(
            invalidate_if_changed(&ctx, Some(&region), Some(changed_hash)),
            QrHandoffState::QrChangedReverify
        );
    }

    #[test]
    fn handoff_remains_valid_when_nothing_changed() {
        let region = QrRegion::new("qr-1".into(), "F1".into(), "L1".into(), 7, None);
        let hash = [1u8; 32];
        let ctx = QrHandoffContext::begin(region.clone(), QrRegionHash::new(&region, hash), "task authority".into());
        assert!(is_handoff_valid_for(&ctx, &region, &QrRegionHash::new(&region, hash)));
    }

    #[test]
    fn qr_decode_is_prohibited() {
        assert!(attempt_qr_decode_pixels(&[]).is_err());
    }

    #[test]
    fn task_authority_prompt_is_task_authority_only() {
        let region = QrRegion::new("qr-1".into(), "F1".into(), "L1".into(), 7, None);
        let hash = [1u8; 32];
        let ctx = QrHandoffContext::begin(region.clone(), QrRegionHash::new(&region, hash), "Pay merchant@payee ₹1,250.00".into());
        assert_eq!(ctx.task_authority_prompt(), Some("Pay merchant@payee ₹1,250.00".into()));
    }
}
