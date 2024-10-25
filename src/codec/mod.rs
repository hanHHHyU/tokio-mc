use std::{
    convert::TryFrom,
    io::{self, Cursor, Error, ErrorKind},
};

use byteorder::{BigEndian, ReadBytesExt as _};

use crate::{
    bytes::{BufMut, Bytes, BytesMut},
    frame::*,
    header::RequestHeader,
};
#[allow(clippy::cast_possible_truncation)]
fn u16_len(len: usize) -> u16 {
    // This type conversion should always be safe, because either
    // the caller is responsible to pass a valid usize or the
    // possible values are limited by the protocol.
    debug_assert!(len <= u16::MAX.into());
    len as u16
}

#[allow(clippy::cast_possible_truncation)]
fn u8_len(len: usize) -> u8 {
    // This type conversion should always be safe, because either
    // the caller is responsible to pass a valid usize or the
    // possible values are limited by the protocol.
    debug_assert!(len <= u8::MAX.into());
    len as u8
}

impl<'a> TryFrom<Request<'a>> for Bytes {
    type Error = Error;

    #[allow(clippy::panic_in_result_fn)] // Intentional unreachable!()
    fn try_from(req: Request<'a>) -> Result<Bytes, Self::Error> {
        use crate::frame::Request::*;
        let header = RequestHeader::new();
        let cnt = request_byte_count(&req, header.len());
        let mut data = BytesMut::with_capacity(cnt);
        data.put_slice(header.bytes());
        match req {
            ReadBits(address, quantity, code) | ReadWords(address, quantity, code) => {
                data.put_u16_le((address & 0xFFFF) as u16);
                data.put_u8((address >> 16) as u8); // 高位字节 |
                data.put_u8(code as u8);
                data.put_u16_le(quantity);
            }
            WriteMultipleBits(address, bits, code) => {}
            WriteMultipleWords(address, words, code) => {}
        }

        Ok(data.freeze())
    }
}

fn request_byte_count(req: &Request<'_>, header_len: usize) -> usize {
    use crate::frame::Request::*;
    match *req {
        ReadBits(_, _, _) | ReadWords(_, _, _) => header_len + 10,
        WriteMultipleBits(_, ref bits, _) => header_len + REQUEST_BYTE_LAST_LEN + bits.len(),
        WriteMultipleWords(_, ref words, _) => header_len + REQUEST_BYTE_LAST_LEN + words.len() * 2,
    }
}
