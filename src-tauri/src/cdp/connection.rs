// CDP connection.
//
// C-04: chromium launched with --remote-debugging-pipe; no TCP debugging endpoint.

pub struct CdpConnection;

impl CdpConnection {
    pub fn connect_pipe(_pipe_path: &str) -> Result<Self, crate::Error> {
        Err(crate::Error::NotImplemented("CdpConnection::connect_pipe is scheduled".into()))
    }
}
