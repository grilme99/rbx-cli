use std::process;

use mlua::{String as LuaString, UserData};

#[derive(Debug, Clone, Copy)]
pub struct Game;

impl UserData for Game {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("GetService", |_, (_, service_name): (Game, LuaString)| {
            let service_name = service_name.to_str()?;

            match service_name {
                "ProcessService" => Ok(ProcessService),
                _ => Err(mlua::Error::RuntimeError(format!(
                    "'{service_name}' is not a valid Service name"
                ))),
            }
        });
    }
}

/// Recreation of an internal Roblox service only available in `roblox-cli`. Allows Lua code to exit the process with a
/// specific status code.
#[derive(Debug, Clone, Copy)]
pub struct ProcessService;

#[allow(unreachable_code)]
impl UserData for ProcessService {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("ExitAsync", |_, _, status_code: Option<i32>| {
            process::exit(status_code.unwrap_or(0));
            Ok(())
        })
    }
}
