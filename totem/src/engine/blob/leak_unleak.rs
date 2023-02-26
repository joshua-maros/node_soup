use super::{TypedBlob, TypedBlobView, Blob};
use crate::engine::BlobLayout;

impl TypedBlob {
    pub fn leak(self) -> (BlobLayout, Box<[u8]>) {
        let bytes = if self.blob.dynamic_components.len() == 0 {
            self.blob.bytes.into_boxed_slice()
        } else {
            todo!()
        };
        (self.layout, bytes)
    }

    pub unsafe fn unleak(layout: BlobLayout, bytes: Box<[u8]>) -> Self {
        let dynamic_size = if layout.is_dynamic() {
            let eltype = layout.layout_after_index(None);
            Some(layout.size() / eltype.size())
        } else {
            None
        };
        if layout.num_dynamic_components(dynamic_size) > 0 {
            todo!()
        } else {
            let object = Blob {
                bytes: Vec::from(bytes),
                dynamic_components: vec![],
            };
            Self { blob: object, layout }
        }
    }
}
