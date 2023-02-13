use crate::engine::DataLayout;

use super::{Blob, freeze::FrozenBlob};

impl Blob {
    pub fn layout(&self) -> &DataLayout {
        self.view().layout()
    }

    pub fn freeze(&self) -> FrozenBlob {
        self.view().freeze()
    }
}