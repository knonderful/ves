#![allow(dead_code)] // TODO: Remove this

pub mod geom;
pub mod surface;
pub mod sprite;

#[macro_export]
macro_rules! sized_surface {
    ($vis:vis $name:ident, $data_type:ty, $width:expr, $height:expr, $default_value:expr) => {
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

        impl $crate::surface::Surface for $name {
            type DataType = $data_type;

            #[inline(always)]
            fn size(&self) -> Size {
                $crate::geom::Size::new($width, $height)
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
            type Input = $crate::geom::Point;

            #[inline(always)]
            fn offset(&self, value: Self::Input) -> Option<usize> {


                if value.x >= $width || value.y >= $height {
                    None
                } else {
                    Some($crate::surface::IntoUsize::into_usize(value.y * $width + value.x))
                }
            }
        }
    }
}
