mod globals;
mod sourcemap;
mod userdata;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Context;
use mlua::{Compiler, Lua, LuaOptions, StdLib};

use self::globals::{create_require_global, create_task_global};
use self::sourcemap::{InnerSourcemapNode, SourcemapNode};
use self::userdata::{Game, InnerInstance};

/// Stores persistent Luau state and allows the execution of multiple Luau scripts that share a global environment
/// (`_G`). `Require`'d modules are cached just like in Roblox.
#[derive(Debug)]
pub struct LuauContext {
    lua: Lua,
    compiler: Compiler,
    sourcemap: SourcemapNode,
}

impl LuauContext {
    pub fn new(sourcemap_path: &Path, compiler: Compiler) -> anyhow::Result<Self> {
        let sourcemap = InnerSourcemapNode::new_from_path(sourcemap_path)
            .context("Failed to parse sourcemap")?;

        let lua = Lua::new_with(StdLib::ALL, LuaOptions::default())
            .context("Failed to create Luau state")?;

        // Enabling the sandbox is important because it enforces separate global environments between scripts, just like
        // Roblox. Given the primary use of rbx-cli is testing, this is important behavior to enforce.
        // TODO: Should this be configurable?
        lua.sandbox(true).context("Failed to enable Luau sandbox")?;

        let globals = lua.globals();

        globals
            .set("game", Game)
            .context("Failed to set game global in Lua state")?;

        create_require_global(&lua, &globals).context("Failed to create require global")?;
        create_task_global(&lua, &globals).context("Failed to create task global")?;

        // Explicitly drop globals so we can move ownership of `lua`
        drop(globals);

        Ok(Self {
            lua,
            compiler,
            sourcemap,
        })
    }

    /// Executes Luau code with a Roblox-like environment. `script_path` is read and is used for resolving module
    /// `require`s to other scripts on the file system using a Rojo sourcemap. Async because scripts could yield.
    pub async fn execute_script(&mut self, script_path: PathBuf) -> anyhow::Result<()> {
        let lua = &self.lua;

        // TODO: Resolve parent instance if one exists.
        let script_instance = InnerInstance::new(self.sourcemap.clone(), None);
        let script_instance = Rc::new(RefCell::new(script_instance));

        // Whilst all scripts share a read-only global environment (because of sandboxing), each script has its own
        // unique 'local' environment. For example, the `script` variable lives inside of that scripts environment,
        // whereas `game` and `require` are globals shared by all script
        let environment = lua.globals().clone(); // Create a copy of globals as a base environment

        environment
            .set("script", script_instance)
            .context("Failed to set script global in Lua chunk environment")?;

        environment.set_readonly(true);

        let script_name = script_path
            .file_prefix()
            .context("Script has no file prefix")?
            .to_string_lossy();

        let chunk = lua
            .load(&script_path)
            .set_name(script_name)
            .context("Failed to set Lua chunk name")?
            .set_environment(environment)
            .context("Failed to set Lua chunk environment")?
            .set_compiler(self.compiler.clone());

        chunk.exec_async().await?;

        Ok(())
    }
}
