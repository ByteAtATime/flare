mod apps;
mod clipboard;
mod clipboard_history;
mod components;
mod deep_link;
mod encryption;
mod extensions;
mod frecency;
mod globals;
mod icons;
mod image_cache;
mod ipc;
mod message;
mod oauth;
mod position;
mod preferences;
mod runtime;
mod screens;
mod selection;
mod settings;
#[cfg(feature = "soulver")]
mod soulver;
mod state;
mod storage;
mod theme;
mod transport;
mod types;
mod update;
mod utils;
mod view;
mod window_title;

use clap::{Parser, Subcommand};
use iced::Subscription;
use iced::futures::{self, stream};
use iced::window;
#[cfg(target_os = "linux")]
use iced_layershell::reexport::{KeyboardInteractivity, Layer};
#[cfg(target_os = "linux")]
use iced_layershell::settings::{LayerShellSettings, StartMode};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

use crate::message::Message;
use crate::runtime::SidecarListener;
use crate::state::State;
use crate::transport::SidecarReader;
use crate::update::update;
use crate::view::view;

#[derive(Parser)]
#[command(name = "flare")]
#[command(about = "A Raycast-compatible launcher for Linux")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    Daemon,
    Toggle,
    Dev,
    Settings,
    Confetti,
}

fn boot() -> (State, iced::Task<Message>) {
    (State::new(), iced::Task::none())
}

fn dev_boot() -> (State, iced::Task<Message>) {
    let (id, open) = window::open(window::Settings::default());
    (State::new(), open.map(move |_| Message::WindowOpened(id)))
}

fn daemon_view<'a>(state: &'a State, _window: window::Id) -> iced::Element<'a, Message> {
    view(state)
}

fn sidecar_stream(reader: Arc<AsyncMutex<SidecarReader>>) -> impl futures::Stream<Item = Message> {
    stream::unfold(reader, |reader| async move {
        let mut guard = reader.lock().await;
        let response = guard.read_next().await;
        drop(guard); // once we're done reading, we can unlock the lock (idk what this is called)

        match response {
            Some(resp) => Some((Message::SidecarMessage(resp), reader)),
            None => None,
        }
    })
}

fn subscription(state: &State) -> Subscription<Message> {
    let keyboard_sub = iced::keyboard::listen().filter_map(|event| match event {
        iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
            Some(Message::KeyPressed(key, modifiers))
        }
        _ => None,
    });

    let sidecar_sub = if let Some(reader) = &state.reader {
        Subscription::run_with(
            SidecarListener {
                reader: reader.clone(),
            },
            |listener| {
                let reader = listener.reader.clone();
                sidecar_stream(reader)
            },
        )
    } else {
        Subscription::none()
    };

    let window_close_sub = window::close_events().map(Message::WindowClosed);

    let animation_sub = if state.action_panel.animation.start_time.is_some() {
        window::frames().map(Message::Tick)
    } else {
        Subscription::none()
    };

    let escape_sub = iced::event::listen_with(|event, _status, id| {
        if let iced::Event::Window(iced::window::Event::Opened {
            position: _,
            size: _,
        }) = event
        {
            return Some(Message::WindowOpened(id));
        }

        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            modifiers,
            ..
        }) = event
        {
            if modifiers.is_empty() {
                return Some(Message::EscapePressed);
            }
        }
        None
    });

    let ipc_sub = ipc::subscription();

    let mut subs = vec![
        sidecar_sub,
        window_close_sub,
        animation_sub,
        escape_sub,
        ipc_sub,
    ];

    if state.window_id.is_some() {
        subs.push(keyboard_sub);
    }

    Subscription::batch(subs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let deep_link = if cli.args.is_empty() {
        deep_link::get_current()
    } else {
        cli.args.first().cloned()
    };

    if let Some(link) = &deep_link {
        if deep_link::is_oauth_redirect(link) {
            if ipc::is_daemon_running() {
                ipc::send_oauth_redirect(link)?;
            }
            return Ok(());
        }

        if deep_link::is_confetti_link(link) {
            return run_confetti();
        }
    }

    match cli.command {
        Some(Command::Toggle) => {
            if ipc::is_daemon_running() {
                ipc::send_toggle()?;
            } else {
                eprintln!("You have to run `flare daemon` first!");
            }
            Ok(())
        }
        Some(Command::Daemon) => run_daemon(),
        Some(Command::Dev) => run_dev(),
        Some(Command::Settings) => settings::run(),
        Some(Command::Confetti) => run_confetti(),
        None => run_application(),
    }
}

fn run_application() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if let Err(e) = deep_link::register_all() {
            eprintln!("Failed to register deep links: {}", e);
        }
    }

    clipboard_history::init();

    iced::application(State::new, update, view)
        .subscription(subscription)
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .font(include_bytes!("./assets/icons.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn run_dev() -> Result<(), Box<dyn std::error::Error>> {
    clipboard_history::init();

    iced::daemon(dev_boot, update, daemon_view)
        .subscription(subscription)
        .title("Flare (Dev)")
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .font(include_bytes!("./assets/icons.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn namespace() -> String {
    String::from("flare")
}

#[cfg(target_os = "linux")]
fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    if ipc::is_daemon_running() {
        eprintln!("Daemon is already running");
        return Ok(());
    }

    if let Err(e) = deep_link::register_all() {
        eprintln!("Failed to register deep links: {}", e);
    }

    clipboard_history::init();

    let flare_settings = preferences::FlareSettings::load();

    let result = if flare_settings.use_layer_shell {
        iced_layershell::daemon(boot, namespace, update, daemon_view)
            .subscription(subscription)
            .title(|_state, _window| Some(String::from("Flare")))
            .font(include_bytes!("./assets/Inter.ttf").as_slice())
            .font(include_bytes!("./assets/icons.ttf").as_slice())
            .default_font(iced::Font::DEFAULT)
            .layer_settings(LayerShellSettings {
                start_mode: StartMode::Background,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                layer: Layer::Overlay,
                ..Default::default()
            })
            .run()
            .map_err(|e| e.to_string())
    } else {
        iced::daemon(boot, update, daemon_view)
            .subscription(subscription)
            .title("Flare")
            .font(include_bytes!("./assets/Inter.ttf").as_slice())
            .font(include_bytes!("./assets/icons.ttf").as_slice())
            .default_font(iced::Font::DEFAULT)
            .run()
            .map_err(|e| e.to_string())
    };

    ipc::cleanup();
    result?;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    if ipc::is_daemon_running() {
        eprintln!("Daemon is already running");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = deep_link::register_all() {
            eprintln!("Failed to register deep links: {}", e);
        }
    }

    clipboard_history::init();

    let result = iced::daemon(boot, update, daemon_view)
        .subscription(subscription)
        .title("Flare")
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .font(include_bytes!("./assets/icons.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string());

    ipc::cleanup();
    result?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_confetti() -> Result<(), Box<dyn std::error::Error>> {
    use crate::components::confetti::{Manager, Options};
    use iced::widget::canvas;
    use iced::{Color, Length, Point, Rectangle, Size, Task, time, window};
    use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use iced_layershell::settings::{LayerShellSettings, StartMode};
    use iced_layershell::to_layer_message;

    struct ConfettiState {
        manager: Manager,
        window_id: Option<window::Id>,
        fired: bool,
    }

    #[to_layer_message]
    #[derive(Debug, Clone)]
    enum ConfettiMessage {
        Tick,
        WindowOpened(window::Id),
        MonitorSize(Option<Size>),
    }

    fn boot() -> (ConfettiState, Task<ConfettiMessage>) {
        (
            ConfettiState {
                manager: Manager::new(),
                window_id: None,
                fired: false,
            },
            Task::none(),
        )
    }

    fn update(state: &mut ConfettiState, message: ConfettiMessage) -> Task<ConfettiMessage> {
        match message {
            ConfettiMessage::Tick => {
                state.manager.update();
                if state.fired && state.manager.is_done() {
                    return iced::exit();
                }
                Task::none()
            }
            ConfettiMessage::WindowOpened(id) => {
                state.window_id = Some(id);
                window::size(id).map(|arg0: iced::Size| ConfettiMessage::MonitorSize(Some(arg0)))
            }
            ConfettiMessage::MonitorSize(size) => {
                if state.fired {
                    return Task::none();
                }
                state.fired = true;

                // distance = (velocity * cos(angle))/(1 - decay)
                // rearranging for velocity, distance(1-decay)/cos(angle)
                // (width/2)(1-0.97)/cos(45) = width / 47.1
                let size = size.unwrap_or(Size::new(1920.0, 1080.0));
                let options = Options {
                    particle_count: 300,
                    spread: 45.0,
                    start_velocity: size.width / 47.1,
                    gravity: size.width / 960.0,
                    decay: 0.97,
                    ticks: 160.0, // about 2.5 seconds, seems like a reasonable duration
                    origin: Point { x: 1.0, y: 1.0 },
                    scalar: 1.2,
                    ..Default::default()
                };

                state.manager.fire_with_bounds(
                    Options {
                        angle: 135.0,
                        origin: Point { x: 1.0, y: 1.0 },
                        ..options.clone()
                    },
                    Rectangle::new(Point::ORIGIN, size),
                );

                state.manager.fire_with_bounds(
                    Options {
                        angle: 45.0,
                        origin: Point { x: 0.0, y: 1.0 },
                        ..options
                    },
                    Rectangle::new(Point::ORIGIN, size),
                );

                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(state: &ConfettiState) -> iced::Element<'_, ConfettiMessage> {
        iced::widget::container(
            canvas(&state.manager)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(Color::TRANSPARENT.into()),
            ..Default::default()
        })
        .into()
    }

    fn subscription(state: &ConfettiState) -> iced::Subscription<ConfettiMessage> {
        use std::time::Duration;

        let tick = time::every(Duration::from_millis(1000 / 60)).map(|_| ConfettiMessage::Tick);

        let window_events = if state.window_id.is_none() {
            iced::event::listen_with(|event, _status, id| {
                if let iced::Event::Window(iced::window::Event::Opened { .. }) = event {
                    return Some(ConfettiMessage::WindowOpened(id));
                }
                None
            })
        } else {
            iced::Subscription::none()
        };

        iced::Subscription::batch([tick, window_events])
    }

    iced_layershell::application(boot, || String::from("flare-confetti"), update, view)
        .subscription(subscription)
        .style(|_state, theme| iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        })
        .layer_settings(LayerShellSettings {
            start_mode: StartMode::Active,
            layer: Layer::Overlay,
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: 0,
            events_transparent: true,
            ..Default::default()
        })
        .run()
        .map_err(|e| e.to_string().into())
}

#[cfg(not(target_os = "linux"))]
fn run_confetti() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Confetti is not supported yet!");
    Ok(())
}
