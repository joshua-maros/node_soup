use super::{TypedBlob, Blob, TypedBlobView};
use crate::engine::BlobLayout;

impl TypedBlob {
    pub fn view(&self) -> TypedBlobView {
        unsafe { TypedBlobView::new(&self.layout, &self.blob.bytes, &self.blob.dynamic_components) }
    }
}

impl<'a> TypedBlobView<'a> {
    pub fn assert_valid(&self) {
        debug_assert_eq!(self.frozen_size() as usize, self.bytes.len());
    }

    pub fn to_owned(&self) -> TypedBlob {
        TypedBlob {
            blob: Blob {
                bytes: self.bytes.into(),
                dynamic_components: self.dynamic_components.into(),
            },
            layout: self.layout.clone(),
        }
    }

    pub fn layout(&self) -> &'a BlobLayout {
        self.layout
    }

    /// Returns the number of elements in the topmost collection this view
    /// covers. If you want the total size of the structure, use frozen_size
    /// instead.
    pub fn len(&self) -> Option<u32> {
        self.layout.len()
    }

    /// How many bytes this blob contains. Dynamic data is stored as pointers to
    /// the start of the data, so they only count for 4/8 bytes each.
    pub fn frozen_size(&self) -> u32 {
        self.layout.size()
    }
}
