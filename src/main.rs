mod cache;
mod components;
mod deep_link;
mod extensions;
mod globals;
mod icons;
mod image_cache;
mod ipc;
mod message;
mod position;
mod runtime;
mod screens;
#[cfg(feature = "soulver")]
mod soulver;
mod state;
mod types;
mod update;
mod utils;
mod view;

use clap::{Parser, Subcommand};
use iced::Subscription;
use iced::futures::channel::mpsc;
use iced::futures::{self, SinkExt, StreamExt};
use iced::window;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::message::Message;
use crate::state::State;
use crate::update::update;
use crate::view::view;

#[derive(Parser)]
#[command(name = "flare")]
#[command(about = "A Raycast-compatible launcher for Linux")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
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
        return settings_view();
    }
    view(state)
}

fn settings_view<'a>() -> iced::Element<'a, Message> {
    iced::widget::container(iced::widget::column![iced::widget::text("Settings")])
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
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

fn subscription(state: &State) -> Subscription<Message> {
    let keyboard_sub =
        iced::keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)));

    let message_sub = Subscription::run(message_stream);

    let window_close_sub = window::close_events().map(Message::WindowClosed);

    if state.window_id.is_some() {
        Subscription::batch(vec![message_sub, keyboard_sub, window_close_sub])
    } else {
        Subscription::batch(vec![message_sub, window_close_sub])
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

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

    let (callback_sender, callback_receiver) = mpsc::unbounded::<(String, Value)>();
    *globals::RUNTIME_SENDER.lock().unwrap() = Some(callback_sender);

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

    std::thread::spawn(move || {
        runtime::run_callback_loop(callback_receiver);
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

    ipc::start_listener(|| {
        if let Some(mut sender) = globals::SENDER.lock().unwrap().clone() {
            let _ = iced::futures::executor::block_on(sender.send(Message::ToggleWindow));
        }
    })?;

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
