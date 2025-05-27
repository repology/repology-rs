// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use tracing::error;

use repology_common::LinkStatus;

struct StatusChooser {
    status: LinkStatus,
}

impl StatusChooser {
    fn new() -> Self {
        Self {
            status: LinkStatus::UnknownError,
        }
    }

    fn get_status_precision(status: LinkStatus) -> u8 {
        use LinkStatus::*;
        match status {
            NotYetProcessed
            | Skipped
            | OutOfSample
            | SatisfiedWithIpv6Success
            | UnsupportedScheme
            | ProtocolDisabled
            | ProtocolDisabledForHost => {
                unreachable!("skipped status should not be encountered in status resolution");
            }

            // generic error categories
            DnsError => 1,
            SslError => 1,
            BadHttp => 1,
            UnknownError => 0,

            // precise errors
            Http(..) => 2,
            Timeout => 2,
            InvalidUrl => 2,
            Blacklisted => 2,
            DnsDomainNotFound => 2,
            DnsNoAddressRecord => 2,
            DnsRefused => 2,
            DnsTimeout => 2,
            DnsIpv4MappedInAaaa => 2,
            NonGlobalIpAddress => 2,
            InvalidCharactersInHostname => 2,
            ConnectionRefused => 2,
            HostUnreachable => 2,
            ConnectionResetByPeer => 2,
            NetworkUnreachable => 2,
            ServerDisconnected => 2,
            ConnectionAborted => 2,
            AddressNotAvailable => 2,
            TooManyRedirects => 2,
            SslCertificateHasExpired => 2,
            SslCertificateHostnameMismatch => 2,
            SslCertificateSelfSigned => 2,
            SslCertificateSelfSignedInChain => 2,
            SslCertificateIncompleteChain => 2,
            SslHandshakeFailure => 2,
        }
    }

    fn push(&mut self, status: LinkStatus) {
        let precision = Self::get_status_precision(status);
        let self_precision = Self::get_status_precision(self.status);

        if precision > self_precision {
            self.status = status
        } else if precision == self_precision && status != self.status {
            error!(status = ?self.status, another_status = ?status, "conflicting statuses of same precision")
        }
    }

    fn get(&self) -> LinkStatus {
        self.status
    }
}

trait ExtractStatus {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str);
}

impl ExtractStatus for &(dyn std::error::Error + 'static) {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        extract_status_generic(*self, chooser, url);
    }
}

impl ExtractStatus for &(dyn std::error::Error + Send + Sync + 'static) {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        extract_status_generic(*self, chooser, url);
    }
}

fn extract_status_generic(
    error: &(dyn std::error::Error + 'static),
    chooser: &mut StatusChooser,
    url: &str,
) {
    if let Some(source) = error.source() {
        source.extract_status(chooser, url);
    }

    error
        .downcast_ref::<h2::Error>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<hickory_resolver::ResolveError>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<hyper::Error>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<reqwest::Error>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<rustls::Error>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<rustls::OtherError>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<std::io::Error>()
        .inspect(|error| error.extract_status(chooser, url));
    error
        .downcast_ref::<webpki::Error>()
        .inspect(|error| error.extract_status(chooser, url));
}

impl ExtractStatus for h2::Error {
    fn extract_status(&self, chooser: &mut StatusChooser, _url: &str) {
        if self.is_reset() {
            chooser.push(LinkStatus::ServerDisconnected);
        }
    }
}

impl ExtractStatus for hickory_resolver::ResolveError {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        use hickory_resolver::ResolveErrorKind;
        use hickory_resolver::proto::ProtoErrorKind;
        use hickory_resolver::proto::op::response_code::ResponseCode;

        chooser.push(LinkStatus::DnsError);

        let ResolveErrorKind::Proto(proto_error) = self.kind() else {
            error!(error = ?self, url, "unhandled hickory_resolver::ResolveErrorKind variant");
            return;
        };

        match proto_error.kind() {
            ProtoErrorKind::NoRecordsFound { response_code, .. }
                if *response_code == ResponseCode::ServFail =>
            {
                chooser.push(LinkStatus::DnsError);
            }
            ProtoErrorKind::NoRecordsFound { response_code, .. }
                if *response_code == ResponseCode::NXDomain =>
            {
                chooser.push(LinkStatus::DnsDomainNotFound);
            }
            ProtoErrorKind::NoRecordsFound { response_code, .. }
                if *response_code == ResponseCode::NoError =>
            {
                chooser.push(LinkStatus::DnsNoAddressRecord);
            }
            ProtoErrorKind::Timeout => {
                chooser.push(LinkStatus::DnsTimeout);
            }
            ProtoErrorKind::Msg(message)
                if message.starts_with("Label contains invalid characters") =>
            {
                chooser.push(LinkStatus::InvalidCharactersInHostname);
            }
            _ => {
                error!(error = ?self, url, "unhandled hickory_resolver::proto::ProtoErrorKind variant");
            }
        }
    }
}

impl ExtractStatus for hyper::Error {
    fn extract_status(&self, chooser: &mut StatusChooser, _url: &str) {
        if self.is_incomplete_message() {
            chooser.push(LinkStatus::ServerDisconnected);
        }
        if self.is_parse() {
            chooser.push(LinkStatus::BadHttp);
        }
    }
}

impl ExtractStatus for reqwest::Error {
    fn extract_status(&self, chooser: &mut StatusChooser, _url: &str) {
        if self.is_timeout() {
            chooser.push(LinkStatus::Timeout);
        }
    }
}

impl ExtractStatus for rustls::Error {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        use rustls::Error::*;
        match self {
            InvalidCertificate(certificate_error) => {
                certificate_error.extract_status(chooser, url);
            }
            AlertReceived(alert_description) => {
                alert_description.extract_status(chooser, url);
            }
            Other(other_error) => {
                other_error.extract_status(chooser, url);
            }
            _ => {
                chooser.push(LinkStatus::SslError);
                error!(error = ?self, url, "unhandled rustls::Error variant");
            }
        }
    }
}

impl ExtractStatus for rustls::OtherError {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        self.0.as_ref().extract_status(chooser, url);
    }
}

impl ExtractStatus for rustls::AlertDescription {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        use rustls::AlertDescription::*;
        match self {
            HandshakeFailure => {
                chooser.push(LinkStatus::SslHandshakeFailure);
            }
            UnrecognisedName => {
                chooser.push(LinkStatus::SslError);
            }
            InternalError => {
                chooser.push(LinkStatus::SslError);
            }
            _ => {
                chooser.push(LinkStatus::SslError);
                error!(error = ?self, url, "unhandled rustls::AlertDescription variant");
            }
        }
    }
}

impl ExtractStatus for rustls::CertificateError {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        use rustls::CertificateError::*;
        match self {
            Expired | ExpiredContext { .. } => {
                chooser.push(LinkStatus::SslCertificateHasExpired);
            }
            UnknownIssuer => {
                chooser.push(LinkStatus::SslCertificateIncompleteChain);
            }
            NotValidForName | NotValidForNameContext { .. } => {
                chooser.push(LinkStatus::SslCertificateHostnameMismatch);
            }
            Other(other_error) => {
                other_error.extract_status(chooser, url);
            }
            _ => {
                chooser.push(LinkStatus::SslError);
                error!(error = ?self, url, "unhandled rustls::CertificateError variant");
            }
        }
    }
}

impl ExtractStatus for std::io::Error {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        use std::io::ErrorKind::*;
        match self.kind() {
            HostUnreachable => {
                chooser.push(LinkStatus::HostUnreachable);
            }
            ConnectionRefused => {
                chooser.push(LinkStatus::ConnectionRefused);
            }
            UnexpectedEof => {
                chooser.push(LinkStatus::ConnectionResetByPeer);
            }
            ConnectionReset => {
                chooser.push(LinkStatus::ConnectionResetByPeer);
            }
            _ => {}
        }
        if let Some(inner) = self.get_ref() {
            inner.extract_status(chooser, url);
        }
    }
}

impl ExtractStatus for webpki::Error {
    fn extract_status(&self, chooser: &mut StatusChooser, url: &str) {
        use webpki::Error::*;
        match self {
            CaUsedAsEndEntity => {
                chooser.push(LinkStatus::SslCertificateSelfSigned);
            }
            _ => {
                chooser.push(LinkStatus::SslError);
                error!(error = ?self, url, "unhandled webpki::Error variant");
            }
        }
    }
}

pub fn extract_status(error: &(dyn std::error::Error + 'static), url: &str) -> LinkStatus {
    let mut chooser = StatusChooser::new();

    error.extract_status(&mut chooser, url);

    let status = chooser.get();

    if status == LinkStatus::UnknownError {
        error!(?error, url, "unhandled error type");
    }

    status
}
