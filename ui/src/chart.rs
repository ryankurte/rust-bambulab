use iced::{
    event::Status,
    mouse::Cursor,
    widget::{canvas::Event, row},
    Element, Length,
};

use plotters::{prelude::*, style::colors::colormaps::VulcanoHSL};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use num_traits::float::FloatConst;

use crate::Message;
use bambu::level::{LevelMap, Point};

/// Chart for rendering bed levelling information
pub struct BedChart {
    /// Chart horizontal rotation
    pub yaw: f64,
    /// Chart vertical rotation
    pub pitch: f64,
    // Measurement points
    pub level: LevelMap,
}

fn yaw_max() -> f64 {
    f64::PI() / 2.0
}
fn pitch_max() -> f64 {
    1.0
}

impl BedChart {
    /// Create a new chart with the default points
    pub fn new() -> Self {
        Self {
            yaw: 0.5,
            pitch: 0.1,
            level: LevelMap::new(TEST_POINTS.to_vec()),
        }
    }

    /// Fetch a chart widget for rendering
    pub fn view(&self) -> Element<'_, Message> {
        row!(ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fill))
        .padding(20)
        .into()
    }
}

pub struct ChartState {
    clicked: bool,
    last: iced::Point,
}

impl Default for ChartState {
    fn default() -> Self {
        Self {
            clicked: false,
            last: iced::Point::default(),
        }
    }
}

impl Chart<Message> for BedChart {
    type State = ChartState;

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let x_axis = (0.0..256.0).step(16.0);
        let z_axis = (0.0..256.0).step(16.0);
        let y_azis = -0.5..1.0;

        let mut chart = builder
            .build_cartesian_3d(x_axis.clone(), y_azis.clone(), z_axis.clone())
            .unwrap();

        chart.with_projection(|mut pb| {
            pb.yaw = self.yaw;
            pb.pitch = self.pitch;
            pb.scale = 0.9;
            pb.into_matrix()
        });

        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .draw()
            .unwrap();

        chart
            .draw_series(
                SurfaceSeries::xoz(
                    self.level.xs.iter().map(|v| *v as f64),
                    self.level.ys.iter().map(|v| *v as f64),
                    |x, y| {
                        self.level
                            .value(x as f32, y as f32)
                            .map(|p| p.c as f64)
                            .unwrap_or_default()
                    },
                )
                .style_func(&|&v| (VulcanoHSL::get_color(v * 2.0)).into()),
            )
            .unwrap();

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()
            .unwrap();
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<Message>) {
        use iced::mouse::{Button as MouseButton, Event as MouseEvent};

        let cursor_over = cursor.position_over(bounds);

        match (event, cursor_over) {
            (Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Left)), Some(p)) => {
                state.clicked = true;
                state.last = p;

                (Status::Captured, None)
            }
            (Event::Mouse(MouseEvent::ButtonReleased(MouseButton::Left)), _) if state.clicked => {
                state.clicked = false;

                (Status::Captured, None)
            }
            (Event::Mouse(MouseEvent::CursorMoved { position }), _) if state.clicked => {
                // Compute delta
                let dy = -(position.x - state.last.x) / bounds.width * yaw_max() as f32 * 10.0;
                let dp = (position.y - state.last.y) / bounds.height * pitch_max() as f32 * 10.0;

                // Update last position
                state.last = position;

                (
                    Status::Captured,
                    Some(Message::YawPitch(
                        self.yaw + dy as f64,
                        self.pitch + dp as f64,
                    )),
                )
            }
            (Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Right)), _) => {
                (Status::Captured, Some(Message::YawPitch(0.5, 0.1)))
            }
            _ => (Status::Ignored, None),
        }
    }
}

/// Test point array for chart rendering
const TEST_POINTS: &[Point] = &[
    Point {
        x: 25.0,
        y: 25.0,
        c: 0.272,
        d: 0.045,
    },
    Point {
        x: 66.2,
        y: 25.0,
        c: 0.042,
        d: 0.037,
    },
    Point {
        x: 107.4,
        y: 25.0,
        c: 0.192,
        d: 0.043,
    },
    Point {
        x: 148.6,
        y: 25.0,
        c: 0.200,
        d: 0.039,
    },
    Point {
        x: 189.8,
        y: 25.0,
        c: 0.039,
        d: 0.041,
    },
    Point {
        x: 231.0,
        y: 25.0,
        c: 0.272,
        d: 0.048,
    },
    Point {
        x: 25.0,
        y: 67.2,
        c: 0.319,
        d: 0.045,
    },
    Point {
        x: 66.2,
        y: 67.2,
        c: 0.052,
        d: 0.040,
    },
    Point {
        x: 107.4,
        y: 67.2,
        c: 0.094,
        d: 0.041,
    },
    Point {
        x: 148.6,
        y: 67.2,
        c: 0.099,
        d: 0.041,
    },
    Point {
        x: 189.8,
        y: 67.2,
        c: 0.059,
        d: 0.045,
    },
    Point {
        x: 231.0,
        y: 67.2,
        c: 0.335,
        d: 0.052,
    },
    Point {
        x: 25.0,
        y: 109.4,
        c: 0.433,
        d: 0.055,
    },
    Point {
        x: 66.2,
        y: 109.4,
        c: 0.126,
        d: 0.040,
    },
    Point {
        x: 107.4,
        y: 109.4,
        c: 0.017,
        d: 0.039,
    },
    Point {
        x: 148.6,
        y: 109.4,
        c: 0.020,
        d: 0.039,
    },
    Point {
        x: 189.8,
        y: 109.4,
        c: 0.196,
        d: 0.044,
    },
    Point {
        x: 231.0,
        y: 109.4,
        c: 0.398,
        d: 0.054,
    },
    Point {
        x: 25.0,
        y: 151.6,
        c: 0.529,
        d: 0.066,
    },
    Point {
        x: 66.2,
        y: 151.6,
        c: 0.205,
        d: 0.050,
    },
    Point {
        x: 107.4,
        y: 151.6,
        c: 0.064,
        d: 0.043,
    },
    Point {
        x: 148.6,
        y: 151.6,
        c: 0.045,
        d: 0.040,
    },
    Point {
        x: 189.8,
        y: 151.6,
        c: 0.212,
        d: 0.046,
    },
    Point {
        x: 231.0,
        y: 151.6,
        c: 0.459,
        d: 0.066,
    },
    Point {
        x: 25.0,
        y: 193.8,
        c: 0.613,
        d: 0.071,
    },
    Point {
        x: 66.2,
        y: 193.8,
        c: 0.291,
        d: 0.053,
    },
    Point {
        x: 107.4,
        y: 193.8,
        c: 0.112,
        d: 0.044,
    },
    Point {
        x: 148.6,
        y: 193.8,
        c: 0.103,
        d: 0.050,
    },
    Point {
        x: 189.8,
        y: 193.8,
        c: 0.235,
        d: 0.052,
    },
    Point {
        x: 231.0,
        y: 193.8,
        c: 0.512,
        d: 0.081,
    },
    Point {
        x: 25.0,
        y: 236.0,
        c: 0.738,
        d: 0.097,
    },
    Point {
        x: 66.2,
        y: 236.0,
        c: 0.369,
        d: 0.060,
    },
    Point {
        x: 107.4,
        y: 236.0,
        c: 0.197,
        d: 0.055,
    },
    Point {
        x: 148.6,
        y: 236.0,
        c: 0.165,
        d: 0.054,
    },
    Point {
        x: 189.8,
        y: 236.0,
        c: 0.290,
        d: 0.056,
    },
    Point {
        x: 231.0,
        y: 236.0,
        c: 0.542,
        d: 0.100,
    },
];
