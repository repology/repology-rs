// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

pub mod repository_absolute;
pub mod repository_misc;
pub mod repository_percent_comparable;
pub mod repository_percent_total;
pub mod total;

pub use repository_absolute::*;
pub use repository_misc::*;
pub use repository_percent_comparable::*;
pub use repository_percent_total::*;
pub use total::*;
