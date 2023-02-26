mod constructors;
mod from;
mod view_wrappers;
mod view;
mod debug;
mod index;
mod extractors;
mod leak_unleak;

use std::fmt::{self, Debug, Formatter, Display};

use self::constructors::SafetyLock;

use super::BlobLayout;

#[derive(Clone, PartialEq)]
pub struct TypedBlob {
    blob: Blob,
    layout: BlobLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Blob {
    bytes: Vec<u8>,
    dynamic_components: Vec<Blob>,
}

#[derive(Clone, PartialEq)]
pub struct TypedBlobView<'a> {
    layout: &'a BlobLayout,
    bytes: &'a [u8],
    dynamic_components: &'a [Blob],
    safety_lock: SafetyLock,
}
