mod apps;
mod clipboard;
mod components;
mod deep_link;
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
}

fn boot() -> (State, iced::Task<Message>) {
    (State::new(), iced::Task::none())
}

fn dev_boot() -> (State, iced::Task<Message>) {
    let (id, open) = window::open(window::Settings::default());
    (State::new(), open.map(move |_| Message::WindowOpened(id)))
}

fn daemon_view<'a>(state: &'a State, window: window::Id) -> iced::Element<'a, Message> {
    if state.settings_window_id == Some(window) {
        return screens::settings::settings_view(&state.extensions, &state.preferences)
            .map(Message::Settings);
    }
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
    let keyboard_sub =
        iced::keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)));

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

    let escape_sub = iced::event::listen_with(|event, _status, _id| {
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

    let result = iced_layershell::daemon(boot, namespace, update, daemon_view)
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
        .map_err(|e| e.to_string());

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
