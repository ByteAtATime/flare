use rustyscript::{Module, Runtime, RuntimeOptions};

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

    let module2 = Module::new(
        "plugin.js",
        "
        const raycast = require('@raycast/api');
        raycast.showToast('Hello from Raycast API!');
        ",
    );

    runtime.load_module(&module)?;
    runtime.load_module(&module2)?;
    Ok(())
}
