// Browser process.
//
// C-04, C-05, C-68: managed Chromium process, pipe-based debugging, one
//        high-risk process maximum, sandbox/site isolation enabled.

pub struct BrowserProcess {
    pub pid: Option<u32>,
    pub kind: ProcessKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessKind {
    HighRisk,
    LowRisk,
}

pub struct ProcessManager;

impl ProcessManager {
    pub fn launch_piped_chromium(_args: &[String]) -> Result<BrowserProcess, crate::Error> {
        Err(crate::Error::NotImplemented("ProcessManager::launch_piped_chromium is scheduled".into()))
    }

    pub fn shutdown_gracefully(_process: &BrowserProcess) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("ProcessManager::shutdown_gracefully is scheduled".into()))
    }
}
