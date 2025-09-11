use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};

/// TLS support
#[cfg(feature = "client-hyper-rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "client-hyper-rustls")))]
mod tls {
    use hyper_util::{
        client::legacy::{connect::HttpConnector, Client as HyperClient},
        rt::TokioExecutor,
    };
    use rustls::ClientConfig;

    pub type HyperHttpsClient<B> = HyperClient<HttpsConnector<HttpConnector>, B>;

    use hyper_rustls::{ConfigBuilderExt, HttpsConnector};
    pub fn build_https_client<B>() -> std::io::Result<HyperHttpsClient<B>>
    where
        B: http_body::Body + Send,
        B::Data: Send,
    {
        let client = HyperClient::builder(TokioExecutor::default()).build(
            HttpsConnector::<HttpConnector>::builder()
                .with_tls_config(
                    ClientConfig::builder()
                        .with_native_roots()?
                        .with_no_client_auth(),
                )
                .https_or_http()
                .enable_all_versions()
                .build(),
        );
        Ok(client)
    }
}

#[cfg(feature = "client-hyper-rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "client-hyper-rustls")))]
pub use tls::*;

type HyperHttpClient<B> = HyperClient<HttpConnector, B>;

pub fn build_http_client<B>() -> HyperHttpClient<B>
where
    B: http_body::Body + Send,
    B::Data: Send,
{
    HyperClient::builder(TokioExecutor::default()).build(HttpConnector::new())
}
