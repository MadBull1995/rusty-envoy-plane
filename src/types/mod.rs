pub mod config;
pub mod extensions;

use prost::Message;

use crate::google::protobuf::Any;
pub trait ToProto {
    fn into_proto(&self) -> impl Message;
    fn type_url(&self) -> &str;
}