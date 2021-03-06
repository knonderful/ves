/// Creates a bit-based struct that has another (primitive) data type as the internal value.
///
/// # Usage notes
///
/// The macro does not verify things like overlapping bit masks or internal value bounds. The latter
/// will result in a compiler error, while the former will result in strange behavior at run-time
/// (e.g. a setter method overriding values of another field). It is highly recommended to add some
/// unit tests for the resulting struct.
///
/// # Example
///
/// ```rust
/// use ves_proto_common::bit_struct;
///
/// bit_struct!(
///     /// An entry in the object character table.
///     #[derive(Copy, Clone, Eq, PartialEq)]
///     pub struct ObjectCharacterTableIndex {
///         value: u8
///     }
///
///     impl {
///         #[bit_struct_field(shift = 0, mask = 0xF)]
///         /// The X-coordinate in the table.
///         pub fn x(&self) -> u8;
///
///         #[bit_struct_field(shift = 4, mask = 0xF)]
///         /// The Y-coordinate in the table.
///         pub fn y(&self) -> u8;
///     }
/// );
/// ```
#[macro_export]
macro_rules! bit_struct {
    (
        $(#[$struct_meta:meta])*
        $struct_vis:vis struct $struct_name:ident {
            value: $value_type:ty
        }

        impl {
            $(
                #[bit_struct_field(shift = $field_shift:expr, mask = $field_mask:expr)]
                $(#[$field_meta:meta])*
                $field_vis:vis fn $field_name:ident (&self) -> $field_type:ident;
            )*
        }

        $(
            padding {
                $(
                    #[bit_struct_field(shift = $padding_shift:expr, mask = $padding_mask:expr)]
                    $(#[$_padding_meta:meta])*
                    $_padding_vis:vis fn $padding_name:ident (&self) -> $padding_type:ident;
                )*
            }
        )?
    ) => {
        $(#[$struct_meta])*
        #[allow(dead_code)]
        $struct_vis struct $struct_name {
            value: $value_type,
        }

        #[allow(dead_code)]
        #[allow(clippy::unnecessary_cast)]
        impl $struct_name {
            /// Creates a new instance from the bit fields.
            pub fn new($($field_name: $field_type,)*) -> Self {
                let value = 0
                $(
                    | (($field_name & $field_mask) as $value_type) << $field_shift
                )* ;

                Self { value }
            }

            $(
                $(#[$field_meta])*
                #[inline(always)]
                $field_vis fn $field_name(&self) -> $field_type {
                    ((self.value >> $field_shift) & $field_mask) as $field_type
                }

                paste::paste! {
                    #[inline(always)]
                    $field_vis fn [<$field_name _mask>]() -> $value_type {
                        ($field_mask as $value_type) << $field_shift
                    }

                    $(#[$field_meta])*
                    #[inline(always)]
                    $field_vis fn [<set_ $field_name>](&mut self, val: $field_type) {
                        let masked_val = val & $field_mask;
                        // Make sure the provided value does not exceed the mask range.
                        assert_eq!(val, masked_val, "Provided value for {} should not exceed {}, but is {}.", stringify!([<set_ $field_name>]), $field_mask as $field_type, val);

                        // Clear the backing bits.
                        let cleared = self.value ^ (self.value & Self::[<$field_name _mask>]());
                        // Apply the provided value.
                        self.value = cleared | ((masked_val as $value_type) << $field_shift);
                    }
                }
            )*
        }

        impl From<$value_type> for $struct_name {
            fn from(value: $value_type) -> Self {
                Self {
                    value
                }
            }
        }

        impl From<$struct_name> for $value_type {
            fn from(obj: $struct_name) -> Self {
                obj.value
            }
        }

        impl From<&$struct_name> for $value_type {
            fn from(obj: &$struct_name) -> Self {
                obj.value
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                $(
                    .field(stringify!($field_name), &self.$field_name())
                )*
                    .finish()
            }
        }

        paste::paste! {
            #[allow(non_snake_case)]
            mod [<test_gen_ $struct_name>] {
                /// This is a sanity check for overlapping fields. It works by XORing all bitmasks (fields and padding)
                /// and then expects that the resulting value contains no binary zeros. Note that this is not a fail-proof
                /// test. For instance, if a bit overlaps twice (i.e. three fields use the same bit), the XORed will
                /// "cancel out" and this test will not fail. This is just intended to be a test for the most common types
                /// of declarative errors.
                #[test]
                #[allow(clippy::unnecessary_cast)]
                fn field_conflicts() {
                    let full_bitmask = 0
                    $(
                        ^ super::$struct_name::[<$field_name _mask>]()
                    )*
                    $(
                        $(
                            ^ (($padding_mask as $value_type) << $padding_shift)
                        )*
                    )?
                    ;

                    let zeros = full_bitmask.count_zeros();
                    assert_eq!(zeros, 0, "The bit mask contains {} zero(s) (might be the most-significant bits): {:b}", zeros, full_bitmask);
                }
            }
        }
    }
}
