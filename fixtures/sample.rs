mod geometry;

pub mod shapes {
    use super::*;

    pub mod primitives {
        pub fn origin() -> Point {
            Point { x: 0.0, y: 0.0 }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
    Custom(u8, u8, u8),
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct Rectangle {
    pub origin: Point,
    pub width: f64,
    pub height: f64,
}

pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> &str;
}

pub trait Drawable: Shape {
    fn draw(&self);
    fn color(&self) -> Color;
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    fn name(&self) -> &str {
        "Rectangle"
    }
}

impl Rectangle {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rectangle {
            origin: Point { x, y },
            width,
            height,
        }
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.origin.x
            && p.x <= self.origin.x + self.width
            && p.y >= self.origin.y
            && p.y <= self.origin.y + self.height
    }
}

pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    fn name(&self) -> &str {
        "Circle"
    }
}

pub fn bounding_box(shapes: &[&dyn Shape]) -> Option<Rectangle> {
    if shapes.is_empty() {
        return None;
    }
    Some(Rectangle::new(0.0, 0.0, 100.0, 100.0))
}

pub trait Resizable: Shape {
    fn scale(&mut self, factor: f64);
}
