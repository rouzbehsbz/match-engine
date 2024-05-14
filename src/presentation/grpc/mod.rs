use tonic::{Response, Status};

pub mod server;

pub type GrpcResult<T> = Result<Response<T>, Status>;
