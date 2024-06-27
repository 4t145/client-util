use client_util::prelude::*;

#[tokio::main]
async fn main() -> client_util::Result<()> {
    let form = Form::new()
        .text("key", "value")
        .part("file", Part::bytes(b"hello, world!"));
    let mut client = client_util::client::hyper_tls_client();
    let request = http::Request::post("https://httpbin.org/anything")
        .version(http::Version::HTTP_11)
        .multipart(form)?;
    let (parts, response) = request
        .send(&mut client)
        .await?
        .json::<serde_json::Value>()
        .await?
        .into_parts();
    println!("{:?}", parts);
    println!("{}", serde_json::to_string_pretty(&response).unwrap());

    Ok(())
}
