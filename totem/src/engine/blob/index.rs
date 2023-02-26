use std::fmt::{self, Debug, Display, Formatter};

use super::{TypedBlob, TypedBlobView, BlobLayout};

impl<'a> TypedBlobView<'a> {
    pub fn index(&self, index: &TypedBlob) -> Self {
        match self.layout {
            BlobLayout::Float | BlobLayout::Integer | BlobLayout::Byte => {
                panic!("Cannot index into scalar value of type {:#?}", self.layout)
            }
            BlobLayout::FixedIndex(len, eltype) => {
                let stride = eltype.size();
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
            BlobLayout::DynamicIndex(_) => todo!(),
            BlobLayout::FixedHeterogeneousMap(keys, eltypes) => {
                let index = index.view();
                let keys = keys.view();
                let mut offset = 0;
                for key_index in 0..keys.len().unwrap() {
                    let eltype = &eltypes[key_index as usize];
                    let elsize = eltype.size();
                    if index == keys.index(&TypedBlob::from(key_index as i32)) {
                        let data = &self.bytes[offset as usize..(offset + elsize) as usize];
                        // TODO: Trim children.
                        return unsafe { Self::new(eltype, data, self.dynamic_components) };
                    } else {
                        offset += elsize;
                    }
                }
                panic!("Invalid index");
            }
            BlobLayout::FixedHomogeneousMap(keys, num_keys, eltype) => todo!(),
            BlobLayout::DynamicMap(_) => todo!(),
        }
    }
}
