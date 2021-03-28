use reqwest;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let res =
        reqwest::get("https://tgftp.nws.noaa.gov/data/observations/metar/decoded/VOBL.TXT").await?;

    let body = res.text().await?;

    println!("Body:\n\n{}", body);

    Ok(())
}
