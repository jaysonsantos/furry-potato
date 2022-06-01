mod client_position;
mod transaction;

#[macro_export]
macro_rules! implement_storage {
    ($type_name:ident, $primary_key:expr, $partition_key:expr) => {
        use $crate::{FromStorage, ToFromStorage, ToStorage};
        impl ToStorage for $type_name {
            fn to_bytes(&self) -> Vec<u8> {
                serde_json::to_vec(&self).expect("failed to convert entity to bytes")
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
                $partition_key(self) as usize
            }

            fn primary_key(&self) -> String {
                $primary_key(self)
            }
        }
    };
}
