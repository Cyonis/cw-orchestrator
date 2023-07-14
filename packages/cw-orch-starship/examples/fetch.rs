use cw_orch_starship::starship::Starship;
use tokio::runtime::Handle;

const CRATE_PATH: &str = env!("CARGO_MANIFEST_DIR");
const JUNO_MNEMONIC: &str = "dilemma imitate split detect useful creek cart sort grow essence fish husband seven hollow envelope wedding host dry permit game april present panic move";
const OSMOSIS_MNEMONIC: &str = "settle gas lobster judge silk stem act shoulder pluck waste pistol word comfort require early mouse provide marine butter crowd clock tube move wool";
const JUNO: &str = "juno-1";
const OSMOSIS: &str = "osmosis-2";

#[tokio::main]
async fn main() {
    let starship = Starship::new_async().await.unwrap();

    starship
        .create_channel("juno-1", "osmosis-1", "a", "b", "gg")
        .await
        .unwrap();
}
