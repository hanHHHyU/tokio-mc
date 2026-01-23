pub(crate) const PREFIX_LENGTH: usize = 7;
pub(crate) const LENGTH_FIELD_SIZE: usize = 2;

pub(crate) const MC_3E_RESPONSE_PREFIX: [u8; PREFIX_LENGTH] =
    [0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00];

pub(crate) const RESPONSE_HEADER_LENGTH: usize = PREFIX_LENGTH + LENGTH_FIELD_SIZE;
pub(crate) const RESPONSE_LENGTH_FIELD_OFFSET: usize = PREFIX_LENGTH;
pub(crate) const RESPONSE_END_CODE_LENGTH: usize = 2;
pub(crate) const MIN_RESPONSE_LENGTH: usize = RESPONSE_HEADER_LENGTH + RESPONSE_END_CODE_LENGTH;

#[cfg(feature = "server")]
pub(crate) const MC_3E_REQUEST_PREFIX: [u8; PREFIX_LENGTH] =
    [0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00];

#[cfg(feature = "server")]
pub(crate) const TIMER_FIELD_SIZE: usize = 2;

#[cfg(feature = "server")]
pub(crate) const REQUEST_HEADER_LENGTH: usize =
    PREFIX_LENGTH + LENGTH_FIELD_SIZE + TIMER_FIELD_SIZE;

#[cfg(feature = "server")]
pub(crate) const REQUEST_LENGTH_FIELD_OFFSET: usize = PREFIX_LENGTH;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::{RequestHeader, ResponseHeader};

    #[test]
    fn response_header_length_matches_header_bytes() {
        let header = ResponseHeader::new();

        assert_eq!(RESPONSE_HEADER_LENGTH, header.len());
    }

    #[test]
    fn min_response_length_includes_end_code() {
        assert_eq!(
            MIN_RESPONSE_LENGTH,
            RESPONSE_HEADER_LENGTH + RESPONSE_END_CODE_LENGTH
        );
    }

    #[test]
    #[cfg(feature = "server")]
    fn request_header_length_matches_header_bytes() {
        let header = RequestHeader::new();

        assert_eq!(REQUEST_HEADER_LENGTH, header.len());
    }
}
