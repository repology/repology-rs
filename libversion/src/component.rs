#[derive(Debug, PartialEq, Eq)]
pub enum Component<'a> {
    LowerBound,
    PreRelease(u8),
    Zero,
    PostRelease(u8),
    NonZero(&'a str),
    LetterSuffix(u8),
    UpperBound,
}

impl Component<'_> {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl Ord for Component<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.discriminant()
            .cmp(&other.discriminant())
            .then_with(|| match (self, other) {
                (Component::PreRelease(a), Component::PreRelease(b)) => a.cmp(&b),
                (Component::PostRelease(a), Component::PostRelease(b)) => a.cmp(&b),
                (Component::NonZero(a), Component::NonZero(b)) => {
                    a.len().cmp(&b.len()).then_with(|| a.cmp(&b))
                }
                (Component::LetterSuffix(a), Component::LetterSuffix(b)) => a.cmp(&b),
                _ => std::cmp::Ordering::Equal,
            })
    }
}

impl PartialOrd for Component<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
