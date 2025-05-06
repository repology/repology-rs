// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::num::ParseIntError;
use std::str::FromStr;

use hickory_resolver::ResolveError;
use serde::Deserialize;
use strum::{EnumDiscriminants, EnumIter, IntoStaticStr};
use tracing::error;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseHttpStatusError {
    BadCode,
    BadErrorName,
}

impl std::fmt::Display for ParseHttpStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::BadCode => "Bad HTTP status code",
                Self::BadErrorName => "Bad HTTP error name",
            }
        )
    }
}

impl Error for ParseHttpStatusError {}

impl From<ParseIntError> for ParseHttpStatusError {
    fn from(_: ParseIntError) -> Self {
        Self::BadCode
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
#[repr(i16)]
#[strum_discriminants(derive(IntoStaticStr, EnumIter))]
pub enum HttpStatus {
    #[strum(disabled)]
    Http(u16) = 0,

    // Generic errors
    UnknownError = -1,
    Timeout = -100,
    InvalidUrl = -101,
    Blacklisted = -102,

    // DNS errors
    DnsError = -200,
    DnsDomainNotFound = -201,
    DnsNoAddressRecord = -202,
    DnsRefused = -203,
    DnsTimeout = -204,
    DnsIpv4MappedInAaaa = -205, // XXX: rename this into NonGlobalIpAddress

    // Connection errors
    ConnectionRefused = -300,
    HostUnreachable = -301,
    ConnectionResetByPeer = -302,
    NetworkUnreachable = -303,
    ServerDisconnected = -304,
    ConnectionAborted = -306,
    AddressNotAvailable = -307,

    // HTTP errors
    TooManyRedirects = -400,
    BadHttp = -402,

    // SSL errors
    SslError = -500,
    SslCertificateHasExpired = -501,
    SslCertificateHostnameMismatch = -502,
    SslCertificateSelfSigned = -503,
    SslCertificateSelfSignedInChain = -504,
    SslCertificateIncompleteChain = -505,
}

impl HttpStatus {
    pub fn is_success(self) -> bool {
        self == Self::Http(200)
    }

    pub fn is_redirect(self) -> bool {
        matches!(self, Self::Http(code) if (300..400).contains(&code))
    }

    pub fn is_permanent_redirect(self) -> bool {
        matches!(self, Self::Http(code) if code == 301 || code == 308)
    }

    pub fn code(self) -> i16 {
        match self {
            Self::Http(code) => code as i16,
            _ => HttpStatusDiscriminants::from(self) as i16,
        }
    }

    pub fn from_code(code: i16) -> Result<Self, ParseHttpStatusError> {
        match code {
            code if code >= 0 => Ok(Self::Http(code as u16)),

            val if val == HttpStatusDiscriminants::UnknownError as i16 => Ok(Self::UnknownError),
            val if val == HttpStatusDiscriminants::Timeout as i16 => Ok(Self::Timeout),
            val if val == HttpStatusDiscriminants::InvalidUrl as i16 => Ok(Self::InvalidUrl),
            val if val == HttpStatusDiscriminants::Blacklisted as i16 => Ok(Self::Blacklisted),

            val if val == HttpStatusDiscriminants::DnsError as i16 => Ok(Self::DnsError),
            val if val == HttpStatusDiscriminants::DnsDomainNotFound as i16 => {
                Ok(Self::DnsDomainNotFound)
            }
            val if val == HttpStatusDiscriminants::DnsNoAddressRecord as i16 => {
                Ok(Self::DnsNoAddressRecord)
            }
            val if val == HttpStatusDiscriminants::DnsRefused as i16 => Ok(Self::DnsRefused),
            val if val == HttpStatusDiscriminants::DnsTimeout as i16 => Ok(Self::DnsTimeout),
            val if val == HttpStatusDiscriminants::DnsIpv4MappedInAaaa as i16 => {
                Ok(Self::DnsIpv4MappedInAaaa)
            }

            val if val == HttpStatusDiscriminants::ConnectionRefused as i16 => {
                Ok(Self::ConnectionRefused)
            }
            val if val == HttpStatusDiscriminants::HostUnreachable as i16 => {
                Ok(Self::HostUnreachable)
            }
            val if val == HttpStatusDiscriminants::ConnectionResetByPeer as i16 => {
                Ok(Self::ConnectionResetByPeer)
            }
            val if val == HttpStatusDiscriminants::NetworkUnreachable as i16 => {
                Ok(Self::NetworkUnreachable)
            }
            val if val == HttpStatusDiscriminants::ServerDisconnected as i16 => {
                Ok(Self::ServerDisconnected)
            }
            val if val == HttpStatusDiscriminants::ConnectionAborted as i16 => {
                Ok(Self::ConnectionAborted)
            }
            val if val == HttpStatusDiscriminants::AddressNotAvailable as i16 => {
                Ok(Self::AddressNotAvailable)
            }

            val if val == HttpStatusDiscriminants::TooManyRedirects as i16 => {
                Ok(Self::TooManyRedirects)
            }
            val if val == HttpStatusDiscriminants::BadHttp as i16 => Ok(Self::BadHttp),

            val if val == HttpStatusDiscriminants::SslError as i16 => Ok(Self::SslError),
            val if val == HttpStatusDiscriminants::SslCertificateHasExpired as i16 => {
                Ok(Self::SslCertificateHasExpired)
            }
            val if val == HttpStatusDiscriminants::SslCertificateHostnameMismatch as i16 => {
                Ok(Self::SslCertificateHostnameMismatch)
            }
            val if val == HttpStatusDiscriminants::SslCertificateSelfSigned as i16 => {
                Ok(Self::SslCertificateSelfSigned)
            }
            val if val == HttpStatusDiscriminants::SslCertificateSelfSignedInChain as i16 => {
                Ok(Self::SslCertificateSelfSignedInChain)
            }
            val if val == HttpStatusDiscriminants::SslCertificateIncompleteChain as i16 => {
                Ok(Self::SslCertificateIncompleteChain)
            }

            _ => Err(ParseHttpStatusError::BadCode),
        }
    }

    pub fn from_code_with_fallback(code: i16) -> Self {
        Self::from_code(code).unwrap_or_else(|_| {
            error!(code, "unknown http status code");
            Self::UnknownError
        })
    }

    pub fn from_error_name(name: &str) -> Result<Self, ParseHttpStatusError> {
        match name {
            "UnknownError" => Ok(Self::UnknownError),
            "Timeout" => Ok(Self::Timeout),
            "InvalidUrl" => Ok(Self::InvalidUrl),
            "Blacklisted" => Ok(Self::Blacklisted),

            "DnsError" => Ok(Self::DnsError),
            "DnsDomainNotFound" => Ok(Self::DnsDomainNotFound),
            "DnsNoAddressRecord" => Ok(Self::DnsNoAddressRecord),
            "DnsRefused" => Ok(Self::DnsRefused),
            "DnsTimeout" => Ok(Self::DnsTimeout),
            "DnsIpv4MappedInAaaa" => Ok(Self::DnsIpv4MappedInAaaa),

            "ConnectionRefused" => Ok(Self::ConnectionRefused),
            "HostUnreachable" => Ok(Self::HostUnreachable),
            "ConnectionResetByPeer" => Ok(Self::ConnectionResetByPeer),
            "NetworkUnreachable" => Ok(Self::NetworkUnreachable),
            "ServerDisconnected" => Ok(Self::ServerDisconnected),
            "ConnectionAborted" => Ok(Self::ConnectionAborted),
            "AddressNotAvailable" => Ok(Self::AddressNotAvailable),

            "TooManyRedirects" => Ok(Self::TooManyRedirects),
            "BadHttp" => Ok(Self::BadHttp),

            "SslError" => Ok(Self::SslError),
            "SslCertificateHasExpired" => Ok(Self::SslCertificateHasExpired),
            "SslCertificateHostnameMismatch" => Ok(Self::SslCertificateHostnameMismatch),
            "SslCertificateSelfSigned" => Ok(Self::SslCertificateSelfSigned),
            "SslCertificateSelfSignedInChain" => Ok(Self::SslCertificateSelfSignedInChain),
            "SslCertificateIncompleteChain" => Ok(Self::SslCertificateIncompleteChain),

            _ => Err(ParseHttpStatusError::BadErrorName),
        }
    }

    pub fn from_error_name_with_fallback(name: &str) -> Self {
        Self::from_error_name(name).unwrap_or_else(|_| {
            error!(name, "unknown http status error name");
            Self::UnknownError
        })
    }

    #[tracing::instrument]
    pub fn from_resolve_error(err: &ResolveError) -> Self {
        use hickory_resolver::ResolveErrorKind;
        use hickory_resolver::proto::ProtoErrorKind;
        use hickory_resolver::proto::op::response_code::ResponseCode;

        let ResolveErrorKind::Proto(err) = err.kind() else {
            error!("no specific handling for this resolve error ResolveErrorKind");
            return Self::DnsError;
        };

        match err.kind() {
            ProtoErrorKind::NoRecordsFound { response_code, .. }
                if *response_code == ResponseCode::ServFail =>
            {
                Self::DnsError
            }
            ProtoErrorKind::NoRecordsFound { response_code, .. }
                if *response_code == ResponseCode::NXDomain =>
            {
                Self::DnsDomainNotFound
            }
            ProtoErrorKind::NoRecordsFound { response_code, .. }
                if *response_code == ResponseCode::NoError =>
            {
                Self::DnsNoAddressRecord
            }
            ProtoErrorKind::Timeout => Self::DnsTimeout,
            _ => {
                error!("no specific handling for this resolve error ProtoErrorKind");
                Self::DnsError
            }
        }
    }

    pub fn pick_from46(ipv4: Option<Self>, ipv6: Option<Self>) -> Option<Self> {
        if ipv6.is_some_and(|status| status != HttpStatus::DnsNoAddressRecord) {
            ipv6
        } else {
            ipv4
        }
    }
}

impl std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Http(code) => {
                write!(f, "{code}")
            }
            _ => {
                let s: &'static str = HttpStatusDiscriminants::from(self).into();
                write!(f, "{s}")
            }
        }
    }
}

impl FromStr for HttpStatus {
    type Err = ParseHttpStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_empty() && s.chars().all(|ch| ch.is_ascii_digit()) {
            let http_code: u16 = s.parse()?;
            Ok(Self::Http(http_code))
        } else {
            Self::from_error_name(s)
        }
    }
}

impl<'de> Deserialize<'de> for HttpStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpStatusWithRedirect {
    pub status: HttpStatus,
    pub redirect: Option<String>,
}

impl From<HttpStatus> for HttpStatusWithRedirect {
    fn from(status: HttpStatus) -> Self {
        Self {
            status,
            redirect: None,
        }
    }
}

impl HttpStatusWithRedirect {
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn code(&self) -> i16 {
        self.status.code()
    }

    pub fn redirect(&self) -> Option<&str> {
        self.redirect.as_deref()
    }
}

impl FromStr for HttpStatusWithRedirect {
    type Err = ParseHttpStatusError;

    fn from_str(status: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            status: status.parse()?,
            redirect: None,
        })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use strum::IntoEnumIterator;

    #[test]
    fn test_http_conversions() {
        assert_eq!(HttpStatus::from_code(200).unwrap(), HttpStatus::Http(200));
        assert_eq!(HttpStatus::from_str("200").unwrap(), HttpStatus::Http(200));
        assert_eq!(HttpStatus::Http(200).code(), 200);
        assert_eq!(HttpStatus::Http(200).to_string(), "200".to_string());
    }

    #[test]
    fn test_error_conversions() {
        for code in HttpStatusDiscriminants::iter().map(|d| d as i16) {
            let status = HttpStatus::from_code(code).unwrap();
            assert_eq!(status.code(), code);
            assert_eq!(HttpStatus::from_str(&status.to_string()).unwrap(), status);
        }
    }

    #[test]
    fn test_failing_conversions() {
        assert_eq!(
            HttpStatus::from_code(-9999),
            Err(ParseHttpStatusError::BadCode)
        );
        assert_eq!(
            HttpStatus::from_error_name("FooBar"),
            Err(ParseHttpStatusError::BadErrorName)
        );
        assert_eq!(
            HttpStatus::from_str("FooBar"),
            Err(ParseHttpStatusError::BadErrorName)
        );
        assert_eq!(
            HttpStatus::from_str("9999999"),
            Err(ParseHttpStatusError::BadCode)
        );
    }

    #[test]
    fn test_is_success() {
        assert!(HttpStatus::Http(200).is_success());
        assert!(!HttpStatus::Http(201).is_success());
        assert!(!HttpStatus::Http(404).is_success());
        assert!(!HttpStatus::DnsTimeout.is_success());
    }

    #[test]
    fn test_is_redirect() {
        assert!(!HttpStatus::Http(200).is_redirect());
        assert!(HttpStatus::Http(301).is_redirect());
        assert!(!HttpStatus::Http(404).is_redirect());
        assert!(!HttpStatus::DnsTimeout.is_redirect());
    }

    #[test]
    fn test_is_permanent_redirect() {
        assert!(HttpStatus::Http(301).is_permanent_redirect());
        assert!(HttpStatus::Http(308).is_permanent_redirect());
        assert!(!HttpStatus::Http(307).is_permanent_redirect());
        assert!(!HttpStatus::Http(404).is_permanent_redirect());
        assert!(!HttpStatus::DnsTimeout.is_permanent_redirect());
    }
}
