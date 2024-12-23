use std::future::Future;

use http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Response, StatusCode};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use reqwest::{Body, Client, Request as Reqwest, Url};
use std::pin::Pin;

pub struct ProxyService {}

impl hyper::service::Service<hyper::Request<Incoming>> for ProxyService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<Incoming>) -> Self::Future {
        Box::pin(async move {
            let response = proxy(req).await.unwrap();
            Ok(response)
        })
    }
}

async fn proxy(req: hyper::Request<Incoming>) -> Result<Response<Body>, hyper::Error> {
    // Extract the target URL from the request path
    let target_url = match req.uri().path().strip_prefix('/') {
        Some(url) => format!("{}?{}", url, req.uri().query().unwrap_or("")),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid URL path".into())
                .unwrap())
        }
    };

    // Try to parse the target URL as a URI
    let target_uri = match target_url.parse::<Url>() {
        Ok(uri) => uri,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid URL format".into())
                .unwrap())
        }
    };

    println!("Forwarding {} request to: {}", req.method(), target_uri);
    // Create a new request to forward
    let mut forward_req = Reqwest::new(req.method().clone(), target_uri);

    // Forward the headers from the original request
    for (key, value) in req.headers() {
        forward_req.headers_mut().insert(key, value.clone());
    }

    let whole_body = req.collect().await?.to_bytes();

    *forward_req.body_mut() = Some(whole_body.into());

    // Send the request to the target server
    let res = Client::new().execute(forward_req).await;

    // Handle the response from the target
    match res {
        Ok(mut response) => {
            let headers = response.headers_mut();
            headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
            Ok(response.into())
        }
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to fetch the target URL".into())
            .unwrap()),
    }
}
