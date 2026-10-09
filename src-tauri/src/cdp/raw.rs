use serde_json::json;

use super::connection::CdpConnection;
use std::io::{Read, Write};

pub struct RawCdp;

impl RawCdp {
    pub fn send_command<T: Read + Write>(connection: &mut CdpConnection<T>, id: u64, method: &str, params: serde_json::Value) -> Result<serde_json::Value, crate::Error> {
        connection.send_command(id, method, params)
    }

    pub fn navigate<T: Read + Write>(connection: &mut CdpConnection<T>, id: u64, url: &str) -> Result<serde_json::Value, crate::Error> {
        if !url.starts_with("https://") && !url.starts_with("http://") { return Err(crate::Error::InvalidParameter("navigation URL must be HTTP(S)".into())); }
        Self::send_command(connection, id, "Page.navigate", json!({"url": url}))
    }

    pub fn click<T: Read + Write>(connection: &mut CdpConnection<T>, id: u64, x: f64, y: f64) -> Result<serde_json::Value, crate::Error> {
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 { return Err(crate::Error::InvalidParameter("click coordinates are invalid".into())); }
        Self::send_command(connection, id, "Input.dispatchMouseEvent", json!({"type":"mousePressed","x":x,"y":y,"button":"left","clickCount":1}))?;
        Self::send_command(connection, id + 1, "Input.dispatchMouseEvent", json!({"type":"mouseReleased","x":x,"y":y,"button":"left","clickCount":1}))
    }
}
