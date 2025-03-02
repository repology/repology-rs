// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod api;
mod badges;
mod graph;
mod legacy_redirects;
mod link;
mod log;
mod maintainer;
mod opensearch;
mod problems;
mod project;
mod projects;
mod repositories;
mod repository;
mod security;
mod static_files;
mod tools;
mod trivial_pages;

pub use api::*;
pub use badges::*;
pub use graph::*;
pub use legacy_redirects::*;
pub use link::*;
pub use log::*;
pub use maintainer::*;
pub use opensearch::*;
pub use project::*;
pub use projects::*;
pub use repositories::*;
pub use repository::*;
pub use security::*;
pub use static_files::*;
pub use tools::*;
pub use trivial_pages::*;
