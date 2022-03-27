pub mod polymerase;

pub mod proto {
    tonic::include_proto!("pcr");
}

pub mod common {
    tonic::include_proto!("common");
}