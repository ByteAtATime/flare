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
use iced::futures::channel::mpsc;
use iced::futures::{self, SinkExt, StreamExt, stream};
use iced::window;
use std::sync::{Arc, Mutex};
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

fn message_stream() -> impl futures::Stream<Item = Message> {
    let receiver = {
        let mut guard = globals::RECEIVER.lock().unwrap();
        guard.take()
    };

    futures::stream::unfold(receiver, |state| async move {
        if let Some(mut receiver) = state {
            if let Some(msg) = receiver.next().await {
                return Some((msg, Some(receiver)));
            }
        } else {
            futures::future::pending::<()>().await;
        }
        None
    })
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

    let message_sub = Subscription::run(message_stream);

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
        message_sub,
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

fn setup_channels() {
    let (sender, receiver) = mpsc::unbounded();
    *globals::SENDER.lock().unwrap() = Some(sender);
    *globals::RECEIVER.lock().unwrap() = Some(receiver);

    let (image_sender, image_receiver) = std::sync::mpsc::channel::<String>();
    *globals::IMAGE_LOADER.lock().unwrap() = Some(image_sender);

    let shared_receiver = Arc::new(Mutex::new(image_receiver));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = reqwest::Client::builder()
                .pool_max_idle_per_host(20)
                .tcp_nodelay(true)
                .build()
                .unwrap();

            let semaphore = Arc::new(tokio::sync::Semaphore::new(10));

            loop {
                let url = {
                    let lock = shared_receiver.lock().unwrap();
                    match lock.recv() {
                        Ok(u) => u,
                        Err(_) => break,
                    }
                };

                let client = client.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    let _permit = permit;

                    match image_cache::fetch_and_cache(&client, url.clone()).await {
                        Ok(handle) => {
                            let sender = globals::SENDER.lock().unwrap().clone();
                            if let Some(mut s) = sender {
                                let _ = s.send(Message::ImageLoaded(url, handle)).await;
                            }
                        }
                        Err(_) => {
                            image_cache::clear_pending(&url);
                        }
                    }
                });
            }
        });
    });
}

fn run_application() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if let Err(e) = deep_link::register_all() {
            eprintln!("Failed to register deep links: {}", e);
        }
    }

    setup_channels();

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
    setup_channels();

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

fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    if ipc::is_daemon_running() {
        eprintln!("Daemon is already running");
        return Ok(());
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if let Err(e) = deep_link::register_all() {
            eprintln!("Failed to register deep links: {}", e);
        }
    }

    setup_channels();

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
