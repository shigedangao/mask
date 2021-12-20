use tonic::{
    Request,
    Response,
    Status
};
use care::care_status_server::CareStatus;
use care::{CareStatusPayload, CareStatusInput, CareStatusOutput};

pub mod care {
    tonic::include_proto!("hospital");
}

#[derive(Debug, Default)]
pub struct CareService {}

#[tonic::async_trait]
impl CareStatus for CareService {
    async fn get_status_by_region(
        &self,
        request: Request<CareStatusInput>
    ) -> Result<Response<CareStatusOutput>, Status> {
        let payload = CareStatusPayload::default();

        let reply = CareStatusOutput {
            cases: vec![payload]
        };

        Ok(Response::new(reply))
    }
}
