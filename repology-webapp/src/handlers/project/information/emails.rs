// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use itertools::Itertools;

#[derive(Default)]
pub struct MaintainerEmailsAggregator<'a> {
    emails: HashSet<&'a str>,
}

impl<'a> MaintainerEmailsAggregator<'a> {
    pub fn add(&mut self, email: &'a str) {
        if let Some((_, domain)) = email.split_once('@')
            && domain.contains('.')
        {
            self.emails.insert(email);
        }
    }

    pub fn into_joined_addresses(self) -> Option<String> {
        if !self.emails.is_empty() {
            Some(self.emails.into_iter().sorted().join(","))
        } else {
            None
        }
    }
}
