// Permissions.
//
// C-69: unnecessary permissions denied; sync/component-update/pings disabled
//        where compatible.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionGrant {
    pub kind: PermissionKind,
    pub granted: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionKind {
    Geolocation,
    Notifications,
    ClipboardRead,
    ClipboardWrite,
    Camera,
    Microphone,
    PaymentHandler,
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn default_deny_all() -> Vec<PermissionGrant> {
        vec![
            PermissionGrant { kind: PermissionKind::Geolocation, granted: false },
            PermissionGrant { kind: PermissionKind::Notifications, granted: false },
            PermissionGrant { kind: PermissionKind::ClipboardRead, granted: false },
            PermissionGrant { kind: PermissionKind::ClipboardWrite, granted: true },
            PermissionGrant { kind: PermissionKind::Camera, granted: false },
            PermissionGrant { kind: PermissionKind::Microphone, granted: false },
            PermissionGrant { kind: PermissionKind::PaymentHandler, granted: false },
        ]
    }
}
