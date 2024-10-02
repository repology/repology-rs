// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::{self, Debug, Display, Formatter};
use std::mem::take;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Part {
    Applications,
    Hardware,
    OperatingSystems,
}

impl Part {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Applications => "a",
            Self::Hardware => "h",
            Self::OperatingSystems => "o",
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Cpe {
    pub part: Part,
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub update: String,
    pub edition: String,
    pub lang: String,
    pub sw_edition: String,
    pub target_sw: String,
    pub target_hw: String,
    pub other: String,
}

impl Display for Cpe {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "cpe:2.3:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.part.as_str(),
            self.vendor,
            self.product,
            self.version,
            self.update,
            self.edition,
            self.lang,
            self.sw_edition,
            self.target_sw,
            self.target_hw,
            self.other
        )
    }
}

impl Debug for Cpe {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Cpe{{{}}}", self)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Eq)]
pub enum CpeParseError {
    InvalidPrefix,
    InvalidVersion,
    InvalidPart,
    InvalidComponentsCount(usize),
}

impl FromStr for Cpe {
    type Err = CpeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut escaped = false;
        let mut current: String = Default::default();
        let mut components: Vec<String> = Default::default();

        for ch in s.chars() {
            if escaped {
                current.push('\\');
                current.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == ':' {
                components.push(take(&mut current));
            } else {
                current.push(ch);
            }
        }
        components.push(current);

        if components.len() != 13 {
            return Err(CpeParseError::InvalidComponentsCount(components.len()));
        } else if components[0] != "cpe" {
            return Err(CpeParseError::InvalidPrefix);
        } else if components[1] != "2.3" {
            return Err(CpeParseError::InvalidVersion);
        }
        let part = match components[2].as_str() {
            "a" => Part::Applications,
            "h" => Part::Hardware,
            "o" => Part::OperatingSystems,
            _ => {
                return Err(CpeParseError::InvalidPart);
            }
        };
        Ok(Self {
            part,
            vendor: take(&mut components[3]),
            product: take(&mut components[4]),
            version: take(&mut components[5]),
            update: take(&mut components[6]),
            edition: take(&mut components[7]),
            lang: take(&mut components[8]),
            sw_edition: take(&mut components[9]),
            target_sw: take(&mut components[10]),
            target_hw: take(&mut components[11]),
            other: take(&mut components[12]),
        })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            "cpe:2.3:a:b:c:d:e:f:g:h:i:j:k".parse(),
            Ok(Cpe {
                part: Part::Applications,
                vendor: "b".into(),
                product: "c".into(),
                version: "d".into(),
                update: "e".into(),
                edition: "f".into(),
                lang: "g".into(),
                sw_edition: "h".into(),
                target_sw: "i".into(),
                target_hw: "j".into(),
                other: "k".into(),
            })
        );
    }

    #[test]
    fn test_parse_stringify() {
        let s = r"cpe:2.3:a:foo\bar\:baz:c:d:e:f:g:h:i:j:k";
        assert_eq!(Cpe::from_str(s).unwrap().to_string(), String::from(s));
    }

    #[test]
    fn test_parse_part() {
        assert_eq!(
            Cpe::from_str("cpe:2.3:a:*:*:*:*:*:*:*:*:*:*").unwrap().part,
            Part::Applications
        );
        assert_eq!(
            Cpe::from_str("cpe:2.3:h:*:*:*:*:*:*:*:*:*:*").unwrap().part,
            Part::Hardware
        );
        assert_eq!(
            Cpe::from_str("cpe:2.3:o:*:*:*:*:*:*:*:*:*:*").unwrap().part,
            Part::OperatingSystems
        );
    }

    #[test]
    fn test_parse_escaped() {
        assert_eq!(
            Cpe::from_str(r"cpe:2.3:a:foo\bar\:baz\\quux:*:*:*:*:*:*:*:*:*")
                .unwrap()
                .vendor,
            String::from(r"foo\bar\:baz\\quux")
        );
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!(
            Cpe::from_str("cpe:2.3:a::*:*:*:*:*:*:*:*:*")
                .unwrap()
                .vendor,
            String::from("")
        );
    }

    #[test]
    fn test_parse_failure_prefix() {
        assert_eq!(
            Cpe::from_str("cpx:2.3:a:*:*:*:*:*:*:*:*:*:*"),
            Err(CpeParseError::InvalidPrefix)
        );
    }

    #[test]
    fn test_parse_failure_version() {
        assert_eq!(
            Cpe::from_str("cpe:2.4:a:*:*:*:*:*:*:*:*:*:*"),
            Err(CpeParseError::InvalidVersion)
        );
    }

    #[test]
    fn test_parse_failure_part() {
        assert_eq!(
            Cpe::from_str("cpe:2.3:x:*:*:*:*:*:*:*:*:*:*"),
            Err(CpeParseError::InvalidPart)
        );
    }

    #[test]
    fn test_parse_failure_components() {
        assert_eq!(
            Cpe::from_str("cpe:2.3:x:*:*:*:*:*:*:*:*:*"),
            Err(CpeParseError::InvalidComponentsCount(12))
        );
        assert_eq!(
            Cpe::from_str("cpe:2.3:x:*:*:*:*:*:*:*:*:*:*:*"),
            Err(CpeParseError::InvalidComponentsCount(14))
        );
    }
}
