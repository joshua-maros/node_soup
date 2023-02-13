use std::fmt::{self, Debug, Display, Formatter};

use super::{Blob, BlobView, DataLayout};

impl<'a> BlobView<'a> {
    pub fn index(&self, index: &Blob) -> Self {
        match self.layout {
            DataLayout::Float | DataLayout::Integer | DataLayout::Byte => {
                panic!("Cannot index into scalar value of type {:#?}", self.layout)
            }
            DataLayout::FixedIndex(len, eltype) => {
                let stride = eltype.frozen_size();
                let index: u32 = index.view().as_i32().unwrap().try_into().unwrap();
                assert!(index < *len);
                if eltype.is_dynamic() {
                    let child = &self.dynamic_components[index as usize];
                    unsafe { Self::new(eltype, &child.bytes, &child.dynamic_components) }
                } else {
                    let start = index * stride;
                    let end = start + stride;
                    // TODO: Trim children.
                    let data = &self.bytes[start as usize..end as usize];
                    unsafe { Self::new(&*eltype, data, self.dynamic_components) }
                }
            }
            DataLayout::DynamicIndex(_) => todo!(),
            DataLayout::FixedHeterogeneousMap(keys, eltypes) => {
                let index = index.view();
                let keys = keys.view();
                let mut offset = 0;
                for key_index in 0..keys.len().unwrap() {
                    let eltype = &eltypes[key_index as usize];
                    let elsize = eltype.frozen_size();
                    if index == keys.index(&Blob::from(key_index as i32)) {
                        let data = &self.bytes[offset as usize..(offset + elsize) as usize];
                        // TODO: Trim children.
                        return unsafe { Self::new(eltype, data, self.dynamic_components) };
                    } else {
                        offset += elsize;
                    }
                }
                panic!("Invalid index");
            }
            DataLayout::FixedHomogeneousMap(keys, num_keys, eltype) => todo!(),
            DataLayout::DynamicMap(_) => todo!(),
        }
    }
}
