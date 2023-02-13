use super::{Blob, BlobView, RawObject};
use crate::engine::ObjectLayout;

impl Blob {
    pub fn leak(self) -> (ObjectLayout, Box<[u8]>) {
        let bytes = if self.object.dynamic_components.len() == 0 {
            self.object.bytes.into_boxed_slice()
        } else {
            todo!()
        };
        (self.layout, bytes)
    }

    pub unsafe fn unleak(layout: ObjectLayout, bytes: Box<[u8]>) -> Self {
        let dynamic_size = if layout.is_dynamic() {
            let eltype = layout.layout_after_index(None);
            Some(layout.size() / eltype.size())
        } else {
            None
        };
        if layout.num_dynamic_components(dynamic_size) > 0 {
            todo!()
        } else {
            let object = RawObject {
                bytes: Vec::from(bytes),
                dynamic_components: vec![],
            };
            Self { object, layout }
        }
    }
}
