use std::future::Future;

use http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue, Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper::{body::Incoming, service::Service};
use reqwest::{Body, Client, Request as Reqwest, Url};
use std::pin::Pin;

pub struct ProxyService;

impl Service<Request<Incoming>> for ProxyService {
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<Incoming>) -> Self::Future {
        Box::pin(async move { proxy(req).await })
    }
}

async fn proxy(req: Request<Incoming>) -> Result<Response<Body>, http::Error> {
    if req.uri().path() == "/favicon.ico" {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("".into());
    }
    // Extract the target URL from the request path
    let target_url = match req.uri().path().strip_prefix('/') {
        Some(url) => match req.uri().query() {
            Some(query) => format!("{}?{}", url, query),
            None => url.to_string(),
        },
        None => {
            println!("Invalid URL path {:?}", req.uri().path());
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid URL path".into());
        }
    };

    // Try to parse the target URL as a URI
    let target_uri = match target_url.parse::<Url>() {
        Ok(uri) => uri,
        Err(_) => {
            println!("Invalid URL format {:?}", target_url);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid URL format".into());
        }
    };

    let text = format!("Forwarding {} request to: {}", req.method(), target_uri);

    let mut forward_req = Reqwest::new(req.method().clone(), target_uri.clone());

    *forward_req.headers_mut() = req.headers().clone();
    forward_req.headers_mut().append(
        "Host",
        target_uri
            .host_str()
            .unwrap_or("localhost")
            .parse()
            .unwrap(),
    );
    *forward_req.body_mut() = Some(Body::wrap(req.into_body().boxed()));

    // Handle the response from the target
    match Client::new().execute(forward_req).await {
        Ok(mut response) => {
            response
                .headers_mut()
                .insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
            println!("{} - {}", text, response.status());
            Ok(response.into())
        }
        Err(err) => {
            println!("{} - {}", text, err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to fetch the target URL".into())
        }
    }
}
