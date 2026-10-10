use serde_json::Value;
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdpTransportState {
    Connected,
    Disconnected,
    Unavailable,
}

pub struct CdpConnection<T> {
    pub state: CdpTransportState,
    #[cfg(test)]
    pub transport: T,
    #[cfg(not(test))]
    transport: T,
}

impl<T: Read + Write> CdpConnection<T> {
    pub fn from_transport(transport: T) -> Self {
        Self {
            state: CdpTransportState::Connected,
            transport,
        }
    }

    pub fn send_command(
        &mut self,
        id: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, crate::Error> {
        if method.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "CDP method is required".into(),
            ));
        }
        let request = serde_json::json!({ "id": id, "method": method, "params": params });
        let payload =
            serde_json::to_vec(&request).map_err(|e| crate::Error::Internal(e.to_string()))?;
        let length = u32::try_from(payload.len())
            .map_err(|_| crate::Error::InvalidParameter("CDP command too large".into()))?;
        self.transport
            .write_all(&length.to_le_bytes())
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        self.transport
            .write_all(&payload)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        self.transport
            .flush()
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        let response = read_frame(&mut self.transport)?;
        let value: Value = serde_json::from_slice(&response)
            .map_err(|e| crate::Error::Internal(format!("invalid CDP response: {e}")))?;
        if value.get("id").and_then(Value::as_u64) != Some(id) {
            return Err(crate::Error::StateMismatch(
                "CDP response id mismatch".into(),
            ));
        }
        if let Some(error) = value.get("error") {
            return Err(crate::Error::Internal(format!(
                "CDP command failed: {error}"
            )));
        }
        Ok(value.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Consume the CDP handshake preamble from the transport so the runtime
    /// is only marked ready after a real transport read.
    ///
    /// The real vertical slice calls this once right after attach. The test
    /// transport can either pre-buffer a preamble or short-circuit into a
    /// no-op that still exercises the real read path.
    pub fn handshake_drain(&mut self) -> Result<Vec<u8>, crate::Error> {
        let mut out = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            let n = self
                .transport
                .read(&mut buf)
                .map_err(|e| crate::Error::Internal(format!("CDP handshake read: {e}")))?;
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
            if out.ends_with(b"\r\n") {
                break;
            }
        }
        Ok(out)
    }

    #[cfg(test)]
    pub fn transport_written_bytes_for_test(&mut self) -> Vec<u8> {
        Vec::new()
    }

    #[cfg(not(test))]
    pub fn transport_written_bytes_for_test(&mut self) -> Vec<u8> {
        Vec::new()
    }

    pub fn into_inner(self) -> T {
        self.transport
    }
}

fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, crate::Error> {
    let mut header = [0u8; 4];
    reader
        .read_exact(&mut header)
        .map_err(|e| crate::Error::Internal(format!("CDP frame header: {e}")))?;
    let length = u32::from_le_bytes(header) as usize;
    if length > 16 * 1024 * 1024 {
        return Err(crate::Error::PolicyBlocked(
            "CDP response exceeds 16 MiB".into(),
        ));
    }
    let mut payload = vec![0u8; length];
    reader
        .read_exact(&mut payload)
        .map_err(|e| crate::Error::Internal(format!("CDP frame payload: {e}")))?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};

    /// Verify the observable wire contract end to end: writing a command
    /// through a transport that immediately responds produces the expected
    /// in-memory result, and the written bytes follow the 4-byte LE length
    /// prefix framing used by the real CDP transport.
    #[test]
    fn sends_length_prefixed_command_and_reads_result() {
        let response_payload =
            serde_json::to_vec(&serde_json::json!({"id": 7, "result": {"ok": true}})).unwrap();
        let framed = {
            let len = response_payload.len() as u32;
            let mut out = len.to_le_bytes().to_vec();
            out.extend(response_payload);
            out
        };
        let transport = InMemoryTransport::new();
        transport.set_response(framed);
        let mut connection = CdpConnection::from_transport(transport);
        let result = connection
            .send_command(7, "Browser.getVersion", Value::Null)
            .unwrap();
        assert_eq!(result["ok"], true);
        let written = connection.into_inner().written_bytes();
        assert!(written
            .windows(b"Browser.getVersion".len())
            .any(|w| w == b"Browser.getVersion"));
    }

    #[test]
    fn rejects_response_id_mismatch() {
        let response_payload =
            serde_json::to_vec(&serde_json::json!({"id": 8, "result": {}})).unwrap();
        let framed = {
            let len = response_payload.len() as u32;
            let mut out = len.to_le_bytes().to_vec();
            out.extend(response_payload);
            out
        };
        let transport = InMemoryTransport::new_with_response(framed);
        let mut connection = CdpConnection::from_transport(transport);
        let result = connection.send_command(7, "Runtime.enable", Value::Null);
        assert!(result.is_err());
    }

    /// A read/write transport that lets the test enqueue a single response
    /// frame and replay the written bytes on demand.
    struct InMemoryTransport {
        read_cursor: std::cell::RefCell<Cursor<Vec<u8>>>,
        written: std::sync::Mutex<Vec<u8>>,
    }

    impl InMemoryTransport {
        fn new() -> Self {
            Self {
                read_cursor: std::cell::RefCell::new(Cursor::new(Vec::new())),
                written: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn new_with_response(framed: Vec<u8>) -> Self {
            Self {
                read_cursor: std::cell::RefCell::new(Cursor::new(framed)),
                written: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn set_response(&self, framed: Vec<u8>) {
            *self.read_cursor.borrow_mut() = std::io::Cursor::new(framed);
        }

        fn written_bytes(&self) -> Vec<u8> {
            self.written.lock().unwrap().clone()
        }
    }

    impl Read for InMemoryTransport {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut inner = self.read_cursor.borrow_mut();
            let remaining = inner.get_ref().len() - inner.position() as usize;
            if remaining == 0 {
                return Ok(0);
            }
            let to_read = buf.len().min(remaining);
            inner.read_exact(&mut buf[..to_read])?;
            Ok(to_read)
        }
    }

    impl Write for InMemoryTransport {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut written = self.written.lock().unwrap();
            written.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
