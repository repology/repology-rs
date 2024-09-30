// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Root<'a> {
    #[serde(borrow)]
    pub products: Vec<Product<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Product<'a> {
    #[serde(borrow)]
    pub cpe: Cpe<'a>,
}

#[derive(Deserialize, Debug)]
pub struct Cpe<'a> {
    pub deprecated: bool,
    #[serde(rename = "cpeName", borrow)]
    pub cpe_name: Cow<'a, str>,
}
