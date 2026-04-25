use reqwest::Client;
#[tokio::main]
async fn main() {
    let url = "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=100000&slippageBps=50";
    let client = Client::new();
    let res = client.get(url).header("User-Agent", "TipLink-Backend/1.0").send().await.unwrap();
    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await.unwrap());
}
