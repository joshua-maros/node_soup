use super::{TypedBlobView, TypedBlob};
use crate::engine::BlobLayout;

impl<'a> TypedBlobView<'a> {
    pub fn as_i32(&self) -> Result<i32, ()> {
        if let BlobLayout::Integer = self.layout {
            debug_assert_eq!(self.bytes.len(), 4);
            Ok(i32::from_ne_bytes(self.bytes.try_into().unwrap()))
        } else {
            Err(())
        }
    }

    pub fn as_f32(&self) -> Result<f32, ()> {
        if let BlobLayout::Float = self.layout {
            debug_assert_eq!(self.bytes.len(), 4);
            Ok(f32::from_ne_bytes(self.bytes.try_into().unwrap()))
        } else {
            Err(())
        }
    }

    pub fn as_string(&self) -> Result<&'a str, ()> {
        if &BlobLayout::DynamicIndex(Box::new(BlobLayout::Byte)) == self.layout {
            debug_assert_eq!(self.dynamic_components.len(), 0);
            std::str::from_utf8(self.bytes).map_err(|_| ())
        } else {
            Err(())
        }
    }
}

impl TypedBlob {
    pub unsafe fn as_raw_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.blob.bytes[..]
    }
}
