// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use tidy_sys::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationStatus {
    Failed,
    Ok,
    WithWarnings,
    WithErrors,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub output: Vec<String>,
}

pub fn validate_html(html: &str) -> ValidationResult {
    let html = html.to_string() + "\0";

    let mut res = ValidationResult {
        status: ValidationStatus::Failed,
        output: vec![],
    };

    let mut buf = TidyBuffer {
        allocator: std::ptr::null_mut(),
        bp: std::ptr::null_mut(),
        size: 0,
        allocated: 0,
        next: 0,
    };

    unsafe {
        let doc = tidyCreate();
        tidySetErrorBuffer(doc, &mut buf as *mut _);

        res.status = match tidyParseString(doc, html.as_ptr() as *const _) {
            0 => ValidationStatus::Ok,
            1 => ValidationStatus::WithWarnings,
            2 => ValidationStatus::WithErrors,
            _ => ValidationStatus::Failed,
        };
        if !buf.bp.is_null() {
            res.output =
                String::from_utf8_lossy(std::slice::from_raw_parts::<u8>(buf.bp, buf.size as _))
                    .trim()
                    .lines()
                    .map(|s| s.to_owned())
                    .collect();
            tidyBufFree(&mut buf as *mut _);
        }

        tidyRelease(doc);
    }

    res
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_accepts_valid_html() {
        let valid_html = "<!DOCTYPE html><html><head><title>Test</title><body></body></html>";
        let res = validate_html(valid_html);
        assert_eq!(
            res,
            ValidationResult {
                status: ValidationStatus::Ok,
                output: vec![]
            }
        );
    }

    #[test]
    fn test_supports_modern_html() {
        let valid_html =
            "<!DOCTYPE html><html><head><title>Test</title><body><nav>foo</nav></body></html>";
        let res = validate_html(valid_html);
        assert_eq!(
            res,
            ValidationResult {
                status: ValidationStatus::Ok,
                output: vec![]
            }
        );
    }

    #[test]
    fn test_supports_unicode() {
        let valid_html = "<!DOCTYPE html><html><head><title>Test</title><body>тест</body></html>";
        let res = validate_html(valid_html);
        assert_eq!(
            res,
            ValidationResult {
                status: ValidationStatus::Ok,
                output: vec![]
            }
        );
    }

    #[test]
    fn test_invalid_empty() {
        assert_ne!(validate_html("").status, ValidationStatus::Ok);
    }

    #[test]
    fn test_invalid_no_doctype() {
        assert_ne!(
            validate_html("<html><head><title>Test</title><body>тест</body></html>").status,
            ValidationStatus::Ok
        );
    }

    #[test]
    fn test_invalid_unclosed_tag() {
        assert_ne!(
            validate_html(
                "<!DOCTYPE html><html><head><title>Test</title><body><h1>foo</body></html>"
            )
            .status,
            ValidationStatus::Ok
        );
    }
}
