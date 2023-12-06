#[derive(Default, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Default, Clone, Copy)]
pub struct Area {
    pub start: Position,
    pub end: Position,
}

impl Area {
    pub fn get_width(&self) -> f64 {
        let diff = self.start.x - self.end.x;
        diff.abs()
    }
    pub fn get_height(&self) -> f64 {
        let diff = self.start.y - self.end.y;
        diff.abs()
    }
}
