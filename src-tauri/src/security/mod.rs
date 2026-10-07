// Security subsystem.
//
// C-82: Unicode normalization uses NFKC plus zero-width, bidi, homoglyph, and
//        mixed-script detection.
// C-143: secure_fill bound to exact scheme/host/port, frame, loader, field,
//        task, credential identity; HTTPS required; certificate errors denied;
//        punycode/homograph mismatch checked; credential identity never exposed
//        as arbitrary list.
// C-145: trusted confirmation dialogs apply spoofing hardening: forced paragraph
//        direction, bidi-control stripping, punycode display for IDN, fixed-width
//        amount rendering, page-derived strings only in visually quarantined
//        "as reported by page" zone.

pub mod unicode_tables;
pub mod idn;
pub mod trust_store;
pub mod clocks;
