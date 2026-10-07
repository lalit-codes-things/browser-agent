// Proxy.
//
// C-71: local proxy plus pf.
// C-73: proxy death must fail closed with zero direct egress; PAC, fallback
//        proxy, QUIC bypasses prohibited.

pub struct ProxyControl;

impl ProxyControl {
    pub fn start(_listen_addr: &str) -> Result<ProxyHandle, crate::Error> {
        Err(crate::Error::NotImplemented("ProxyControl::start is scheduled".into()))
    }

    pub fn stop(_handle: &ProxyHandle) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("ProxyControl::stop is scheduled".into()))
    }
}

pub struct ProxyHandle;
