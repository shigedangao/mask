pub mod case;
pub mod status;

// import generated struct by tonic
pub mod care {
    tonic::include_proto!("hospital");
}
