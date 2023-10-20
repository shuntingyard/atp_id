use async_trait::async_trait;

#[derive(Default)]
pub struct BskyClient(reqwest::Client);

#[async_trait]
impl atrium_api::xrpc::HttpClient for BskyClient {
    async fn send(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
        let res = self.0.execute(req.try_into()?).await?;
        let mut builder = http::Response::builder().status(res.status());
        for (k, v) in res.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(res.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

impl atrium_api::xrpc::XrpcClient for BskyClient {
    fn host(&self) -> &str {
        "https://bsky.social"
    }

    fn auth(&self, _reauthenticate: bool) -> Option<&str> {
        None
    }
}

atrium_api::impl_traits!(BskyClient);
