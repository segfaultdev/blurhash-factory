use std::env;

use url::Url;
use tonic::{Request, Response, Status, Code, transport::Server};
use image::{GenericImageView};
use reqwest;
use blurhash::encode;

use factory::{GenerateRequest, GenerateReply, generate_reply::Metadata};
use factory::factory_server::{Factory, FactoryServer};

mod factory {
    tonic::include_proto!("factory");
}

#[derive(Default)]
struct Service {
    max_size: i32,
    allowed_host: Vec<String>,
}

impl Service {
    fn new(max_size: i32, allowed_host: Vec<String>) -> Self {
        Self {
            max_size,
            allowed_host,
        }
    }

    async fn get_source(&self, source: &String) -> Result<(Vec<u8>, u64), String> {
        let request = reqwest::get(source).await;

        if let Err(error) = &request {
            return Err(format!("Error {} from source", error.status().unwrap().as_str()))
        }

        let response = request.unwrap();
        let size = response.content_length().unwrap();
        let bytes = response.bytes().await.unwrap();
        
        Ok((bytes.to_vec(), size))
    }
}


#[tonic::async_trait]
impl Factory for Service {
    async fn generate(&self, request: Request<GenerateRequest>) -> Result<Response<GenerateReply>, Status> {
        let url = Url::parse(request.into_inner().source.as_str());

        if url.is_err() {
            return Err(Status::new(Code::InvalidArgument, "Invalid source"));
        }

        let parsed = url.unwrap();

        if !parsed.has_host() || !self.allowed_host.contains(&parsed.host().unwrap().to_string()) {
            return Err(Status::new(Code::PermissionDenied, "Source host is not allowed"))
        }

        match self.get_source(&(&parsed.into_string()).clone()).await {
            Ok((buffer, size)) => {
                if size > self.max_size as u64 {
                    return Err(Status::new(Code::Aborted, "Source size is too large"));
                }

                let image = image::load_from_memory(&buffer).expect("Cannot open image");
                let (width, height ) = image.dimensions();
                let hash = encode(4, 3, width, height, &image.to_rgba8().to_vec());

                let reply = GenerateReply {
                    hash,
                    metadata: Some(
                        Metadata {
                            width: width as i32,
                            height: height as i32,
                        },
                    )
                };

                Ok(Response::new(reply))
            }
            Err(msg)=> Err(Status::new(Code::InvalidArgument, msg))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();
    let max = env::var("MAX_IMAGE_SIZE").map_or(1048576, |m| m.parse().expect("Invalid MAX_IMAGE_SIZE"));
    let allowed = env::var("ALLOWED_HOSTS").map_or(
        vec![], |host| host.split(',').map(|s| String::from(s)).collect()
    );
    let service = Service::new(max, allowed);

    println!("FactoryServer listening on {}", &addr);

    Server::builder()
        .add_service(FactoryServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
