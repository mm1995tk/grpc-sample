use sample_a::sample_api_a_server::{SampleApiA, SampleApiAServer};
use sample_a::{RequestA, ResponseA};
use sample_b::sample_api_b_server::{SampleApiB, SampleApiBServer};
use sample_b::{RequestB, ResponseB};
use tonic::{
    body::BoxBody,
    transport::{NamedService, Server},
    Request, Response, Status,
};

use anyhow;
use hyper::Body;
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceBuilder};

pub mod sample_a {
    tonic::include_proto!("sample_a");
}

pub mod sample_b {
    tonic::include_proto!("sample_b");
}

#[derive(Debug, Default)]
pub struct SampleApiAImpl {}
#[derive(Debug, Default)]
pub struct SampleApiBImpl {}

#[tonic::async_trait]
impl SampleApiA for SampleApiAImpl {
    async fn proc_a(&self, _request: Request<RequestA>) -> Result<Response<ResponseA>, Status> {
        let response = sample_a::ResponseA {
            message: format!("a!").into(),
        };

        Ok(Response::new(response))
    }
}

#[tonic::async_trait]
impl SampleApiB for SampleApiBImpl {
    async fn proc_b(&self, request: Request<RequestB>) -> Result<Response<ResponseB>, Status> {
        println!("Got a request: {:?}", request);

        let response = sample_b::ResponseB {
            message: format!("b!").into(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let addr = "[::1]:50051".parse()?;
    let sample_a = SampleApiAImpl::default();
    let sample_b = SampleApiBImpl::default();
    let default_layer = ServiceBuilder::new()
        .layer(MyMiddlewareLayer::default())
        .layer(tonic::service::interceptor(intercept0));

    let svc_a = ServiceBuilder::new()
        .layer(MyMiddlewareLayer::default())
        .layer(tonic::service::interceptor(intercept1))
        .service(SampleApiAServer::new(sample_a));

    let svc_b = SampleApiBServer::new(sample_b);

    Server::builder()
        .layer(default_layer)
        .add_service(svc_a)
        .add_service(svc_b)
        .serve(addr)
        .await?;

    Ok(())
}

fn intercept0(req: Request<()>) -> Result<Request<()>, Status> {
    println!("Intercepting 0");

    Ok(req)
}

fn intercept1(req: Request<()>) -> Result<Request<()>, Status> {
    println!("Intercepting 1");

    Ok(req)
}

#[derive(Debug, Clone, Default)]
struct MyMiddlewareLayer;

impl<S> Layer<S> for MyMiddlewareLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyMiddleware { inner: service }
    }
}

#[derive(Debug, Clone)]
struct MyMiddleware<S> {
    inner: S,
}

impl<S> Service<hyper::Request<Body>> for MyMiddleware<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            println!("start");
            let response = inner.call(req).await?;
            println!("return");
            Ok(response)
        })
    }
}

impl<S: NamedService> NamedService for MyMiddleware<S> {
    const NAME: &'static str = S::NAME;
}
