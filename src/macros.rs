#[macro_export]
#[doc(hidden)]
macro_rules! map_native {
    (
        $(#[$struct_doc:meta])*
        $native:ident => $rusty:ident {
            $(
                $(#[$field_doc:meta])*
                $native_name:ident => $rusty_name:ident: $field_type:ty
            ),+ $(,)*
        }
    ) => {

        $(#[$struct_doc])*
        #[derive(Debug, Clone, Default)]
        pub struct $rusty {
            $(
                $(#[$field_doc])*
                pub $rusty_name: $field_type
            ),+
        }

        impl From<&$native> for $rusty {
            fn from(native: &$native) -> Self {
                $rusty {
                    $( $rusty_name : native.$native_name.into()),+
                }
            }
        }

        impl From<$rusty> for $native {
            fn from(rusty: $rusty) -> Self {
                $native {
                    $( $native_name : rusty.$rusty_name.into()),+
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! constant_to_enum {
    (
        $(#[$struct_doc:meta])*
        ($const_type:ident => $required_const_type:ident) => $enum_name:ident {
            $(
                $(#[$field_doc:meta])*
                $native_name:ident => $rusty_name:ident
            ),+ $(,)*
        }
    ) => {

        $(#[$struct_doc])*
        #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, strum::EnumString, strum::EnumIter, strum::Display)]
        pub enum $enum_name {
            $(
                $(#[$field_doc])*
                $rusty_name,
            )*
            Unknown($const_type)
        }

        impl Default for $enum_name {
            fn default() -> Self {
                $enum_name::Unknown(0)
            }
        }

        impl From<$required_const_type> for $enum_name {
            fn from(native: $required_const_type) -> Self {
                match native as $const_type {
                    $( $native_name => $enum_name::$rusty_name, )*
                    unknown => $enum_name::Unknown(unknown)
                }
            }
        }

        impl From<$enum_name> for $required_const_type {
            fn from(rusty: $enum_name) -> Self {
                match rusty {
                    $( $enum_name::$rusty_name => $native_name as $required_const_type, )*
                    $enum_name::Unknown(val) => val as $required_const_type
                }
            }
        }
    }
}
