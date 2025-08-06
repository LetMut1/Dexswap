use {
    solana_sdk::signer::keypair::Keypair,
    std::{
        error::Error,
        path::Path,
    },
};
pub struct Loader;
impl Loader {
    pub fn load_keypair_from_file(keypair_file_path: &str) -> Result<Keypair, Box<dyn Error + 'static>> {
        let keypair_file_path_ = Path::new(keypair_file_path);
        let keypair_file_data = if keypair_file_path_.try_exists()? {
            std::fs::read_to_string(keypair_file_path_)?
        } else {
            return Err("The keypair.json file does not exist.".into());
        };
        Ok(Keypair::from_bytes(serde_json::from_str::<Vec<u8>>(keypair_file_data.as_str())?.as_slice())?)
    }
}
