use super::TypedBlob;

#[derive(Clone, Debug, PartialEq)]
pub enum BlobLayout {
    Float,
    Integer,
    Byte,
    FixedIndex(u32, Box<BlobLayout>),
    DynamicIndex(Box<BlobLayout>),
    FixedHeterogeneousMap(Box<TypedBlob>, Vec<BlobLayout>),
    FixedHomogeneousMap(Box<TypedBlob>, u32, Box<BlobLayout>),
    DynamicMap(Box<BlobLayout>),
}

impl BlobLayout {
    /// Returns if a value matching this layout can be resized. Does not tell
    /// you anything about whether or not components of this value can be
    /// resized.
    pub fn is_dynamic(&self) -> bool {
        match self {
            BlobLayout::Float | BlobLayout::Integer | BlobLayout::Byte => false,
            BlobLayout::FixedIndex(_, base) | BlobLayout::FixedHomogeneousMap(_, _, base) => {
                false
            }
            BlobLayout::FixedHeterogeneousMap(_, base) => false,
            BlobLayout::DynamicIndex(_) | BlobLayout::DynamicMap(_) => true,
        }
    }

    /// Equivalent to `!self.is_dynamic()`
    pub fn is_fixed(&self) -> bool {
        !self.is_dynamic()
    }

    /// How many components with dynamic (adjustable) size an object of this
    /// layout would have.
    pub fn num_dynamic_components(&self, dynamic_size: Option<u32>) -> usize {
        match self {
            BlobLayout::Float
            | BlobLayout::Integer
            | BlobLayout::Byte
            | BlobLayout::FixedIndex(..)
            | BlobLayout::FixedHomogeneousMap(_, _, _)
            | BlobLayout::FixedHeterogeneousMap(_, _) => {
                self.num_dynamic_components_when_component()
            }
            BlobLayout::DynamicIndex(eltype) | BlobLayout::DynamicMap(eltype) => {
                eltype.num_dynamic_components_when_component() * dynamic_size.unwrap() as usize
            }
        }
    }

    /// How many dynamic components are required to store an object of this
    /// layout, assuming it's just one component of a larger object with
    /// multiple components. (If it has a blob all to itself, use
    /// num_dynamic_components instead.)
    fn num_dynamic_components_when_component(&self) -> usize {
        match self {
            BlobLayout::Float | BlobLayout::Integer | BlobLayout::Byte => 0,
            BlobLayout::FixedIndex(len, eltype)
            | BlobLayout::FixedHomogeneousMap(_, len, eltype) => {
                *len as usize * eltype.num_dynamic_components_when_component()
            }
            BlobLayout::DynamicIndex(_) => 1,
            BlobLayout::FixedHeterogeneousMap(_, eltypes) => eltypes
                .iter()
                .map(|eltype| eltype.num_dynamic_components_when_component())
                .sum(),
            BlobLayout::DynamicMap(_) => 1,
        }
    }

    /// Returns the number of elements in the topmost collection this layout
    /// describes. If you want the total size of the structure, use frozen_size
    /// instead.
    pub fn len(&self) -> Option<u32> {
        match self {
            BlobLayout::Float | BlobLayout::Integer | BlobLayout::Byte => None,
            BlobLayout::FixedIndex(len, _) => Some(*len),
            BlobLayout::DynamicIndex(_) => todo!(),
            BlobLayout::FixedHeterogeneousMap(keys, _) => Some(keys.layout().len().unwrap()),
            BlobLayout::FixedHomogeneousMap(_, num_keys, _) => Some(*num_keys),
            BlobLayout::DynamicMap(_) => todo!(),
        }
    }

    pub fn string_keys(&self) -> Option<Vec<&str>> {
        match self {
            BlobLayout::Float
            | BlobLayout::Integer
            | BlobLayout::Byte
            | BlobLayout::FixedIndex(_, _)
            | BlobLayout::DynamicIndex(_)
            | BlobLayout::DynamicMap(_) => None,
            BlobLayout::FixedHeterogeneousMap(keys, _)
            | BlobLayout::FixedHomogeneousMap(keys, _, _) => {
                let mut string_keys = Vec::new();
                let keys = keys.view();
                for index in 0..keys.layout().len().unwrap() {
                    string_keys.push(keys.index(&(index as i32).into()).as_string().ok()?)
                }
                Some(string_keys)
            }
        }
    }

    /// How many bytes are needed to store a piece of data in this layout, where
    /// dynamic values are stored as native-width pointers.
    pub fn size(&self) -> u32 {
        match self {
            BlobLayout::Byte => 1,
            BlobLayout::Float | BlobLayout::Integer => 4,
            BlobLayout::FixedIndex(size, eltype)
            | BlobLayout::FixedHomogeneousMap(_, size, eltype) => *size * eltype.size(),
            BlobLayout::FixedHeterogeneousMap(_, eltypes) => {
                eltypes.iter().map(|eltype| eltype.size()).sum()
            }
            BlobLayout::DynamicMap(eltype) | BlobLayout::DynamicIndex(eltype) => {
                std::mem::size_of::<usize>() as u32
            }
        }
    }

    pub fn layout_after_index(&self, fixed_index: Option<&TypedBlob>) -> &BlobLayout {
        match self {
            BlobLayout::Float | BlobLayout::Integer | BlobLayout::Byte => {
                panic!("Cannot index value of scalar type {:#?}", self)
            }
            BlobLayout::FixedIndex(_, eltype)
            | BlobLayout::DynamicIndex(eltype)
            | BlobLayout::FixedHomogeneousMap(_, _, eltype)
            | BlobLayout::DynamicMap(eltype) => &*eltype,
            BlobLayout::FixedHeterogeneousMap(keys, eltypes) => {
                let keys = keys.view();
                let fixed_index = fixed_index.unwrap();
                let fixed_index = fixed_index.view();
                let mut options = Vec::new();
                for key_index in 0..keys.len().unwrap() {
                    let key = keys.index(&TypedBlob::from(key_index as i32));
                    if fixed_index == key {
                        return &eltypes[key_index as usize];
                    } else {
                        options.push(key);
                    }
                }
                panic!(
                    "Invalid index {:#?}, options are {:#?}",
                    fixed_index, options
                );
            }
        }
    }

    pub fn default_blob(&self) -> TypedBlob {
        match self {
            BlobLayout::Float => 0.0.into(),
            BlobLayout::Integer => 0.into(),
            BlobLayout::Byte => 0u8.into(),
            BlobLayout::FixedIndex(_, _) => todo!(),
            BlobLayout::DynamicIndex(_) => todo!(),
            BlobLayout::FixedHeterogeneousMap(keys_blob, eltypes) => {
                let mut components = Vec::new();
                for index in 0..keys_blob.view().len().unwrap() {
                    let name = keys_blob.view().index(&(index as i32).into()).to_owned();
                    components.push((name, eltypes[index as usize].default_blob()));
                }
                TypedBlob::fixed_heterogeneous_map(components)
            }
            BlobLayout::FixedHomogeneousMap(_, _, _) => todo!(),
            BlobLayout::DynamicMap(_) => todo!(),
        }
    }
}
