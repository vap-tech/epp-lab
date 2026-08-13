use std::io::Cursor;

use chrono::TimeZone;
use sha2::{Digest, Sha256};
use x509_parser::prelude::*;

#[derive(Debug)]
pub(crate) struct CertificateMetadata {
    pub fingerprint_sha256: String,
    pub subject: String,
    pub serial_number: String,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CertificateError {
    #[error("invalid PEM certificate: {0}")]
    Pem(String),
    #[error("invalid X.509 certificate: {0}")]
    X509(String),
    #[error("invalid certificate time: {0}")]
    Time(String),
}

pub(crate) fn parse_pem(pem: &str) -> Result<CertificateMetadata, CertificateError> {
    let mut reader = Cursor::new(pem.as_bytes());
    let der = rustls_pemfile::certs(&mut reader)
        .next()
        .ok_or_else(|| CertificateError::Pem("certificate not found".into()))
        .map_err(|e| CertificateError::Pem(e.to_string()))?
        .map_err(|e| CertificateError::Pem(e.to_string()))?;
    let (_, certificate) = X509Certificate::from_der(der.as_ref())
        .map_err(|e| CertificateError::X509(e.to_string()))?;
    let validity = certificate.validity();
    let not_before = chrono::Utc
        .timestamp_opt(validity.not_before.to_datetime().unix_timestamp(), 0)
        .single()
        .ok_or_else(|| CertificateError::Time("not_before is out of range".into()))?;
    let not_after = chrono::Utc
        .timestamp_opt(validity.not_after.to_datetime().unix_timestamp(), 0)
        .single()
        .ok_or_else(|| CertificateError::Time("not_after is out of range".into()))?;
    let fingerprint = hex::encode(Sha256::digest(der.as_ref()));
    Ok(CertificateMetadata {
        fingerprint_sha256: fingerprint,
        subject: certificate.subject().to_string(),
        serial_number: certificate.raw_serial_as_string(),
        not_before,
        not_after,
    })
}

#[cfg(test)]
mod tests {
    use super::{CertificateError, parse_pem};

    #[test]
    fn rejects_missing_certificate() {
        assert!(matches!(
            parse_pem("not a certificate"),
            Err(CertificateError::Pem(_))
        ));
    }
}
