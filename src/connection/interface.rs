use std::{collections::BTreeMap, fmt::Display};

use thiserror::Error;

type InterfaceList = BTreeMap<u8, BTreeMap<u8, State>>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid input, was not either RELEASED or RESERVED_BY_YOU or RESERVED_BY_OTHER but {:?}", .0)]
    InvalidInput(String),
}

#[derive(Default, Debug)]
pub struct Interfaces {
    pub modules: InterfaceList,
}

#[derive(Debug, Clone)]
pub struct State {
    pub lock: Lock,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lock {
    Released,
    ReservedByYou,
    ReservedByOther,
}

impl Lock {
    const fn str(self) -> &'static str {
        match self {
            Lock::Released => "Released",
            Lock::ReservedByYou => "ReleasedByYou",
            Lock::ReservedByOther => "ReleasedByOther",
        }
    }
}

impl Display for Lock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl From<Lock> for &'static str {
    fn from(lock: Lock) -> Self {
        lock.str()
    }
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
