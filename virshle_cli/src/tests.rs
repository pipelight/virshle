use super::Cli;
use clap::Parser;
use miette::Result;
use tracing::debug;

async fn exec(cmd: &str) -> Result<()> {
    println!("\n");
    println!("-> {}\n", cmd);
    let os_str: Vec<&str> = cmd.split(' ').collect();
    let cli = Cli::parse_from(os_str);
    Cli::switch(cli).await?;
    Ok(())
}

#[tokio::test]
async fn parse_command_line() -> Result<()> {
    exec("vishle --help").await?;
    Ok(())
}

#[tokio::test]
async fn vm_ls() -> Result<()> {
    exec("vishle vm ls").await?;
    exec("vishle vm ls -v").await?;
    exec("vishle vm ls -vv").await?;
    Ok(())
}
#[tokio::test]
async fn vm_ssh_helpers() -> Result<()> {
    exec("vishle vm get-list-names").await?;
    Ok(())
}

#[tokio::test]
async fn template_ls() -> Result<()> {
    exec("vishle template ls").await?;
    exec("vishle template ls -v").await?;
    Ok(())
}

#[tokio::test]
async fn node_commands() -> Result<()> {
    exec("vishle node ls").await?;
    Ok(())
}
#[tokio::test]
async fn peer_commands() -> Result<()> {
    exec("vishle peer ping").await?;
    exec("vishle peer ls").await?;
    exec("vishle peer ls -vvvv").await?;
    exec("vishle peer ls --all").await?;
    exec("vishle peer ls --all -vvvv").await?;
    Ok(())
}
