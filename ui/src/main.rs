#![feature(const_trait_impl)]

use std::hash::Hash;

use clap::Parser;
use futures::stream::StreamExt;
use iced::{
    advanced::subscription::{EventStream, Recipe},
    futures::stream::BoxStream,
    widget::{column, Row},
    Application, Command, Element, Length, Settings, Theme,
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter, FmtSubscriber};

use bambu::{ConnectOpts, Printer};

mod chart;
use chart::BedChart;

mod message;
pub use message::Message;

mod control;
use control::Controls;

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Args {
    #[clap(flatten)]
    opts: ConnectOpts,

    /// Enable verbose logging
    #[clap(long, default_value = "debug")]
    log_level: LevelFilter,
}

#[derive(Clone, Debug, PartialEq, displaydoc::Display)]
pub enum Status {
    /// Idle
    Idle,
    /// Connecting to printer
    Connecting,
    /// Connected to printer
    Connected,
    /// Disconnected from printer
    Disconnected,
}

fn main() -> anyhow::Result<()> {
    // Load arguments
    let args = Args::parse();

    // Setup logging
    let filter = EnvFilter::from_default_env()
        .add_directive("paho_mqtt=warn".parse().unwrap())
        .add_directive("wgpu_core=warn".parse().unwrap())
        .add_directive("wgpu_native=warn".parse().unwrap())
        .add_directive("wgpu_hal=warn".parse().unwrap())
        .add_directive("iced_wgpu=warn".parse().unwrap())
        .add_directive("naga=warn".parse().unwrap())
        .add_directive("cosmic_text=warn".parse().unwrap())
        .add_directive(args.log_level.into());

    let _ = FmtSubscriber::builder()
        .compact()
        .without_time()
        .with_max_level(args.log_level)
        .with_env_filter(filter)
        .try_init();

    App::run(Settings {
        antialiasing: true,
        window: iced::window::Settings {
            resizable: true,
            ..Default::default()
        },
        flags: args.opts,
        ..Default::default()
    })?;

    Ok(())
}

struct App {
    c: Controls,
    p: Option<Printer>,
    bc: BedChart,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ConnectOpts;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                c: Controls {
                    opts: flags,
                    connected: false,
                },
                p: None,
                bc: BedChart::new(),
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        "Bambu Bed Level Viewer".to_string()
    }

    // Handle events
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::SetHostname(h) => self.c.opts.hostname = h,
            Message::SetAccessCode(h) => self.c.opts.access_code = h,
            Message::Connect(opts) => return Self::connect(opts),
            Message::Connected(printer) => {
                debug!("Received printer, unpacking");
                self.p = Some(printer.clone());
                self.c.connected = true;
            }
            Message::Disconnect if self.p.is_some() => {
                let p = self.p.take().unwrap();
                return Self::disconnect(p);
            }
            Message::Disconnected => {
                self.c.connected = false;
            }

            Message::Report(data) => {
                debug!("RX {data:?}")
            }

            Message::Pitch(p) => self.bc.pitch = p,
            Message::Yaw(y) => self.bc.yaw = y,
            Message::YawPitch(y, p) => {
                self.bc.pitch = p;
                self.bc.yaw = y;
            }
            _ => (),
        }

        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        if let Some(c) = &self.p {
            iced::Subscription::from_recipe(PrinterSubscription { printer: c.clone() })
        } else {
            iced::Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        Row::new()
            .push(column!(self.c.view()).width(Length::FillPortion(4)))
            .push(column!(self.bc.view()).width(Length::FillPortion(10)))
            .into()
    }
}

impl App {
    /// Connect to a printer
    fn connect(opts: ConnectOpts) -> Command<Message> {
        Command::perform(
            async move {
                debug!("Connecting to printer: {opts:?}");
                let p = Printer::connect(opts).await?;

                debug!("Connected!");

                Ok(p)
            },
            |r: Result<Printer, anyhow::Error>| match r {
                Ok(c) => Message::Connected(c),
                Err(e) => {
                    error!("Connection failed: {:?}", e);
                    Message::Tick
                }
            },
        )
    }

    /// Disconnect from a printer
    fn disconnect(printer: Printer) -> Command<Message> {
        Command::perform(
            async move {
                debug!("Disconnecting from printer: {printer:?}");
                printer.disconnect().await
            },
            |r: Result<(), bambu::Error>| match r {
                Ok(_) => Message::Disconnected,
                Err(e) => {
                    error!("Disconnection failed: {e:?}");
                    Message::Tick
                }
            },
        )
    }
}

struct PrinterSubscription {
    printer: Printer,
}

impl Recipe for PrinterSubscription {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::Hasher) {
        self.printer.hash(state)
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Self::Output> {
        use tokio_stream::wrappers::UnboundedReceiverStream;

        let l = self.printer.listen().unwrap();

        Box::pin(UnboundedReceiverStream::from(l).map(|(t, d)| Message::Report(d)))
    }
}
