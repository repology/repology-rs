// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use itertools::Itertools;
use std::collections::HashMap;

pub enum Precedence {
    Greatest,
    Lowest,
}

pub struct Item<Group, Payload> {
    pub group: Group,
    pub payload: Payload,
    pub value: f64,
}

pub struct Top<Group, Payload> {
    size: usize,
    precedence: Precedence,
    groups: HashMap<Group, (Payload, f64)>,
}

impl<Group, Payload> Top<Group, Payload>
where
    Group: std::cmp::Eq + std::hash::Hash,
{
    pub fn new(size: usize, precedence: Precedence) -> Self {
        Self {
            size,
            precedence,
            groups: Default::default(),
        }
    }

    pub fn add(&mut self, group: Group, payload: Payload, value: f64) {
        if let Some(item) = self.groups.get_mut(&group) {
            if match self.precedence {
                Precedence::Greatest => value > item.1,
                Precedence::Lowest => value < item.1,
            } {
                *item = (payload, value);
            }
        } else {
            self.groups.insert(group, (payload, value));
        }
    }

    pub fn get(self) -> impl Iterator<Item = Item<Group, Payload>> {
        self.groups
            .into_iter()
            .sorted_by(|(_, (_, a)), (_, (_, b))| match self.precedence {
                Precedence::Greatest => a.total_cmp(b).reverse(),
                Precedence::Lowest => a.total_cmp(b),
            })
            .take(self.size)
            .map(|(group, (payload, value))| Item::<Group, Payload> {
                group,
                payload,
                value,
            })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_lowest() {
        let mut top = Top::new(2, Precedence::Lowest);
        top.add("foo", "small", 1.0);
        top.add("foo", "big", 2.0);
        top.add("bar", "big", 4.0);
        top.add("bar", "small", 3.0);
        top.add("baz", "small", 5.0);
        top.add("baz", "big", 6.0);
        assert_eq!(
            top.get()
                .map(|item| (item.group, item.payload))
                .collect_vec(),
            vec![("foo", "small"), ("bar", "small"),]
        );
    }

    #[test]
    fn test_greatest() {
        let mut top = Top::new(2, Precedence::Greatest);
        top.add("foo", "small", 1.0);
        top.add("foo", "big", 2.0);
        top.add("bar", "big", 4.0);
        top.add("bar", "small", 3.0);
        top.add("baz", "small", 5.0);
        top.add("baz", "big", 6.0);
        assert_eq!(
            top.get()
                .map(|item| (item.group, item.payload))
                .collect_vec(),
            vec![("baz", "big",), ("bar", "big",),]
        );
    }
}
