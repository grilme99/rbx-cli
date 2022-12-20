use anyhow::Context;
use mlua::{Lua, Table};

use crate::luau::userdata::InnerInstance;

pub fn create_require_global(lua: &Lua, globals: &Table) -> anyhow::Result<()> {
    let require_fn = lua
        .create_function(|_, instance: InnerInstance| {
            println!("Requiring {}", instance.name);
            Ok(())
        })
        .context("Failed to create Lau require function")?;

    globals
        .set("require", require_fn)
        .context("Failed to set require global in Lua state")?;

    Ok(())
}
