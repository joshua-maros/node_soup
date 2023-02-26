use super::TypedBlob;
use crate::engine::BlobLayout;

impl TypedBlob {
    pub fn layout(&self) -> &BlobLayout {
        self.view().layout()
    }
}
