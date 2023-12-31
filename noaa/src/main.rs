mod cli;

use anyhow::Result;
use cli::SubCommand;
use weathernoaa::weather::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = cli::init();
    let app = NoaaApp::new();
    match cmd.sub {
        SubCommand::Info { station_id } => {
            let result = app.get_weather(&station_id).await?;
            println!("{:#?}", result);
        }
    }
    Ok(())
}
