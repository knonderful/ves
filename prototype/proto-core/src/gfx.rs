use euclid::default::{Point2D, Size2D, Rect};

pub type CoordinateType = u32;
pub type Position = Point2D<CoordinateType>;
pub type Dimensions = Size2D<CoordinateType>;
pub type Rectangle = Rect<CoordinateType>;

pub trait Pixel: From<(u8, u8, u8)> + From<(u8, u8, u8, u8)> {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;

    fn set_rgb(&mut self, r: u8, g: u8, b: u8);
    fn set_rgba(&mut self, r: u8, g: u8, b: u8, a: u8);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelArgb8888 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const ALPHA_OPAQUE: u8 = 255;

impl Pixel for PixelArgb8888 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, ALPHA_OPAQUE)
    }

    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.set_rgba(r, g, b, ALPHA_OPAQUE);
    }

    fn set_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.a = a;
    }
}

impl From<(u8, u8, u8)> for PixelArgb8888 {
    fn from(components: (u8, u8, u8)) -> Self {
        Self::from_rgb(components.0, components.1, components.2)
    }
}

impl From<(u8, u8, u8, u8)> for PixelArgb8888 {
    fn from(components: (u8, u8, u8, u8)) -> Self {
        Self::from_rgba(components.0, components.1, components.2, components.3)
    }
}

impl Default for PixelArgb8888 {
    fn default() -> Self {
        Self::from_rgb(0, 0, 0)
    }
}

pub trait Surface {
    type PixelType;

    fn data(&self) -> &[Self::PixelType];

    fn data_mut(&mut self) -> &mut [Self::PixelType];

    fn width(&self) -> CoordinateType;

    fn height(&self) -> CoordinateType;

    fn get_index_wrapped(&self, position: Position) -> usize {
        let width = self.width();
        let height = self.height();
        let x = position.x % width;
        let y = position.y & height;
        (y * width + x) as usize
    }
}

#[macro_export]
macro_rules! surface {
    ($struct_name: ident, $width: expr, $height: expr) => {
        pub struct $struct_name<T> {
            data: [T; $width * $height],
        }

        impl<T: Default + Copy> $struct_name<T> {
            pub fn window(&self, rectangle: crate::gfx::Rectangle) -> crate::gfx::SurfaceWindow<T> {
                crate::gfx::SurfaceWindow::new(self, rectangle)
            }
        }

        impl<T: Default + Copy> Default for $struct_name<T> {
            fn default() -> Self {
                Self {
                    data: [Default::default(); $width * $height],
                }
            }
        }

        impl<T> crate::gfx::Surface for $struct_name<T> {
            type PixelType = T;

            fn data(&self) -> &[T] {
                &self.data
            }
        
            fn data_mut(&mut self) -> &mut [T] {
                &mut self.data
            }
        
            fn width(&self) -> crate::gfx::CoordinateType {
                $width as crate::gfx::CoordinateType
            }
        
            fn height(&self) -> crate::gfx::CoordinateType {
                $height as crate::gfx::CoordinateType
            }
        }
    }
}

pub struct SurfaceWindow<'surf, T> {
    surface: &'surf Surface<PixelType=T>,
    start: Position,
    end: Position,
}

impl<'surf, T> SurfaceWindow<'surf, T> {
    pub fn new(surface: &'surf Surface<PixelType=T>, rectangle: Rectangle) -> Self {
        // The underlying type from the euclid crate takes the maximum position as _inclusive_ (probably
        // because it has to be able to work with float-based coordinates, in which case that makes sense).
        // For our case that doesn't make sense, so we deduct 1 from both coordinate elements.
        let start = rectangle.origin;
        let end = Position::new(rectangle.max_x() - 1, rectangle.max_y() - 1);

        Self {
            surface,
            start,
            end,
        }
    }

    pub fn iter(&self) -> SurfaceIter<T> {
        SurfaceIter::new(self.surface, self.start, self.end)
    }
}

pub struct SurfaceIter<'surf, T> {
    surface: &'surf Surface<PixelType=T>,
    origin_x: CoordinateType,
    position: Position,
    final_position: Position,
}

impl<'surf, T> SurfaceIter<'surf, T> {
    fn new(surface: &'surf Surface<PixelType=T>, start: Position, end: Position) -> Self {
        Self {
            surface,
            origin_x: start.x,
            position: start,
            final_position: end,
        }
    }
}

impl<'surf, T> Iterator for SurfaceIter<'surf, T> {
    type Item = &'surf T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position.y > self.final_position.y {
            return None;
        }

        let index = self.surface.get_index_wrapped(self.position);
        let pixel = &self.surface.data()[index];

        if self.position.x == self.final_position.x {
            self.position.x = self.origin_x;
            self.position.y += 1;
        } else {
            self.position.x += 1;
        }

        Some(pixel)
    }
}



pub struct SurfaceIterMut<'surf, T> {
    surface: &'surf mut Surface<PixelType=T>,
    origin_x: CoordinateType,
    position: Position,
    final_position: Position,
}

impl<'surf, T> SurfaceIterMut<'surf, T> {
    fn new(surface: &'surf mut Surface<PixelType=T>, start: Position, end: Position) -> Self {
        Self {
            surface,
            origin_x: start.x,
            position: start,
            final_position: end,
        }
    }
}

impl<'surf, T> Iterator for SurfaceIterMut<'surf, T> {
    type Item = &'surf mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position.y > self.final_position.y {
            return None;
        }

        let index = self.surface.get_index_wrapped(self.position);
        // We need to "reset" the lifetime in order to satisfy the 'surf lifetime.
        // This is safe because Surface guarantees that the index is within its data buffer.
        let pixel = unsafe {
            let x = self.surface.data_mut().as_mut_ptr().add(index);
            &mut *x
        };

        if self.position.x == self.final_position.x {
            self.position.x = self.origin_x;
            self.position.y += 1;
        } else {
            self.position.x += 1;
        }

        Some(pixel)
    }
}





pub type FrameBufferPixel = PixelArgb8888;

pub struct FrameBuffer {
    dimensions: Dimensions,
    data: Vec<FrameBufferPixel>,
}

impl FrameBuffer {
    pub fn new(dimensions: Dimensions) -> Self {
        FrameBuffer {
            dimensions,
            data: vec!(Default::default(); dimensions.area() as usize),
        }
    }

    pub fn window(&mut self, rectangle: Rectangle) -> FrameBufferWindow {
        FrameBufferWindow::new(self, rectangle)
    }

    pub fn width(&self) -> CoordinateType {
        self.dimensions.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> CoordinateType {
        self.dimensions.height
    }

    pub fn raw_data(&self) -> &[u8] {
        unsafe {
            // We are in control of FrameBufferPixel. It is aligned in ARGB8888 format.
            // Defining the frame buffer data in this type (instead of u8) is more ergonomic
            // everywhere in our code, but it does require this cast to be efficient.
            std::mem::transmute::<&[FrameBufferPixel], &[u8]>(self.data.as_slice())
        }
    }
}

pub struct FrameBufferWindow<'fb> {
    framebuffer: &'fb mut FrameBuffer,
    origin_x: CoordinateType,
    position: Position,
    final_position: Position,
}

impl<'fb> FrameBufferWindow<'fb> {
    fn new(framebuffer: &'fb mut FrameBuffer, rectangle: Rectangle) -> Self {
        // The underlying type from the euclid crate takes the maximum position as _inclusive_ (probably
        // because it has to be able to work with float-based coordinates, in which case that makes sense).
        // For our case that doesn't make sense, so we deduct 1 from both coordinate elements.
        let final_position = (rectangle.max() - Position::new(1, 1)).to_point();
        let position = rectangle.origin;

        assert!(final_position.x < framebuffer.width());
        assert!(final_position.y < framebuffer.height());

        Self {
            framebuffer,
            origin_x: position.x,
            position,
            final_position,
        }
    }
}

impl<'fb> Iterator for FrameBufferWindow<'fb> {
    type Item = &'fb mut FrameBufferPixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position.y > self.final_position.y {
            return None;
        }

        let pitch = self.framebuffer.width() as usize;
        let index = self.position.y as usize * pitch + self.position.x as usize;

        // We need to "reset" the lifetime in order to get to the 'fb lifetime.
        // This is safe because the constructor guarantees that the rectangle is within the data buffer.
        let pixel = unsafe {
            let x = self.framebuffer.data.as_mut_ptr().add(index);
            &mut *x
        };

        if self.position.x == self.final_position.x {
            self.position.x = self.origin_x;
            self.position.y += 1;
        } else {
            self.position.x += 1;
        }

        Some(pixel)
    }
}