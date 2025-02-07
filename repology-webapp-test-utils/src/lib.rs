// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]

pub mod legacy_macros;
pub mod request_response;
mod tidy;

pub use legacy_macros::*;
pub use request_response::*;
