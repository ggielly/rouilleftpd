// Gestion des connexions TLS pour rouilleftpd
use crate::core_tls::error::TlsError;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{rustls, TlsAcceptor};

pub struct TlsConnection {
    tls_acceptor: Option<TlsAcceptor>,
}

impl TlsConnection {
    pub fn new(cert_file: &str, key_file: &str) -> Result<Self, TlsError> {
        if !std::path::Path::new(cert_file).exists() || !std::path::Path::new(key_file).exists() {
            return Err(TlsError::TlsNotConfigured);
        }

        // Charger le certificat et la clé
        let certs = match std::fs::read(cert_file) {
            Ok(c) => c,
            Err(e) => return Err(TlsError::CertificateLoadError(e.to_string())),
        };

        let key = match std::fs::read(key_file) {
            Ok(k) => k,
            Err(e) => return Err(TlsError::PrivateKeyLoadError(e.to_string())),
        };

        // Créer la configuration TLS
        let cert_chain = match rustls_pemfile::certs(&mut &certs[..]) {
            Ok(c) => c,
            Err(e) => return Err(TlsError::CertificateLoadError(e.to_string())),
        };

        let mut keys = match rustls_pemfile::pkcs8_private_keys(&mut &key[..]) {
            Ok(k) => k,
            Err(e) => return Err(TlsError::PrivateKeyLoadError(e.to_string())),
        };

        let private_key = match keys.pop() {
            Some(k) => k,
            None => {
                return Err(TlsError::PrivateKeyLoadError(
                    "No private key found".to_string(),
                ))
            }
        };

        // Convertir les certificats et clés au format attendu par rustls
        let cert_chain: Vec<rustls::Certificate> =
            cert_chain.into_iter().map(rustls::Certificate).collect();

        let private_key = rustls::PrivateKey(private_key);

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| TlsError::TlsConfigError(e.to_string()))?;

        let acceptor = TlsAcceptor::from(Arc::new(config));

        Ok(Self {
            tls_acceptor: Some(acceptor),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.tls_acceptor.is_some()
    }

    pub async fn accept_tls(
        &self,
        stream: TcpStream,
    ) -> Result<tokio_rustls::server::TlsStream<TcpStream>, TlsError> {
        let acceptor = match &self.tls_acceptor {
            Some(a) => a,
            None => return Err(TlsError::TlsNotConfigured),
        };

        match acceptor.accept(stream).await {
            Ok(tls_stream) => Ok(tls_stream),
            Err(e) => Err(TlsError::TlsHandshakeError(e.to_string())),
        }
    }
}
