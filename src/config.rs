use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub use_account_idx: usize,
    pub accounts: Vec<Account>,
}

#[derive(Deserialize)]
pub struct Account {
    pub phone: String,
    pub password: String,
    pub device_id: String,
    pub pin: String,
    pub otp: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        let cfg = std::fs::read_to_string("config.toml")
            .expect("Please create config.toml in working dir");
        let config: Config = toml::from_str(&cfg).unwrap();

        config
    }

    pub fn account(&self) -> &Account {
        self.accounts
            .get(self.use_account_idx)
            .expect("Invalid account index passed")
    }
}
