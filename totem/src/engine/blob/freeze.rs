use super::{BlobView, RawObject};

pub enum FrozenBlob<'a> {
    /// This is used when the data as it exists inide the blob can be used
    /// as-is.
    Existing(&'a [u8]),
    /// This is used when the data in the blob contains references to dynamic
    /// data that must be locked down prior to use.
    Generated(Box<[u8]>),
}

impl<'a> FrozenBlob<'a> {
    pub fn bytes(&self) -> &[u8] {
        match self {
            FrozenBlob::Existing(bytes) => *bytes,
            FrozenBlob::Generated(boxed_bytes) => &**boxed_bytes,
        }
    }

    pub fn into_owned_bytes(self) -> Box<[u8]> {
        match self {
            FrozenBlob::Existing(bytes) => Vec::from(bytes).into_boxed_slice(),
            FrozenBlob::Generated(boxed_bytes) => boxed_bytes,
        }
    }
}

// struct FreezeContext<'a> {
//     layout:
//     bytes: &'a [u8],
//     dynamic_components: &'a [RawObject],
//     component_pointer_locations: &'a mut Vec<usize>,
//     component_start_locations: &'a mut Vec<usize>,
// }

// fn freeze(ctx: &mut FreezeContext) {}

impl<'a> BlobView<'a> {
    pub fn freeze(&self) -> FrozenBlob<'a> {
        if self.dynamic_components.len() == 0 {
            FrozenBlob::Existing(self.bytes)
        } else {
            todo!();
            // let mut data = Vec::new();
            // let mut component_pointer_locations = Vec::new();
            // let mut component_start_locations = Vec::new();
            // let mut ctx = FreezeContext {
            //     object: &self.,
            //     component_pointer_locations: &mut
            // component_pointer_locations,
            //     component_start_locations: &mut component_start_locations,
            // };
            // data.extend_from_slice(&self.bytes);
            // let mut boxed = data.into_boxed_slice();
            // FrozenBlob::Generated(boxed)
        }
    }
}
