use std::time::Duration;
use tower::{timeout::error::Elapsed, ServiceBuilder};
// use client_util::prelude::*;
use client_util::prelude::*;
#[tokio::main]
async fn main() -> client_util::Result<()> {
    let mut client = client_util::client::build_https_client().unwrap();
    let request = RequestBuilder::get("https://httpbin.org/json")?
        .version(http::Version::HTTP_11)
        .empty();
    let (parts, response) = request
        .send(&mut client)
        .await?
        .json::<serde_json::Value>()
        .await?
        .into_parts();
    println!("{parts:?}");
    println!("{response:?}");
    let request = RequestBuilder::post("https://httpbin.org/json")?
        .json(&serde_json::json!({"key": "value"}))?;
    let response = request
        .send(
            ServiceBuilder::new()
                .timeout(Duration::ZERO)
                .service(&mut client),
        )
        .await;
    let timeout_err = response.expect_err("should timeout");
    use std::error::Error;
    assert!(timeout_err
        .source()
        .expect("should have source")
        .is::<Elapsed>());
    Ok(())
}
