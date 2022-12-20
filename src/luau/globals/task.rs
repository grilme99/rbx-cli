use std::time::{Duration, Instant};

use anyhow::Context;
use mlua::{Lua, Table};
use tokio::time::sleep;

/// Waits for the specified time and returns how long it actually took.
async fn wait(_lua: &Lua, seconds: f64) -> mlua::Result<f64> {
    let start = Instant::now();
    sleep(Duration::from_secs_f64(seconds)).await;

    let duration = start.elapsed().as_secs_f64();
    Ok(duration)
}

pub fn create_task_global(lua: &Lua, globals: &Table) -> anyhow::Result<()> {
    let task = lua.create_table().context("Failed to create task table")?;

    let wait_fn = lua
        .create_async_function(wait)
        .context("Failed to create Lua sleep function")?;

    task.set("wait", wait_fn.clone())
        .context("Failed to set wait function in task library")?;

    globals
        .set("task", task)
        .context("Failed to set task global in Lua state")?;

    // Roblox has a global `wait` function and a `task.wait` function. In Roblox they have slightly different
    // implementations, but we'll just use the same here.
    globals
        .set("wait", wait_fn)
        .context("Failed to set wait global in Lua state")?;

    Ok(())
}
