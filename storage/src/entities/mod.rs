mod client_position;
mod transaction;

#[macro_export]
macro_rules! implement_storage {
    ($type_name:ident, $primary_key:expr, $update_instance:expr) => {
        use $crate::{FromStorage, ToFromStorage, ToStorage};
        impl ToStorage for $type_name {
            fn to_bytes(&self) -> Vec<u8> {
                serde_json::to_vec(&self).expect("failed to convert entity to bytes")
            }

            fn get_updated(&self, new: &Self) -> Self {
                $update_instance(self, new)
            }
        }
        impl FromStorage for $type_name {
            fn from_bytes(input: &[u8]) -> ::std::result::Result<Self, $crate::errors::Data>
            where
                Self: Sized,
            {
                Ok(serde_json::from_slice(input)?)
            }
        }
        impl ToFromStorage for $type_name {
            fn partition(&self) -> usize {
                self.client as usize
            }

            fn primary_key(&self) -> String {
                $primary_key(self)
            }
        }
    };
}
