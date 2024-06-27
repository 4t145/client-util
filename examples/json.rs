use client_util::prelude::*;

#[tokio::main]
async fn main() -> client_util::Result<()> {
    let mut client = client_util::client::hyper_tls_client();
    let request = http::Request::get("https://httpbin.org/json")
        .version(http::Version::HTTP_11)
        .empty()?;
    let (parts, response) = request
        .send(&mut client)
        .await?
        .json::<serde_json::Value>()
        .await?
        .into_parts();
    println!("{:?}", parts);
    println!("{:?}", response);

    let request = http::Request::post("https://httpbin.org/json")
        .json(&serde_json::json!({"key": "value"}))?;
    let response = request.send(&mut client).await?;
    
    println!("{:?}", response);
    Ok(())
}
