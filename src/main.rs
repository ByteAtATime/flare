use rustyscript::{Module, Runtime, RuntimeOptions};

fn main() -> Result<(), rustyscript::Error> {
    let mut runtime = Runtime::new(RuntimeOptions {
        ..Default::default()
    })?;

    let module = Module::new(
        "setup.js",
        "
        import { createRequire } from 'module';
        const nodeRequire = createRequire(import.meta.url);

        globalThis.require = (moduleName) => {
            return nodeRequire(moduleName);
        };
        
        globalThis.module = { exports: {} };
        ",
    );

    let module2 = Module::new(
        "plugin.js",
        "
        const fs = require('fs');
        module.exports = {
            content: fs.readFileSync('./src/main.rs', 'utf-8')
        };
        ",
    );

    let module3 = Module::new(
        "module.js",
        "
        console.log(module.exports.content);
        ",
    );

    runtime.load_module(&module)?;
    runtime.load_module(&module2)?;
    runtime.load_module(&module3)?;
    Ok(())
}
