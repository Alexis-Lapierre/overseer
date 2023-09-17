use std::collections::BTreeMap;

use thiserror::Error;

type InterfaceList = BTreeMap<u8, BTreeMap<u8, State>>;

#[derive(Default, Debug)]
pub struct Interfaces {
    pub modules: InterfaceList,
}

#[derive(Debug, Clone)]
pub struct State {
    pub lock: Lock,
}

#[derive(Debug, Clone, Copy)]
pub enum Lock {
    Released,
    ReservedByYou,
    ReservedByOther,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid input, was not either RELEASED or RESERVED_BY_YOU or RESERVED_BY_OTHER but {:?}", .0)]
    InvalidInput(String),
}

impl TryFrom<&str> for Lock {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "RELEASED" => Ok(Self::Released),
            "RESERVED_BY_YOU" => Ok(Self::ReservedByYou),
            "RESERVED_BY_OTHER" => Ok(Self::ReservedByOther),
            other => Err(Error::InvalidInput(other.to_string())),
        }
    }
}
