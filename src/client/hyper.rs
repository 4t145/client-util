pub use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::{connect::Connect, Client};

pub type HyperClient = Client<HttpConnector, crate::DynBody>;

pub fn build_hyper_client_with_connector<C>(connector: C) -> Client<C, crate::DynBody>
where
    C: Connect + Clone,
{
    use hyper_util::client::legacy::Builder;
    use hyper_util::rt::TokioExecutor;
    let builder = Builder::new(TokioExecutor::new());
    builder.build::<_, crate::DynBody>(connector)
}

pub fn build_hyper_client() -> HyperClient {
    build_hyper_client_with_connector(HttpConnector::new())
}

crate::shared_client!(pub hyper_client: build_hyper_client -> HyperClient);

/// TLS support
#[cfg(feature = "client-hyper-rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "client-hyper-rustls")))]
mod tls {
    use hyper_rustls::HttpsConnector;
    use hyper_rustls::HttpsConnectorBuilder;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::client::legacy::Client;
    use rustls::client::danger::ServerCertVerifier;
    use rustls::SignatureScheme;

    #[derive(Debug)]
    pub(crate) struct NoServerCertVerifier;

    impl ServerCertVerifier for NoServerCertVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            vec![
                SignatureScheme::RSA_PKCS1_SHA1,
                SignatureScheme::ECDSA_SHA1_Legacy,
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::ECDSA_NISTP256_SHA256,
                SignatureScheme::RSA_PKCS1_SHA384,
                SignatureScheme::ECDSA_NISTP384_SHA384,
                SignatureScheme::RSA_PKCS1_SHA512,
                SignatureScheme::ECDSA_NISTP521_SHA512,
                SignatureScheme::RSA_PSS_SHA256,
                SignatureScheme::RSA_PSS_SHA384,
                SignatureScheme::RSA_PSS_SHA512,
                SignatureScheme::ED25519,
                SignatureScheme::ED448,
            ]
        }
    }
    use super::build_hyper_client_with_connector;

    pub type HyperTlsClient = Client<HttpsConnector<HttpConnector>, crate::DynBody>;

    pub fn build_connector_with_tls_config(
        tls_config: rustls::ClientConfig,
    ) -> HttpsConnector<HttpConnector> {
        HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build()
    }

    pub fn rustls_default_config() -> rustls::ClientConfig {
        let store = rustls::RootCertStore::empty();
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(store)
            .with_no_client_auth();
        let mut dangerous_config = rustls::ClientConfig::dangerous(&mut config);
        dangerous_config.set_certificate_verifier(std::sync::Arc::new(NoServerCertVerifier));
        config
    }

    pub fn build_tls_hyper_client() -> HyperTlsClient {
        let connector = build_connector_with_tls_config(rustls_default_config());
        build_hyper_client_with_connector(connector)
    }

    crate::shared_client!(pub hyper_tls_client: build_tls_hyper_client -> HyperTlsClient);
}

#[cfg(feature = "client-hyper-rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "client-hyper-rustls")))]
pub use tls::*;
