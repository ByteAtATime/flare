use crate::globals::SENDER;
use crate::message::Message;
use crate::types::{ToastOptions, Tree};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};

pub fn setup_and_run(mut callback_receiver: mpsc::UnboundedReceiver<String>) {
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
            import { React, updateContainer, invokeCallback } from './renderer.js';
    
            const PluginRoot = module.exports.default;
            const AppElement = React.createElement(PluginRoot);
            updateContainer(AppElement, () => {
                console.log("initial render callback fired!");
            });

            export { invokeCallback };
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

    let command_runner_handle = runtime.load_module(&command_runner).unwrap();

    loop {
        match callback_receiver.try_next() {
            Ok(Some(callback_id)) => {
                let result: Result<Value, _> = runtime.call_function(
                    Some(&command_runner_handle),
                    "invokeCallback",
                    &[Value::String(callback_id)],
                );
                if let Err(e) = result {
                    eprintln!("callback died: {:?}", e);
                }
            }
            Ok(None) => break,
            Err(_) => {}
        }
    }
}
