mod constructors;
mod from;
mod view_wrappers;
mod view;
mod debug;
mod index;
mod extractors;
mod freeze;

use std::fmt::{self, Debug, Formatter, Display};

use self::constructors::SafetyLock;

use super::DataLayout;

#[derive(Clone, PartialEq)]
pub struct Blob {
    object: RawObject,
    layout: DataLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawObject {
    bytes: Vec<u8>,
    dynamic_components: Vec<RawObject>,
}

#[derive(Clone, PartialEq)]
pub struct BlobView<'a> {
    layout: &'a DataLayout,
    bytes: &'a [u8],
    dynamic_components: &'a [RawObject],
    safety_lock: SafetyLock,
}
