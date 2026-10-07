// Network subsystem.
//
// C-71: network enforcement uses local proxy plus pf; mutating requests
//        (non-GET plus navigations) pause at CDP until proxy release; reads
//        are passively observed.
// C-72: correlation key uses requestId, destination, method, body digest where
//        available; correlation miss defaults to deny.
// C-73: proxy death must fail closed with zero direct egress; PAC, fallback
//        proxy, QUIC bypasses prohibited.
// C-75: UDP/443 blocked so QUIC falls back to TCP through proxy; DoH disabled.
// C-76: proxy authority is destination/egress control only; HTTPS body/action
//        semantics come from browser instrumentation, not trusted HTTPS interception.
// C-156: agent-process egress is zero by design; browser background destinations
//        explicitly permitted per measured Chromium behavior; unexpected network
//        traffic is a first-class tracked metric and adversarial injector.

pub mod proxy;
pub mod canonical;
pub mod intercept;
pub mod classify;
pub mod tls;
pub mod pf;
