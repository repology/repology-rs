// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(file_buffered)]
#![feature(coverage_attribute)]
#![feature(iter_collect_into)]
#![feature(const_trait_impl)]
#![feature(test)]
#![feature(trait_alias)]
#![feature(debug_closure_helpers)]

pub mod fetching;
pub mod package;
pub mod parsing;
pub mod repositories_config;
pub mod ruleset;
pub mod utils;
