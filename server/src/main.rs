use sample_a::sample_api_a_server::{SampleApiA, SampleApiAServer};
use sample_a::{RequestA, ResponseA};
use sample_b::sample_api_b_server::{SampleApiB, SampleApiBServer};
use sample_b::{RequestB, ResponseB};
use tonic::{transport::Server, Request, Response, Status};
use tower::ServiceBuilder;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let sample_a = SampleApiAImpl::default();
    let sample_b = SampleApiBImpl::default();

    let svc_a = ServiceBuilder::new()
        .layer(tonic::service::interceptor(intercept1))
        .service(SampleApiAServer::new(sample_a));

    let svc_b = SampleApiBServer::new(sample_b);

    Server::builder()
        .layer(tonic::service::interceptor(intercept0))
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

// fn intercept2(req: Request<()>) -> Result<Request<()>, Status> {
//     println!("Intercepting 2");

//     Ok(req)
// }

// fn intercept3(req: Request<()>) -> Result<Request<()>, Status> {
//     println!("Intercepting 3");

//     Ok(req)
// }
