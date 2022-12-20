#![feature(path_file_prefix)]

mod luau;

pub use luau::LuauContext;

// Re-export mlua's compiler for use with LuauContext
pub use mlua::Compiler as LuauCompiler;
