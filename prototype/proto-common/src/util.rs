/// Creates a bit-based struct that has another (primitive) data type as the internal value.
///
/// # Parameters
///
/// * Visibility of the struct.
/// * Name of the struct.
/// * Internal data type.
/// * Multiple field sections with:
///   * Name of the field.
///   * Type of the field.
///   * Offset of the first bit (lsb) in the internal value.
///   * Bit mask to apply _at the provided offset_.
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
/// bit_struct!(pub OamEntry, u32,
///     { pos_x: u8 @ 0; 0xFF }
///     { pos_y: u8 @ 8; 0xFF }
///     { pos_x_neg: u8 @ 27; 0b1 }
///     { pos_y_neg: u8 @ 28; 0b1 }
///     { char_table_index: u8 @ 16 ; 0xFF }
///     { palette_table_index: u8 @ 24 ; 0b111 }
///     { flip_x: u8 @ 29; 0b1 }
///     { flip_y: u8 @ 30; 0b1 }
/// );
/// ```
#[macro_export]
macro_rules! bit_struct {
    ( $vis:vis $struct_name:ident, $from_type:ty, $( { $field_name:ident : $field_type:ident @ $field_shift:expr ; $field_mask:expr } )* ) => {
        #[allow(dead_code)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        $vis struct $struct_name {
            value: $from_type,
        }

        paste! {
            #[allow(dead_code)]
            impl $struct_name {
                $(
                    #[inline(always)]
                    $vis fn $field_name(&self) -> $field_type {
                        ((self.value >> $field_shift) & $field_mask) as $field_type
                    }

                    #[inline(always)]
                    $vis fn [<set_ $field_name>](&mut self, val: $field_type) {
                        let masked_val = val & $field_mask;
                        // Make sure the provided value does not exceed the mask range.
                        assert_eq!(val, masked_val, "Provided value for {} should not exceed {}, but is {}.", stringify!([<set_ $field_name>]), $field_mask, val);

                        // Clear the backing bits.
                        let window = ($field_mask as $from_type) << $field_shift;
                        let cleared = self.value ^ (self.value & window);
                        // Apply the provided value.
                        self.value = cleared | ((masked_val as $from_type) << $field_shift);
                    }
                )*
            }
        }

        impl From<$from_type> for $struct_name {
            fn from(value: $from_type) -> Self {
                Self {
                    value
                }
            }
        }
    }
}