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
#[serde(rename_all = "lowercase")]
pub enum PermissionKind {
    geolocation,
    notifications,
    clipboardRead,
    clipboardWrite,
    camera,
    microphone,
    paymentHandler,
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn default_deny_all() -> Vec<PermissionGrant> {
        vec![
            PermissionGrant { kind: PermissionKind::geolocation, granted: false },
            PermissionGrant { kind: PermissionKind::notifications, granted: false },
            PermissionGrant { kind: PermissionKind::clipboardRead, granted: false },
            PermissionGrant { kind: PermissionKind::clipboardWrite, granted: true },
            PermissionGrant { kind: PermissionKind::camera, granted: false },
            PermissionGrant { kind: PermissionKind::microphone, granted: false },
            PermissionGrant { kind: PermissionKind::paymentHandler, granted: false },
        ]
    }
}
