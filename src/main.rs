use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};

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
        println!(
            "ooh new toast from js: {}",
            args[0].as_str().unwrap_or_default()
        );
        Ok(Value::Null)
    })?;

    runtime.load_module(&module)?;
    runtime.load_module(&module2)?;

    let tokio_runtime = runtime.tokio_runtime();

    tokio_runtime.block_on(async { runtime.load_module_async(&command_runner).await })?;

    Ok(())
}
