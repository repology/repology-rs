// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod common;
pub mod nonexistent;

pub mod badges;
pub mod history;
pub mod information;
pub mod packages;
pub mod related;
pub mod versions;

pub use badges::*;
pub use history::*;
pub use information::*;
pub use packages::*;
pub use related::*;
pub use versions::*;
