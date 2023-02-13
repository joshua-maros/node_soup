use super::Blob;

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectLayout {
    Float,
    Integer,
    Byte,
    FixedIndex(u32, Box<ObjectLayout>),
    DynamicIndex(Box<ObjectLayout>),
    FixedHeterogeneousMap(Box<Blob>, Vec<ObjectLayout>),
    FixedHomogeneousMap(Box<Blob>, u32, Box<ObjectLayout>),
    DynamicMap(Box<ObjectLayout>),
}

impl ObjectLayout {
    /// Returns if a value matching this layout can be resized. Does not tell
    /// you anything about whether or not components of this value can be
    /// resized.
    pub fn is_dynamic(&self) -> bool {
        match self {
            ObjectLayout::Float | ObjectLayout::Integer | ObjectLayout::Byte => false,
            ObjectLayout::FixedIndex(_, base) | ObjectLayout::FixedHomogeneousMap(_, _, base) => {
                false
            }
            ObjectLayout::FixedHeterogeneousMap(_, base) => false,
            ObjectLayout::DynamicIndex(_) | ObjectLayout::DynamicMap(_) => true,
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
            ObjectLayout::Float
            | ObjectLayout::Integer
            | ObjectLayout::Byte
            | ObjectLayout::FixedIndex(..)
            | ObjectLayout::FixedHomogeneousMap(_, _, _)
            | ObjectLayout::FixedHeterogeneousMap(_, _) => {
                self.num_dynamic_components_when_component()
            }
            ObjectLayout::DynamicIndex(eltype) | ObjectLayout::DynamicMap(eltype) => {
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
            ObjectLayout::Float | ObjectLayout::Integer | ObjectLayout::Byte => 0,
            ObjectLayout::FixedIndex(len, eltype)
            | ObjectLayout::FixedHomogeneousMap(_, len, eltype) => {
                *len as usize * eltype.num_dynamic_components_when_component()
            }
            ObjectLayout::DynamicIndex(_) => 1,
            ObjectLayout::FixedHeterogeneousMap(_, eltypes) => eltypes
                .iter()
                .map(|eltype| eltype.num_dynamic_components_when_component())
                .sum(),
            ObjectLayout::DynamicMap(_) => 1,
        }
    }

    /// Returns the number of elements in the topmost collection this layout
    /// describes. If you want the total size of the structure, use frozen_size
    /// instead.
    pub fn len(&self) -> Option<u32> {
        match self {
            ObjectLayout::Float | ObjectLayout::Integer | ObjectLayout::Byte => None,
            ObjectLayout::FixedIndex(len, _) => Some(*len),
            ObjectLayout::DynamicIndex(_) => todo!(),
            ObjectLayout::FixedHeterogeneousMap(keys, _) => Some(keys.layout().len().unwrap()),
            ObjectLayout::FixedHomogeneousMap(_, num_keys, _) => Some(*num_keys),
            ObjectLayout::DynamicMap(_) => todo!(),
        }
    }

    pub fn string_keys(&self) -> Option<Vec<&str>> {
        match self {
            ObjectLayout::Float
            | ObjectLayout::Integer
            | ObjectLayout::Byte
            | ObjectLayout::FixedIndex(_, _)
            | ObjectLayout::DynamicIndex(_)
            | ObjectLayout::DynamicMap(_) => None,
            ObjectLayout::FixedHeterogeneousMap(keys, _)
            | ObjectLayout::FixedHomogeneousMap(keys, _, _) => {
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
            ObjectLayout::Byte => 1,
            ObjectLayout::Float | ObjectLayout::Integer => 4,
            ObjectLayout::FixedIndex(size, eltype)
            | ObjectLayout::FixedHomogeneousMap(_, size, eltype) => *size * eltype.size(),
            ObjectLayout::FixedHeterogeneousMap(_, eltypes) => {
                eltypes.iter().map(|eltype| eltype.size()).sum()
            }
            ObjectLayout::DynamicMap(eltype) | ObjectLayout::DynamicIndex(eltype) => {
                std::mem::size_of::<usize>() as u32
            }
        }
    }

    pub fn layout_after_index(&self, fixed_index: Option<&Blob>) -> &ObjectLayout {
        match self {
            ObjectLayout::Float | ObjectLayout::Integer | ObjectLayout::Byte => {
                panic!("Cannot index value of scalar type {:#?}", self)
            }
            ObjectLayout::FixedIndex(_, eltype)
            | ObjectLayout::DynamicIndex(eltype)
            | ObjectLayout::FixedHomogeneousMap(_, _, eltype)
            | ObjectLayout::DynamicMap(eltype) => &*eltype,
            ObjectLayout::FixedHeterogeneousMap(keys, eltypes) => {
                let keys = keys.view();
                let fixed_index = fixed_index.unwrap();
                let fixed_index = fixed_index.view();
                let mut options = Vec::new();
                for key_index in 0..keys.len().unwrap() {
                    let key = keys.index(&Blob::from(key_index as i32));
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

    pub fn default_blob(&self) -> Blob {
        match self {
            ObjectLayout::Float => 0.0.into(),
            ObjectLayout::Integer => 0.into(),
            ObjectLayout::Byte => 0u8.into(),
            ObjectLayout::FixedIndex(_, _) => todo!(),
            ObjectLayout::DynamicIndex(_) => todo!(),
            ObjectLayout::FixedHeterogeneousMap(keys_blob, eltypes) => {
                let mut components = Vec::new();
                for index in 0..keys_blob.view().len().unwrap() {
                    let name = keys_blob.view().index(&(index as i32).into()).to_owned();
                    components.push((name, eltypes[index as usize].default_blob()));
                }
                Blob::fixed_heterogeneous_map(components)
            }
            ObjectLayout::FixedHomogeneousMap(_, _, _) => todo!(),
            ObjectLayout::DynamicMap(_) => todo!(),
        }
    }
}
