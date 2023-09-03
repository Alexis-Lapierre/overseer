use std::collections::{BTreeMap, BTreeSet};

type InterfaceList = BTreeMap<u8, BTreeSet<u8>>;

#[derive(Default, Debug)]
pub struct Interfaces {
    pub modules: InterfaceList,
}
