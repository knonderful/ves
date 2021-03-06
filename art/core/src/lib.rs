#![allow(dead_code)] // TODO: Remove this

use crate::geom_art::Size;
use crate::sprite::{Palette, Sprite, Tile};
use crate::surface::Surface;

pub mod geom_art;
pub mod movie;
pub mod sprite;
pub mod surface;

/// Macro for creating [`surface::Surface`] implementations that do no require any allocation.
///
/// # Parameters
/// * `vis`: Output type visibility.
/// * `name`: Output type name.
/// * `data_type`: Data type of an element in the surface.
/// * `width`: Width of the surface in pixels.
/// * `height`: Height of the surface in pixels.
/// * `default_value`: Default element value.
#[macro_export]
macro_rules! sized_surface {
    ($vis:vis $name:ident, $data_type:ty, $space_unit_type:ty, $width:expr, $height:expr, $default_value:expr) => {
        #[doc = concat!("A [`Sized`] implementation of [`Surface`] of ", stringify!($width), "x", stringify!($height), " pixels.")]
        #[derive(Clone, Debug, Eq, PartialEq)]
        $vis struct $name {
            data: [$data_type; $width * $height],
        }

        impl $name {
            /// Creates a new instance.
            pub fn new() -> Self {
                Self {
                    data: [$default_value; $width * $height],
                }
            }
        }

        impl $crate::surface::Surface<$space_unit_type> for $name {
            type DataType = $data_type;

            #[inline(always)]
            fn size(&self) -> ves_geom::Size<$space_unit_type> {
                ves_geom::Size::new($width, $height)
            }

            #[inline(always)]
            fn data(&self) -> &[Self::DataType] {
                &self.data
            }

            #[inline(always)]
            fn data_mut(&mut self) -> &mut [Self::DataType] {
                &mut self.data
            }
        }

        impl $crate::surface::Offset for $name {
            type Input = ves_geom::Point<$space_unit_type>;

            #[inline(always)]
            fn offset(&self, value: impl Into<Self::Input>) -> Option<usize> {
                let value: Self::Input = value.into();
                let size = self.size();
                if value.x >= size.width || value.y >= size.height {
                    None
                } else {
                    Some((value.y * size.width + value.x).into())
                }
            }
        }
    }
}
