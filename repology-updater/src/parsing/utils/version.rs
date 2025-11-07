// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

const MAX_STRIPS: usize = 10;

enum VersionStripKind {
    Left,
    LeftGreedy,
    Right,
    RightGreedy,
}

pub struct VersionStripper {
    // would be nice to just use Vector here, but
    // we would also like to have constness
    strips: [Option<(VersionStripKind, &'static str)>; MAX_STRIPS],
    len: usize,
}

impl VersionStripper {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            strips: [const { None }; MAX_STRIPS],
            len: 0,
        }
    }

    #[cfg_attr(not(test), expect(unused))] // will be used in parsers
    pub const fn with_strip_left(mut self, separator: &'static str) -> Self {
        self.strips[self.len] = Some((VersionStripKind::Left, separator));
        self.len += 1;
        self
    }

    #[cfg_attr(not(test), expect(unused))] // will be used in parsers
    pub const fn with_strip_left_greedy(mut self, separator: &'static str) -> Self {
        self.strips[self.len] = Some((VersionStripKind::LeftGreedy, separator));
        self.len += 1;
        self
    }

    pub const fn with_strip_right(mut self, separator: &'static str) -> Self {
        self.strips[self.len] = Some((VersionStripKind::Right, separator));
        self.len += 1;
        self
    }

    #[cfg_attr(not(test), expect(unused))] // will be used in parsers
    pub const fn with_strip_right_greedy(mut self, separator: &'static str) -> Self {
        self.strips[self.len] = Some((VersionStripKind::RightGreedy, separator));
        self.len += 1;
        self
    }

    pub fn apply<'a>(&self, version: &'a str) -> &'a str {
        let mut version = version;

        for strip in self.strips[0..self.len]
            .iter()
            .map(|strip| strip.as_ref().unwrap())
        {
            version = match strip.0 {
                VersionStripKind::Left => version
                    .split_once(strip.1)
                    .map(|(_, right)| right)
                    .unwrap_or(version),
                VersionStripKind::LeftGreedy => version
                    .rsplit_once(strip.1)
                    .map(|(_, right)| right)
                    .unwrap_or(version),
                VersionStripKind::Right => version
                    .rsplit_once(strip.1)
                    .map(|(left, _)| left)
                    .unwrap_or(version),
                VersionStripKind::RightGreedy => version
                    .split_once(strip.1)
                    .map(|(left, _)| left)
                    .unwrap_or(version),
            };
        }

        version
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_version_stripper_basic() {
        let stripper = VersionStripper::new().with_strip_left(",");
        assert_eq!(stripper.apply("123"), "123");
        assert_eq!(stripper.apply("1,2"), "2");
        assert_eq!(stripper.apply("1,2,3"), "2,3");

        let stripper = VersionStripper::new().with_strip_left_greedy(",");
        assert_eq!(stripper.apply("123"), "123");
        assert_eq!(stripper.apply("1,2"), "2");
        assert_eq!(stripper.apply("1,2,3"), "3");

        let stripper = VersionStripper::new().with_strip_right(",");
        assert_eq!(stripper.apply("123"), "123");
        assert_eq!(stripper.apply("1,2"), "1");
        assert_eq!(stripper.apply("1,2,3"), "1,2");

        let stripper = VersionStripper::new().with_strip_right_greedy(",");
        assert_eq!(stripper.apply("123"), "123");
        assert_eq!(stripper.apply("1,2"), "1");
        assert_eq!(stripper.apply("1,2,3"), "1");
    }

    #[test]
    fn test_version_stripper_order() {
        let stripper = VersionStripper::new()
            .with_strip_left(",")
            .with_strip_left(".");
        assert_eq!(stripper.apply("1,2.3,4.5"), "3,4.5");

        let stripper = VersionStripper::new()
            .with_strip_left(".")
            .with_strip_left(",");
        assert_eq!(stripper.apply("1,2.3,4.5"), "4.5");
    }

    #[test]
    fn test_version_stripper_const() {
        const STRIPPER: VersionStripper = VersionStripper::new()
            .with_strip_left(",")
            .with_strip_right(":");
        assert_eq!(STRIPPER.apply("1,2:3"), "2");
    }
}
