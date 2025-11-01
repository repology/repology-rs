// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(const_trait_impl)]
#![feature(coverage_attribute)]
#![feature(debug_closure_helpers)]
#![feature(file_buffered)]
#![feature(iter_collect_into)]
#![feature(test)]
#![feature(trait_alias)]

pub mod fetching;
pub mod package;
pub mod parsing;
pub mod repositories_config;
pub mod ruleset;
pub mod utils;
