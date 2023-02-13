use super::{BlobView, Blob};
use crate::engine::DataLayout;

impl<'a> BlobView<'a> {
    pub fn as_i32(&self) -> Result<i32, ()> {
        if let DataLayout::Integer = self.layout {
            debug_assert_eq!(self.bytes.len(), 4);
            Ok(i32::from_ne_bytes(self.bytes.try_into().unwrap()))
        } else {
            Err(())
        }
    }

    pub fn as_f32(&self) -> Result<f32, ()> {
        if let DataLayout::Float = self.layout {
            debug_assert_eq!(self.bytes.len(), 4);
            Ok(f32::from_ne_bytes(self.bytes.try_into().unwrap()))
        } else {
            Err(())
        }
    }

    pub fn as_string(&self) -> Result<&'a str, ()> {
        if &DataLayout::DynamicIndex(Box::new(DataLayout::Byte)) == self.layout {
            debug_assert_eq!(self.dynamic_components.len(), 0);
            std::str::from_utf8(self.bytes).map_err(|_| ())
        } else {
            Err(())
        }
    }
}

impl Blob {
    pub unsafe fn as_raw_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.object.bytes[..]
    }
}
