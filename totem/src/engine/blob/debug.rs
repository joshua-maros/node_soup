use std::fmt::{self, Debug, Formatter};

use super::{TypedBlob, TypedBlobView};
use crate::engine::BlobLayout;

impl Debug for TypedBlob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.view())
    }
}

impl<'a> Debug for TypedBlobView<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Ok(value) = self.as_i32() {
            write!(f, "{}", value)
        } else if let Ok(value) = self.as_f32() {
            write!(f, "{}", value)
        } else if let Ok(value) = self.as_string() {
            write!(f, "{}", value)
        } else if let BlobLayout::FixedIndex(len, _) = self.layout {
            write!(f, "[")?;
            for index in 0..*len {
                <Self as Debug>::fmt(&self.index(&(index as i32).into()), f)?;
                write!(f, ", ")?;
            }
            write!(f, "]")
        } else {
            f.debug_struct("BlobView")
                .field("layout", self.layout)
                .field("bytes", &self.bytes)
                .field("dynamic_components", &self.dynamic_components)
                .finish()
        }
    }
}
