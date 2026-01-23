use std::fmt::Debug;

use crate::frame::{convert_keyence_to_mitsubishi_address, Model};
use crate::Error;

pub(crate) trait AddressConverter: Debug + Send + Sync {
    fn convert(&self, addr: &str) -> Result<String, Error>;
}

#[derive(Debug, Default)]
pub(crate) struct MitsubishiAddressConverter;

impl AddressConverter for MitsubishiAddressConverter {
    fn convert(&self, addr: &str) -> Result<String, Error> {
        Ok(addr.to_string())
    }
}

#[derive(Debug, Default)]
pub(crate) struct KeyenceAddressConverter;

impl AddressConverter for KeyenceAddressConverter {
    fn convert(&self, addr: &str) -> Result<String, Error> {
        convert_keyence_to_mitsubishi_address(addr).map_err(Error::from)
    }
}

pub(crate) fn converter_for_model(model: Model) -> Box<dyn AddressConverter> {
    match model {
        Model::Mitsubishi => Box::new(MitsubishiAddressConverter::default()),
        Model::Keyence => Box::new(KeyenceAddressConverter::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converter_for_mitsubishi_returns_original_address() {
        let converter = converter_for_model(Model::Mitsubishi);

        let result = converter.convert("D100").expect("conversion failed");

        assert_eq!(result, "D100");
    }

    #[test]
    fn converter_for_keyence_converts_dm_address() {
        let converter = converter_for_model(Model::Keyence);

        let result = converter.convert("DM100").expect("conversion failed");

        assert_eq!(result, "D100");
    }

    #[test]
    fn keyence_converter_invalid_address_returns_kv_error() {
        let converter = converter_for_model(Model::Keyence);

        let err = converter
            .convert("INVALID")
            .expect_err("expected conversion to fail");

        assert!(matches!(err, Error::KV(_)));
    }
}
