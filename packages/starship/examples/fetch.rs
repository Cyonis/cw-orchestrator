use cw_orch_starship::StarshipClient;

#[tokio::main]
async fn main() {
    let starship = StarshipClient::new_async(None).await.unwrap();

    starship
        .create_channel("juno-1", "osmosis-1", "a", "b", "gg")
        .await
        .unwrap();
}
