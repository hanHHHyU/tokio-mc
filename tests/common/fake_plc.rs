#![cfg(feature = "server")]

use std::collections::HashMap;
use std::future;
use std::sync::{Arc, Mutex};

use tokio_mc::frame::{
    convert_to_base, find_instruction_code, split_address, ProtocolError, Request, Response,
};
use tokio_mc::server::Service;

#[derive(Debug, Default)]
pub struct FakePlc {
    d_registers: Arc<Mutex<HashMap<u32, u16>>>,
    m_bits: Arc<Mutex<HashMap<u32, bool>>>,
    x_inputs: Arc<Mutex<HashMap<u32, bool>>>,
    y_outputs: Arc<Mutex<HashMap<u32, bool>>>,
}

impl FakePlc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preset_d_register(&self, addr: u32, value: u16) {
        self.d_registers.lock().unwrap().insert(addr, value);
    }

    pub fn preset_m_bit(&self, addr: u32, value: bool) {
        self.m_bits.lock().unwrap().insert(addr, value);
    }

    pub fn preset_x_input(&self, addr: u32, value: bool) {
        self.x_inputs.lock().unwrap().insert(addr, value);
    }

    pub fn preset_y_output(&self, addr: u32, value: bool) {
        self.y_outputs.lock().unwrap().insert(addr, value);
    }

    fn parse_address(address: &str) -> Result<(&str, u32), ProtocolError> {
        let (prefix, number) = split_address(address)
            .ok_or_else(|| ProtocolError::InvalidAddress(address.to_string()))?;
        let (_, base) = find_instruction_code(prefix)
            .ok_or_else(|| ProtocolError::InvalidAddress(address.to_string()))?;
        let number = convert_to_base(number, base)
            .ok_or_else(|| ProtocolError::InvalidAddress(address.to_string()))?;
        Ok((prefix, number))
    }

    fn handle_read_u8s(&self, addr: &str, qty: u32) -> Result<Vec<u8>, ProtocolError> {
        let (prefix, start) = Self::parse_address(addr)?;
        if prefix != "D" {
            return Err(ProtocolError::InvalidAddress(addr.to_string()));
        }

        let registers = self.d_registers.lock().unwrap();
        let mut result = Vec::with_capacity(qty as usize * 2);
        for offset in 0..qty {
            let value = registers.get(&(start + offset)).copied().unwrap_or(0);
            result.extend_from_slice(&value.to_le_bytes());
        }
        Ok(result)
    }

    fn handle_write_u8s(&self, addr: &str, data: &[u8]) -> Result<(), ProtocolError> {
        let (prefix, start) = Self::parse_address(addr)?;
        if prefix != "D" {
            return Err(ProtocolError::InvalidAddress(addr.to_string()));
        }

        let mut registers = self.d_registers.lock().unwrap();
        let mut index = 0u32;
        let mut chunks = data.chunks_exact(2);
        for chunk in &mut chunks {
            let value = u16::from_le_bytes([chunk[0], chunk[1]]);
            registers.insert(start + index, value);
            index += 1;
        }
        if !chunks.remainder().is_empty() {
            let value = u16::from_le_bytes([chunks.remainder()[0], 0]);
            registers.insert(start + index, value);
        }

        Ok(())
    }

    fn handle_read_bits(&self, addr: &str, qty: u32) -> Result<Vec<bool>, ProtocolError> {
        let (prefix, start) = Self::parse_address(addr)?;
        match prefix {
            "M" => {
                let bits = self.m_bits.lock().unwrap();
                Ok((start..start + qty)
                    .map(|addr| bits.get(&addr).copied().unwrap_or(false))
                    .collect())
            }
            "X" => {
                let bits = self.x_inputs.lock().unwrap();
                Ok((start..start + qty)
                    .map(|addr| bits.get(&addr).copied().unwrap_or(false))
                    .collect())
            }
            "Y" => {
                let bits = self.y_outputs.lock().unwrap();
                Ok((start..start + qty)
                    .map(|addr| bits.get(&addr).copied().unwrap_or(false))
                    .collect())
            }
            _ => Err(ProtocolError::InvalidAddress(addr.to_string())),
        }
    }

    fn handle_write_bits(&self, addr: &str, bits: &[bool]) -> Result<(), ProtocolError> {
        let (prefix, start) = Self::parse_address(addr)?;
        match prefix {
            "M" => {
                let mut m_bits = self.m_bits.lock().unwrap();
                for (offset, value) in bits.iter().copied().enumerate() {
                    m_bits.insert(start + offset as u32, value);
                }
                Ok(())
            }
            "X" => {
                let mut x_bits = self.x_inputs.lock().unwrap();
                for (offset, value) in bits.iter().copied().enumerate() {
                    x_bits.insert(start + offset as u32, value);
                }
                Ok(())
            }
            "Y" => {
                let mut y_bits = self.y_outputs.lock().unwrap();
                for (offset, value) in bits.iter().copied().enumerate() {
                    y_bits.insert(start + offset as u32, value);
                }
                Ok(())
            }
            _ => Err(ProtocolError::InvalidAddress(addr.to_string())),
        }
    }
}

impl Service for FakePlc {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ProtocolError;
    type Future = future::Ready<Result<Response, ProtocolError>>;

    fn call(&self, req: Request<'static>) -> Self::Future {
        let result = match req {
            Request::ReadU8s(addr, qty) => self.handle_read_u8s(&addr, qty).map(Response::ReadU8s),
            Request::WriteU8s(addr, data) => self
                .handle_write_u8s(&addr, &data)
                .map(|()| Response::WriteU8s()),
            Request::ReadBits(addr, qty) => {
                self.handle_read_bits(&addr, qty).map(Response::ReadBits)
            }
            Request::WriteBits(addr, bits) => self
                .handle_write_bits(&addr, &bits)
                .map(|()| Response::WriteBits()),
        };

        future::ready(result)
    }
}
