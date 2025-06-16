// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::num::ParseIntError;

use strum::{EnumDiscriminants, EnumIter, IntoStaticStr};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseLinkStatusError {
    BadCode,
    BadErrorName,
}

impl std::fmt::Display for ParseLinkStatusError {
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

impl Error for ParseLinkStatusError {}

impl From<ParseIntError> for ParseLinkStatusError {
    fn from(_: ParseIntError) -> Self {
        Self::BadCode
    }
}

// Note: better not remove variants from this enum unless these have no
// instances in database `links` table. Otherwise, these will be converted
// into `UnknownError`s
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
#[repr(i16)]
#[strum_discriminants(derive(IntoStaticStr, EnumIter))]
pub enum LinkStatus {
    #[strum(disabled)]
    Http(u16) = 1,

    // Not checked family
    NotYetProcessed = 0,
    Skipped = -1,
    OutOfSample = -2,
    SatisfiedWithIpv6Success = -3,
    UnsupportedScheme = -4,
    ProtocolDisabled = -5,
    ProtocolDisabledForHost = -6,

    // Generic errors
    Timeout = -100,
    InvalidUrl = -101,
    Blacklisted = -102,
    UnknownError = -103,

    // DNS errors
    DnsError = -200,
    DnsDomainNotFound = -201,
    DnsNoAddressRecord = -202,
    DnsRefused = -203,
    DnsTimeout = -204,
    DnsIpv4MappedInAaaa = -205, // XXX: rename this into NonGlobalIpAddress
    NonGlobalIpAddress = -206,
    InvalidCharactersInHostname = -207,
    InvalidHostname = -208,

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
    RedirectToNonHttp = -403,

    // SSL errors
    SslError = -500,
    SslCertificateHasExpired = -501,
    SslCertificateHostnameMismatch = -502,
    SslCertificateSelfSigned = -503,
    SslCertificateSelfSignedInChain = -504,
    SslCertificateIncompleteChain = -505,
    SslHandshakeFailure = -506,
    CertificateUnknownIssuer = -507,
    InvalidCertificate = -508,
}

impl LinkStatus {
    pub fn is_success(self) -> Option<bool> {
        match self {
            Self::Http(200) => Some(true),
            _ if self.is_not_checked() => None,
            _ => Some(false),
        }
    }

    pub fn is_redirect(self) -> bool {
        matches!(self, Self::Http(code) if (300..400).contains(&code))
    }

    pub fn is_permanent_redirect(self) -> bool {
        matches!(self, Self::Http(code) if code == 301 || code == 308)
    }

    pub fn is_not_checked(self) -> bool {
        (-99..=0).contains(&self.code())
    }

    pub fn is_dns_error(self) -> bool {
        (-299..=-200).contains(&self.code())
    }

    pub fn is_connection_error(self) -> bool {
        (-399..=-300).contains(&self.code())
    }

    pub fn is_http_error(self) -> bool {
        (-499..=-400).contains(&self.code())
    }

    pub fn is_ssl_error(self) -> bool {
        (-599..=-500).contains(&self.code())
    }

    pub fn code(self) -> i16 {
        match self {
            Self::Http(code) => code as i16,
            _ => LinkStatusDiscriminants::from(self) as i16,
        }
    }

    pub fn pick_from46(ipv4: Self, ipv6: Self) -> Self {
        use LinkStatus::*;
        if ipv4 == Http(200) || ipv6 == Http(200) {
            Http(200)
        } else if ipv4.is_not_checked() {
            ipv6
        } else {
            ipv4
        }
    }
}

impl std::fmt::Display for LinkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Http(code) => {
                write!(f, "{code}")
            }
            _ => {
                let s: &'static str = LinkStatusDiscriminants::from(self).into();
                write!(f, "{s}")
            }
        }
    }
}

impl TryFrom<i16> for LinkStatus {
    type Error = ParseLinkStatusError;

    fn try_from(code: i16) -> Result<Self, Self::Error> {
        use LinkStatusDiscriminants as D;
        match code {
            code if code > 0 => Ok(Self::Http(code as u16)),

            val if val == D::NotYetProcessed as i16 => Ok(Self::NotYetProcessed),
            val if val == D::Skipped as i16 => Ok(Self::Skipped),
            val if val == D::OutOfSample as i16 => Ok(Self::OutOfSample),
            val if val == D::SatisfiedWithIpv6Success as i16 => Ok(Self::SatisfiedWithIpv6Success),
            val if val == D::UnsupportedScheme as i16 => Ok(Self::UnsupportedScheme),
            val if val == D::ProtocolDisabled as i16 => Ok(Self::ProtocolDisabled),
            val if val == D::ProtocolDisabledForHost as i16 => Ok(Self::ProtocolDisabledForHost),

            val if val == D::Timeout as i16 => Ok(Self::Timeout),
            val if val == D::InvalidUrl as i16 => Ok(Self::InvalidUrl),
            val if val == D::Blacklisted as i16 => Ok(Self::Blacklisted),
            val if val == D::UnknownError as i16 => Ok(Self::UnknownError),

            val if val == D::DnsError as i16 => Ok(Self::DnsError),
            val if val == D::DnsDomainNotFound as i16 => Ok(Self::DnsDomainNotFound),
            val if val == D::DnsNoAddressRecord as i16 => Ok(Self::DnsNoAddressRecord),
            val if val == D::DnsRefused as i16 => Ok(Self::DnsRefused),
            val if val == D::DnsTimeout as i16 => Ok(Self::DnsTimeout),
            val if val == D::DnsIpv4MappedInAaaa as i16 => Ok(Self::DnsIpv4MappedInAaaa),
            val if val == D::NonGlobalIpAddress as i16 => Ok(Self::NonGlobalIpAddress),
            val if val == D::InvalidCharactersInHostname as i16 => {
                Ok(Self::InvalidCharactersInHostname)
            }
            val if val == D::InvalidHostname as i16 => Ok(Self::InvalidHostname),

            val if val == D::ConnectionRefused as i16 => Ok(Self::ConnectionRefused),
            val if val == D::HostUnreachable as i16 => Ok(Self::HostUnreachable),
            val if val == D::ConnectionResetByPeer as i16 => Ok(Self::ConnectionResetByPeer),
            val if val == D::NetworkUnreachable as i16 => Ok(Self::NetworkUnreachable),
            val if val == D::ServerDisconnected as i16 => Ok(Self::ServerDisconnected),
            val if val == D::ConnectionAborted as i16 => Ok(Self::ConnectionAborted),
            val if val == D::AddressNotAvailable as i16 => Ok(Self::AddressNotAvailable),

            val if val == D::TooManyRedirects as i16 => Ok(Self::TooManyRedirects),
            val if val == D::BadHttp as i16 => Ok(Self::BadHttp),
            val if val == D::RedirectToNonHttp as i16 => Ok(Self::RedirectToNonHttp),

            val if val == D::SslError as i16 => Ok(Self::SslError),
            val if val == D::SslCertificateHasExpired as i16 => Ok(Self::SslCertificateHasExpired),
            val if val == D::SslCertificateHostnameMismatch as i16 => {
                Ok(Self::SslCertificateHostnameMismatch)
            }
            val if val == D::SslCertificateSelfSigned as i16 => Ok(Self::SslCertificateSelfSigned),
            val if val == D::SslCertificateSelfSignedInChain as i16 => {
                Ok(Self::SslCertificateSelfSignedInChain)
            }
            val if val == D::SslCertificateIncompleteChain as i16 => {
                Ok(Self::SslCertificateIncompleteChain)
            }
            val if val == D::SslHandshakeFailure as i16 => Ok(Self::SslHandshakeFailure),
            val if val == D::CertificateUnknownIssuer as i16 => Ok(Self::CertificateUnknownIssuer),
            val if val == D::InvalidCertificate as i16 => Ok(Self::InvalidCertificate),

            _ => Err(ParseLinkStatusError::BadCode),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkStatusWithRedirect {
    pub status: LinkStatus,
    pub redirect: Option<String>,
}

impl From<LinkStatus> for LinkStatusWithRedirect {
    fn from(status: LinkStatus) -> Self {
        Self {
            status,
            redirect: None,
        }
    }
}

impl LinkStatusWithRedirect {
    pub fn is_success(&self) -> Option<bool> {
        self.status.is_success()
    }

    pub fn code(&self) -> i16 {
        self.status.code()
    }

    pub fn redirect(&self) -> Option<&str> {
        self.redirect.as_deref()
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use strum::IntoEnumIterator;

    #[test]
    fn test_http_conversions() {
        assert_eq!(LinkStatus::try_from(200).unwrap(), LinkStatus::Http(200));
        assert_eq!(LinkStatus::Http(200).code(), 200);
        assert_eq!(LinkStatus::Http(200).to_string(), "200".to_string());
    }

    #[test]
    fn test_error_conversions() {
        for code in LinkStatusDiscriminants::iter().map(|d| d as i16) {
            let status = LinkStatus::try_from(code).unwrap();
            assert_eq!(status.code(), code);
        }
    }

    #[test]
    fn test_failing_conversions() {
        assert_eq!(
            LinkStatus::try_from(-9999),
            Err(ParseLinkStatusError::BadCode)
        );
    }

    #[test]
    fn test_is_success() {
        assert_eq!(LinkStatus::Http(200).is_success(), Some(true));
        assert_eq!(LinkStatus::Http(201).is_success(), Some(false));
        assert_eq!(LinkStatus::Http(404).is_success(), Some(false));
        assert_eq!(LinkStatus::DnsTimeout.is_success(), Some(false));
        assert_eq!(LinkStatus::Skipped.is_success(), None);
    }

    #[test]
    fn test_is_redirect() {
        assert!(!LinkStatus::Http(200).is_redirect());
        assert!(LinkStatus::Http(301).is_redirect());
        assert!(!LinkStatus::Http(404).is_redirect());
        assert!(!LinkStatus::DnsTimeout.is_redirect());
    }

    #[test]
    fn test_is_permanent_redirect() {
        assert!(LinkStatus::Http(301).is_permanent_redirect());
        assert!(LinkStatus::Http(308).is_permanent_redirect());
        assert!(!LinkStatus::Http(307).is_permanent_redirect());
        assert!(!LinkStatus::Http(404).is_permanent_redirect());
        assert!(!LinkStatus::DnsTimeout.is_permanent_redirect());
    }

    #[test]
    fn test_is() {
        assert!(LinkStatus::SslError.is_ssl_error());
        assert!(!LinkStatus::SslError.is_http_error());
    }
}
