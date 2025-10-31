use iced::futures;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::{column, container, text};
use iced::{Color, Element, Font, Length, Subscription, Theme};
use rustyscript::deno_core::PollEventLoopOptions;
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};
use std::sync::Mutex;

static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);

const INTER_FONT: Font = Font::with_name("Inter");

#[derive(serde::Deserialize)]
struct ToastOptions {
    title: String,
    message: Option<String>,
    style: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Tree {
    id: String,
    children: Vec<TreeNode>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TreeNode {
    #[serde(rename = "type")]
    node_type: String,
    props: Option<Value>,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn parse_props<T: serde::de::DeserializeOwned + Default>(&self) -> T {
        self.props
            .as_ref()
            .and_then(|p| serde_json::from_value(p.clone()).ok())
            .unwrap_or_default()
    }

    fn render_children(&self) -> impl Iterator<Item = Element<'_, Message>> + '_ {
        self.children.iter().map(render_tree_node)
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct GridSectionProps {
    #[serde(default)]
    title: String,
    #[serde(default)]
    columns: Option<i32>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct GridItemProps {
    #[serde(default)]
    title: String,
    #[serde(default)]
    subtitle: Option<String>,
    #[serde(default)]
    content: Option<GridItemContent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GridItemContent {
    color: Option<GridItemColor>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GridItemColor {
    light: String,
    dark: String,
    #[serde(rename = "adjustContrast")]
    adjust_contrast: bool,
}

#[derive(Default)]
struct State {
    toast_message: String,
    tree: Option<Tree>,
}

#[derive(Debug, Clone)]
enum Message {
    UpdateToast(String),
    UpdateTree(Tree),
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}

fn render_tree_node(node: &TreeNode) -> Element<'_, Message> {
    use iced::widget::row;

    match node.node_type.as_str() {
        "Grid" => node
            .render_children()
            .fold(column![], |col, child| col.push(child))
            .into(),
        "Grid.Section" => {
            let props: GridSectionProps = node.parse_props();
            let items_row = node
                .render_children()
                .fold(row![].spacing(10), |row, child| row.push(child));

            column![text(props.title).size(16).font(INTER_FONT), items_row]
                .padding(10)
                .spacing(10)
                .into()
        }
        "Grid.Item" => {
            let props: GridItemProps = node.parse_props();
            let bg_color = props
                .content
                .as_ref()
                .and_then(|c| c.color.as_ref())
                .map(|color| parse_hex_color(&color.light))
                .unwrap_or(Color::from_rgb8(0x33, 0x33, 0x33));

            container(
                column![
                    container(text(""))
                        .width(150)
                        .height(150)
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(bg_color.into()),
                            border: iced::Border {
                                color: Color::from_rgb8(0x55, 0x55, 0x55),
                                width: 2.0,
                                radius: 8.0.into(),
                            },
                            ..Default::default()
                        }),
                    text(props.title).size(14).font(INTER_FONT),
                    text(props.subtitle.unwrap_or_default())
                        .size(12)
                        .font(INTER_FONT)
                        .style(|_theme: &Theme| text::Style {
                            color: Color::from_rgb8(0xaa, 0xaa, 0xaa).into(),
                            ..Default::default()
                        })
                ]
                .spacing(5),
            )
            .into()
        }
        _ => text("Unknown").into(),
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let content = state
        .tree
        .as_ref()
        .map(|tree| {
            tree.children
                .iter()
                .fold(column![].height(Length::Fill), |col, child| {
                    col.push(render_tree_node(child))
                })
        })
        .unwrap_or_else(|| column![].height(Length::Fill));

    container(column![
        content,
        container(
            text(&state.toast_message)
                .size(16)
                .font(INTER_FONT)
                .shaping(text::Shaping::Advanced)
        )
        .width(Length::Fill)
        .padding([0, 8])
        .center_y(40)
        .style(|_theme: &Theme| container::Style {
            background: Some(Color::from_rgb8(0x22, 0x22, 0x22).into()),
            text_color: Some(Color::WHITE),
            ..Default::default()
        })
    ])
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::UpdateToast(new_message) => state.toast_message = new_message,
        Message::UpdateTree(tree) => {
            println!("Tree update: {:?}", tree);
            state.tree = Some(tree);
        }
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    struct ToastListener;

    if let Some(receiver) = RECEIVER.lock().unwrap().take() {
        let stream = futures::stream::unfold(receiver, |mut receiver| async {
            receiver.next().await.map(|message| (message, receiver))
        });

        Subscription::run_with_id(std::any::TypeId::of::<ToastListener>(), stream)
    } else {
        Subscription::none()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::unbounded();
    *SENDER.lock().unwrap() = Some(sender);
    *RECEIVER.lock().unwrap() = Some(receiver);

    std::thread::spawn(|| {
        let mut runtime = Runtime::new(RuntimeOptions::default()).unwrap();

        let renderer_module = Module::new("renderer.js", include_str!("../renderer/dist/index.js"));
        runtime.load_module(&renderer_module).unwrap();

        let module = Module::new(
            "setup.js",
            "
            import { createRequire } from 'module';
            const nodeRequire = createRequire(import.meta.url);
    
            import { raycastApi, React, ReactJsxRuntime } from './renderer.js';
    
            globalThis.require = (moduleName) => {
                if (moduleName === '@raycast/api') {
                    return raycastApi;
                }
    
                if (moduleName === 'react') return React;
                if (moduleName === 'react/jsx-runtime') return ReactJsxRuntime;
    
                return nodeRequire(moduleName);
            };
            
            globalThis.module = { exports: {} };
            ",
        );
        runtime.load_module(&module).unwrap();

        let module2 = Module::new("plugin.js", include_str!("../test/plugin.js"));
        runtime.load_module(&module2).unwrap();

        let command_runner = Module::new(
            "runner.js",
            r#"
            import { React, updateContainer } from './renderer.js';
    
            const PluginRoot = module.exports.default;
            const AppElement = React.createElement(PluginRoot);
            updateContainer(AppElement, () => {
                console.log("initial render callback fired!");
            });
        "#,
        );

        runtime
            .register_async_function("showToast", |args| {
                Box::pin(async move {
                    if let Ok(value) = serde_json::from_value::<ToastOptions>(args[0].clone()) {
                        if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                            sender
                                .send(Message::UpdateToast(value.title.clone()))
                                .await
                                .unwrap();
                        }
                    }
                    Ok(Value::Null)
                })
            })
            .unwrap();

        runtime
            .register_async_function("updateTree", |args| {
                Box::pin(async move {
                    if let Ok(tree) = serde_json::from_value::<Tree>(args[0].clone()) {
                        if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                            sender.send(Message::UpdateTree(tree)).await.unwrap();
                        }
                    }
                    Ok(Value::Null)
                })
            })
            .unwrap();

        runtime.load_module(&command_runner).unwrap();

        runtime
            .block_on_event_loop(PollEventLoopOptions::default(), None)
            .unwrap();
    });

    iced::application("flare", update, view)
        .subscription(subscription)
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}
