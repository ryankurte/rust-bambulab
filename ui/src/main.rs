
use iced::{
    alignment::{self, Horizontal, Alignment},
    Application,
    Length,
    Command,
    Settings, Theme, Element, Point,
    widget::canvas::{Cursor, Event},
};
use iced_native::{
    subscription::Recipe,
    widget::{
        Button, Column, PickList, ProgressBar, Row, Slider, VerticalSlider, Text, TextInput, Space,
    }, row, column,
};

use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend, ChartBuilder};

use num_traits::float::FloatConst;

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    App::run(Settings{
        antialiasing: true,
        window: iced::window::Settings{
            resizable: true,
            ..Default::default()
        },
        ..Default::default()
    })?;

    Ok(())
}

struct App {
    bc: BedChart,
}

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Yaw(f64),
    Pitch(f64),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {

        (Self{ bc: BedChart::new() }, iced::Command::none())
    }

    fn title(&self) -> String {
        "VMouse GUI".to_string()
    }

    // Handle events
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Pitch(p) => self.bc.pitch = p,
            Message::Yaw(y) => self.bc.yaw = y,
            _ => (),
        }

        iced::Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        column!(
            Row::new()
                .padding(10)
                .push(
                    row!(self.bc.view())
                    .width(Length::FillPortion(10))
                )
                .push(
                    row!(VerticalSlider::new(
                        0.0..=1.0,
                        self.bc.pitch,
                        move |x| Message::Pitch(x),
                    ).step(0.01))
                    .width(Length::Shrink)
                )
                .height(Length::FillPortion(10)),
            Row::new()
                .padding(10)
                .push(
                    row!(Slider::new(
                        0.0..=f64::PI()/2.0,
                        self.bc.yaw,
                        move |x| Message::Yaw(x),
                    ).step(0.01))
                    .height(Length::Shrink)
                    .width(Length::FillPortion(10))
                )
                .push(
                    row!()
                    .width(Length::FillPortion(1))
                )
                .height(Length::Shrink)
        )
        .into()
    }
}

struct BedChart {
    pub yaw: f64,
    pub pitch: f64,
}

impl BedChart {
    fn new() -> Self {
        Self{
            yaw: 0.5,
            pitch: 0.1,
        }
    }

    fn view(&self)->Element<'_, Message> {
        ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<M> Chart<M> for BedChart {
    type State =  ();

    fn build_chart<DB:DrawingBackend>(&self, state: &Self::State, mut builder: ChartBuilder<DB>) {
        let x_axis = (-3.0..3.0).step(0.1);
        let y_azis = -3.0..3.0;
        let z_axis = (-3.0..3.0).step(0.1);

        let mut chart = builder
            .build_cartesian_3d(x_axis.clone(), y_azis.clone(), z_axis.clone()).unwrap();

        chart.with_projection(|mut pb| {
            pb.yaw = self.yaw;
            pb.pitch = self.pitch;
            pb.scale = 0.9;
            pb.into_matrix()
        });
        
        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .draw().unwrap();

        chart.draw_series(
            SurfaceSeries::xoz(
                (-15..=15).map(|x| x as f64 / 5.0),
                (-15..=15).map(|x| x as f64 / 5.0),
                pdf,
            )
            .style(BLUE.mix(0.8)),
        ).unwrap();

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw().unwrap();
    }
}

fn pdf(x: f64, y: f64) -> f64 {
    const SDX: f64 = 0.1;
    const SDY: f64 = 0.1;
    const A: f64 = 5.0;
    let x = x as f64 / 10.0;
    let y = y as f64 / 10.0;
    A * (-x * x / 2.0 / SDX / SDX - y * y / 2.0 / SDY / SDY).exp()
}
