pub mod dep;

pub mod proto {
    tonic::include_proto!("pos");
}

pub mod common {
    tonic::include_proto!("common");
}