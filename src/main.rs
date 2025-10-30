use iced::Element;
use iced::widget::{button, text};
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn view(counter: &u64) -> Element<Message> {
    button(text(counter)).on_press(Message::Increment).into()
}

fn update(counter: &mut u64, message: Message) {
    match message {
        Message::Increment => *counter += 1,
    }
}

fn main() -> Result<(), rustyscript::Error> {
    let mut runtime = Runtime::new(RuntimeOptions {
        ..Default::default()
    })?;

    let renderer_module = Module::new("renderer.js", include_str!("../renderer/dist/index.js"));
    runtime.load_module(&renderer_module)?;

    let module = Module::new(
        "setup.js",
        "
        import { createRequire } from 'module';
        const nodeRequire = createRequire(import.meta.url);

        import { raycastApi } from './renderer.js';

        globalThis.require = (moduleName) => {
            if (moduleName === '@raycast/api') {
                return raycastApi;
            }
            return nodeRequire(moduleName);
        };
        
        globalThis.module = { exports: {} };
        ",
    );

    let module2 = Module::new("plugin.js", include_str!("../test/plugin.js"));

    let command_runner = Module::new(
        "runner.js",
        r#"
        await module.exports.default();
    "#,
    );

    runtime.register_function("showToast", |args| {
        println!("ooh new toast from js: {:?}", args[0]);
        Ok(Value::Null)
    })?;

    runtime.load_module(&module)?;
    runtime.load_module(&module2)?;

    let tokio_runtime = runtime.tokio_runtime();

    tokio_runtime.block_on(async { runtime.load_module_async(&command_runner).await })?;

    iced::run("A cool counter", update, view)
        .map_err(|e| rustyscript::Error::Runtime(e.to_string()))
}
