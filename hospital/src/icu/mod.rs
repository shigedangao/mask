pub mod level;

pub mod proto_icu {
    tonic::include_proto!("icu");
}

pub mod common {
    tonic::include_proto!("common");
}
