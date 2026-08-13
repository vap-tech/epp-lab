use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::{
    RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};

pub fn load_acceptor(
    cert_path: &Path,
    key_path: &Path,
    client_ca_path: &Path,
) -> Result<TlsAcceptor> {
    let certificates = load_certificates(cert_path)?;
    let private_key = load_private_key(key_path)?;
    let client_ca = load_certificates(client_ca_path)?;
    let mut roots = RootCertStore::empty();
    for certificate in client_ca {
        roots
            .add(certificate)
            .context("failed to add client CA certificate")?;
    }

    let verifier = tokio_rustls::rustls::server::WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .context("failed to build client certificate verifier")?;
    let config = ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(certificates, private_key)
        .context("failed to build TLS server configuration")?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}

fn load_certificates(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open certificate file {}", path.display()))?;
    rustls_pemfile::certs(&mut BufReader::new(file))
        .collect::<io::Result<Vec<_>>>()
        .with_context(|| format!("failed to parse certificates from {}", path.display()))
}

fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open private key file {}", path.display()))?;
    rustls_pemfile::private_key(&mut BufReader::new(file))
        .context("failed to parse private key")?
        .with_context(|| format!("no private key found in {}", path.display()))
}
