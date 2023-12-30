mod cli;

use anyhow::Result;
use cli::SubCommand;
use weathernoaa::weather::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = cli::init();

    match cmd.sub {
        SubCommand::Info { station_id } => {
            let result = get_weather(station_id).await?;
            println!("{:#?}", result);
        }
    }
    Ok(())
}
