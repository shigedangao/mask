pub mod case;
pub mod status;
pub mod level;

// import generated struct by tonic
pub mod proto_hospital {
    tonic::include_proto!("hospital");
}

pub mod proto_newcase {
    tonic::include_proto!("newcase");
}
