// Raw CDP escape hatch.
//
// C-03: raw-CDP escape hatch available where required; still no arbitrary
//        model-controlled JavaScript.

pub struct RawCdp;

impl RawCdp {
    pub fn send_command(_session: &str, _command: &str) -> Result<serde_json::Value, crate::Error> {
        Err(crate::Error::NotImplemented("RawCdp::send_command is scheduled".into()))
    }
}
