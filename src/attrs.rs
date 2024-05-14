#[derive(Debug, Copy, Clone)]
pub enum Size {
    Expand,
    // Split(f32),
    Fit,
    Fixed(f32)
}

#[derive(Debug)]
enum Justify {
    Min,
    Max,
    Center
}
