use weathernoaa::weather::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let result = get_weather("VOBL".into()).await?;
    println!("{:?}",result);
    Ok(())
}
