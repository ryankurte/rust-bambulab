use std::fmt::Display;

/// Bed levelling point
#[derive(Clone, PartialEq, Debug)]
pub struct Point {
    /// X location
    pub x: f32,
    /// Y location
    pub y: f32,
    /// z offset
    pub c: f32,
    /// z variance
    pub d: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32, c: f32, d: f32) -> Self {
        Self { x, y, c, d }
    }
}

/// Bed levelling map
pub struct LevelMap {
    pub xs: Vec<f32>,
    pub ys: Vec<f32>,
    pub points: Vec<Point>,
}

impl LevelMap {
    /// Create a new LevelMap from a list of levelling points
    pub fn new(points: Vec<Point>) -> Self {
        // Fetch X and Y axes'
        let mut xs = Vec::<f32>::new();
        let mut ys = Vec::<f32>::new();

        for Point { x, y, .. } in points.iter() {
            if !xs.iter().any(|v| &v == &x) {
                xs.push(*x);
            }

            if !ys.iter().any(|v| &v == &y) {
                ys.push(*y);
            }
        }

        Self { xs, ys, points }
    }

    pub fn value(&self, x: f32, y: f32) -> Option<&Point> {
        self.points.iter().find(|v| v.x == x && v.y == y)
    }
}

/// Display a [LevelMap] in the terminal
impl Display for LevelMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write X indicies
        write!(f, "        ")?;
        for x in &self.xs {
            write!(f, "{x:<7.01}")?;
        }
        writeln!(f)?;

        // Write points line by line
        for y in &self.ys {
            write!(f, "{y:>5.01}: ")?;

            for x in &self.xs {
                let v = self.value(*x, *y);

                match v {
                    Some(v) => write!(f, "{:>6.03} ", v.c)?,
                    None => write!(f, "????? ")?,
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
