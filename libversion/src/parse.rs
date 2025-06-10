use crate::Flags;
use crate::component::*;
use crate::string::*;

#[derive(PartialEq, Debug)]
pub enum KeywordClass {
    Unknown,
    PreRelease,
    PostRelease,
}

pub fn classify_keyword(s: &str, flags: Flags) -> KeywordClass {
    if string_is_equal_to_lowercase(s, "alpha") {
        return KeywordClass::PreRelease;
    } else if string_is_equal_to_lowercase(s, "beta") {
        return KeywordClass::PreRelease;
    } else if string_is_equal_to_lowercase(s, "rc") {
        return KeywordClass::PreRelease;
    } else if string_has_prefix_lowercase(s, "pre") {
        return KeywordClass::PreRelease;
    } else if string_has_prefix_lowercase(s, "post") {
        return KeywordClass::PostRelease;
    } else if string_has_prefix_lowercase(s, "patch") {
        return KeywordClass::PostRelease;
    } else if string_is_equal_to_lowercase(s, "pl") {
        // patchlevel
        return KeywordClass::PostRelease;
    } else if string_is_equal_to_lowercase(s, "errata") {
        return KeywordClass::PostRelease;
    } else if flags.contains(Flags::PIsPatch) && string_is_equal_to_lowercase(s, "p") {
        return KeywordClass::PostRelease;
    }
    KeywordClass::Unknown
}

pub fn parse_token_to_component(input: &str, flags: Flags) -> (Component<'_>, &str) {
    let (alpha, rest) = split_alpha(input);
    if let Some(first_char) = alpha.as_bytes().first().copied() {
        (
            match classify_keyword(alpha, flags) {
                KeywordClass::Unknown => {
                    if flags.contains(Flags::AnyIsPatch) {
                        Component::PostRelease(to_lower(first_char))
                    } else {
                        Component::PreRelease(to_lower(first_char))
                    }
                }
                KeywordClass::PreRelease => Component::PreRelease(to_lower(first_char)),
                KeywordClass::PostRelease => Component::PostRelease(to_lower(first_char)),
            },
            rest,
        )
    } else {
        let (number, rest) = split_number(skip_zeroes(input));
        (
            if number.is_empty() {
                Component::Zero
            } else {
                Component::NonZero(number)
            },
            rest,
        )
    }
}

pub fn make_default_component(flags: Flags) -> Component<'static> {
    if flags.contains(Flags::LowerBound) {
        Component::LowerBound
    } else if flags.contains(Flags::UpperBound) {
        Component::UpperBound
    } else {
        Component::Zero
    }
}

pub enum SomeComponents<'a> {
    One(Component<'a>),
    Two(Component<'a>, Component<'a>),
}

pub fn get_next_version_component(s: &str, flags: Flags) -> (SomeComponents<'_>, &str) {
    let s = skip_separator(s);

    if s.is_empty() {
        return (SomeComponents::One(make_default_component(flags)), s);
    }

    let (component, rest) = parse_token_to_component(s, flags);

    let (alpha, rest_after_alpha) = split_alpha(rest);

    if let Some(first_char) = alpha.as_bytes().first().copied()
        && !rest_after_alpha
            .as_bytes()
            .first()
            .copied()
            .is_some_and(is_number)
    {
        return (
            SomeComponents::Two(
                component,
                match classify_keyword(alpha, flags) {
                    KeywordClass::Unknown => Component::LetterSuffix(to_lower(first_char)),
                    KeywordClass::PreRelease => Component::PreRelease(to_lower(first_char)),
                    KeywordClass::PostRelease => Component::PostRelease(to_lower(first_char)),
                },
            ),
            rest_after_alpha,
        );
    }

    (SomeComponents::One(component), rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_keyword() {
        assert_eq!(
            classify_keyword("ALPHA", Flags::empty()),
            KeywordClass::PreRelease
        );
        assert_eq!(
            classify_keyword("ALPHABET", Flags::empty()),
            KeywordClass::Unknown
        );
        assert_eq!(
            classify_keyword("BETA", Flags::empty()),
            KeywordClass::PreRelease
        );
        assert_eq!(
            classify_keyword("BETAKE", Flags::empty()),
            KeywordClass::Unknown
        );
        assert_eq!(
            classify_keyword("RC", Flags::empty()),
            KeywordClass::PreRelease
        );
        assert_eq!(
            classify_keyword("PRE", Flags::empty()),
            KeywordClass::PreRelease
        );
        assert_eq!(
            classify_keyword("PRERELEASE", Flags::empty()),
            KeywordClass::PreRelease
        );
        assert_eq!(
            classify_keyword("POST", Flags::empty()),
            KeywordClass::PostRelease
        );
        assert_eq!(
            classify_keyword("POSTRELEASE", Flags::empty()),
            KeywordClass::PostRelease
        );
        assert_eq!(
            classify_keyword("PATCH", Flags::empty()),
            KeywordClass::PostRelease
        );
        assert_eq!(
            classify_keyword("PATCHLEVEL", Flags::empty()),
            KeywordClass::PostRelease
        );
        assert_eq!(
            classify_keyword("PL", Flags::empty()),
            KeywordClass::PostRelease
        );
        assert_eq!(
            classify_keyword("ERRATA", Flags::empty()),
            KeywordClass::PostRelease
        );

        assert_eq!(
            classify_keyword("FOOBAR", Flags::empty()),
            KeywordClass::Unknown
        );
    }
}
