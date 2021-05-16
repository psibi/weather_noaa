use anyhow::Result;
use weathernoaa::weather::*;

#[tokio::main]
async fn main() -> Result<()> {
    let result = get_weather("VOBL".into()).await?;
    println!("{:?}", result);
    Ok(())
}
