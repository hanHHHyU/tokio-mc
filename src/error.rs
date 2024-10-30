// SPDX-FileCopyrightText: Copyright (c) 2017-2024 slowtec GmbH <post@slowtec.de>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types.

use thiserror::Error;

use crate::frame::ProtocolError;

/// Protocol or transport errors.
///
/// Devices that don't implement the _Modbus_ protocol correctly
/// or network issues can cause these errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Protocol error occurred: {0:?}")]
    Protocol(#[from] ProtocolError), // 将 ProtocolError 包装为 Protocol 错误
    #[error(transparent)]
    Transport(#[from] std::io::Error),
}
