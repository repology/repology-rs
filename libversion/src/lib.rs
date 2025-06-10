use crate::iter::VersionComponentIterator;
use bitflags::bitflags;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

mod component;
mod iter;
mod parse;
mod string;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Flags: u32 {
        const PIsPatch   = 0b00000001;
        const AnyIsPatch = 0b00000010;
        const LowerBound = 0b00000100;
        const UpperBound = 0b00001000;
    }
}

fn version_compare4(v1: &str, v2: &str, v1_flags: Flags, v2_flags: Flags) -> std::cmp::Ordering {
    let mut v1_it = VersionComponentIterator::new(v1, v1_flags);
    let mut v2_it = VersionComponentIterator::new(v2, v2_flags);

    let mut will_need_extra_component = v1_flags.intersects(Flags::LowerBound | Flags::UpperBound)
        || v2_flags.intersects(Flags::LowerBound | Flags::UpperBound);

    loop {
        let v1_comp = v1_it.next();
        let v2_comp = v2_it.next();

        let res = v1_comp.cmp(&v2_comp);
        if res != std::cmp::Ordering::Equal {
            return res;
        }

        if v1_it.is_exhausted() && v2_it.is_exhausted() {
            if !will_need_extra_component {
                return std::cmp::Ordering::Equal;
            } else {
                will_need_extra_component = false;
            }
        }
    }
}

// AsVersionWithFlags
pub trait AsVersionWithFlags {
    fn version(&self) -> &str;
    fn flags(&self) -> Flags;
}

impl AsVersionWithFlags for &String {
    fn version(&self) -> &str {
        self
    }

    fn flags(&self) -> Flags {
        Flags::empty()
    }
}

impl AsVersionWithFlags for &str {
    fn version(&self) -> &str {
        self
    }

    fn flags(&self) -> Flags {
        Flags::empty()
    }
}

impl AsVersionWithFlags for (&String, Flags) {
    fn version(&self) -> &str {
        self.0
    }

    fn flags(&self) -> Flags {
        self.1
    }
}

impl AsVersionWithFlags for (&str, Flags) {
    fn version(&self) -> &str {
        self.0
    }

    fn flags(&self) -> Flags {
        self.1
    }
}

impl<T: AsVersionWithFlags> AsVersionWithFlags for &T {
    fn version(&self) -> &str {
        (self as &T).version()
    }

    fn flags(&self) -> Flags {
        (self as &T).flags()
    }
}

// VersionString
pub struct VersionString {
    version: String,
    flags: Flags,
}

impl VersionString {
    pub fn new(version: String, flags: Flags) -> Self {
        Self { version, flags }
    }
}

impl AsVersionWithFlags for VersionString {
    fn version(&self) -> &str {
        &self.version
    }

    fn flags(&self) -> Flags {
        self.flags
    }
}

impl<T: AsVersionWithFlags> PartialEq<T> for VersionString {
    fn eq(&self, other: &T) -> bool {
        version_compare(self, other) == Ordering::Equal
    }
}

impl Eq for VersionString {}

impl<T: AsVersionWithFlags> PartialOrd<T> for VersionString {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        Some(version_compare(self, other))
    }
}

impl Ord for VersionString {
    fn cmp(&self, other: &Self) -> Ordering {
        version_compare(self, other)
    }
}

// VersionStr
pub struct VersionStr<'a> {
    version: &'a str,
    flags: Flags,
}

impl<'a> VersionStr<'a> {
    pub fn new(version: &'a str, flags: Flags) -> VersionStr<'a> {
        Self { version, flags }
    }
}

impl AsVersionWithFlags for VersionStr<'_> {
    fn version(&self) -> &str {
        self.version
    }

    fn flags(&self) -> Flags {
        self.flags
    }
}

impl<T: AsVersionWithFlags> PartialEq<T> for VersionStr<'_> {
    fn eq(&self, other: &T) -> bool {
        version_compare(self, other) == Ordering::Equal
    }
}

impl Eq for VersionStr<'_> {}

impl<T: AsVersionWithFlags> PartialOrd<T> for VersionStr<'_> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        Some(version_compare(self, other))
    }
}

impl Ord for VersionStr<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        version_compare(self, other)
    }
}

pub fn version_compare<V1: AsVersionWithFlags, V2: AsVersionWithFlags>(
    v1: V1,
    v2: V2,
) -> std::cmp::Ordering {
    version_compare4(v1.version(), v2.version(), v1.flags(), v2.flags())
}
