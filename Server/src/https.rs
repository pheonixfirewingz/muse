use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rustls::{ServerConfig, Certificate, PrivateKey, server::NoClientAuth};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio_rustls::TlsAcceptor;
use anyhow::{Result, Context};
use std::env;
use std::sync::Arc;

pub struct HttpsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub port: u16,
}

impl Default for HttpsConfig {
    fn default() -> Self {
        Self {
            cert_path: env::var("HTTPS_CERT_PATH").unwrap_or_else(|_| "certs/cert.pem".to_string()),
            key_path: env::var("HTTPS_KEY_PATH").unwrap_or_else(|_| "certs/key.pem".to_string()),
            port: env::var("HTTPS_PORT")
                .unwrap_or_else(|_| "8443".to_string())
                .parse()
                .unwrap_or(8443),
        }
    }
}

pub fn load_rustls_config(config: &HttpsConfig) -> Result<ServerConfig> {
    let cert_file = &mut BufReader::new(
        File::open(&config.cert_path)
            .with_context(|| format!("Failed to open certificate file: {}", config.cert_path))?
    );
    let key_file = &mut BufReader::new(
        File::open(&config.key_path)
            .with_context(|| format!("Failed to open private key file: {}", config.key_path))?
    );

    let cert_chain = certs(cert_file)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Certificate)
        .collect();

    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        // Try RSA private keys if PKCS8 keys are not found
        let key_file = &mut BufReader::new(
            File::open(&config.key_path)
                .with_context(|| format!("Failed to open private key file: {}", config.key_path))?
        );
        keys = rsa_private_keys(key_file)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(PrivateKey)
            .collect();
    }

    if keys.is_empty() {
        anyhow::bail!("No private keys found in {}", config.key_path);
    }

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .with_context(|| "Failed to create TLS configuration")?;

    Ok(config)
}

pub fn create_tls_acceptor(config: &HttpsConfig) -> Result<TlsAcceptor> {
    let rustls_config = load_rustls_config(config)?;
    Ok(TlsAcceptor::from(Arc::new(rustls_config)))
}

pub fn certs_exist(config: &HttpsConfig) -> bool {
    Path::new(&config.cert_path).exists() && Path::new(&config.key_path).exists()
} 