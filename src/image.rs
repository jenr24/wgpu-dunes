#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Pixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}


#[macro_export]
macro_rules! sizedImage {
    ($size:expr, $name:ident, $pixel:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            size: usize,
            pixels: [[$pixel; $size]; $size],
        }
    };
}

sizedImage!(1024, Image1024, Pixel);