use std::env;

use lib_rbxcli::{LuauCompiler, LuauContext};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root_path = env::current_dir()?.join("test-projects/require-modules");
    let sourcemap_path = root_path.join("sourcemap.json");
    let root_script = root_path.join("src/init.lua");

    let compiler = LuauCompiler::default();
    let mut context = LuauContext::new(&sourcemap_path, compiler)?;

    context.execute_script(root_script).await?;

    //     execute_script("game:GetService(\"ProcessService\")
    // print('boo ' .. tostring(script.Parent))", env::current_dir()?)?;

    Ok(())
}
