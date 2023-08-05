pub mod api;
pub mod config;

pub type IResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> IResult<()> {
    let config = config::Config::new();

    let proxy = reqwest::Proxy::all("http://127.0.0.1:9090").unwrap();

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .danger_accept_invalid_certs(true)
        .build()?;

    // if true {
    //     let send_otp_res = api::request_otp(client.clone(), &config).await;
    //     if send_otp_res.is_err() {
    //         println!("Error on sending otp {:?}", send_otp_res.unwrap_err());
    //     }
    //     return Ok(());
    // }

    // let send_otp_res = api::allow_device_to_login(client.clone(), &config).await;
    // if send_otp_res.is_err() {
    //     println!("Error on sending otp {:?}", send_otp_res.unwrap_err());
    // }

    let login_res = api::login(client.clone(), &config).await;

    if login_res.is_err() {
        println!("Error on Login {:?}", login_res.unwrap_err());
        return Ok(());
    }

    let cache_login_info = std::fs::read_to_string(api::login_info_file(&config));

    if cache_login_info.is_ok() {
        let info: serde_json::Value = serde_json::from_str(&cache_login_info.unwrap()).unwrap();
        let token = info["token"].as_str().unwrap();
        let account_number = info["bankAccounts"][0]["accountNumber"].as_str().unwrap();
        let login_info = api::LoginInfo {
            token,
            account_number,
        };

        let login_res = api::get_full_statements(client, &config, &login_info).await;
        if login_res.is_err() {
            println!("Error on sending otp {:?}", login_res.unwrap_err());
        }

        // let login_res = api::send_esewa_payment(client.clone(), &config, &login_info).await;
        // if login_res.is_err() {
        //     println!("Error on sending otp {:?}", login_res.unwrap_err());
        // }

        // let login_res = api::send_bank_payment(client.clone(), &config, &login_info).await;
        // if login_res.is_err() {
        //     println!("Error on sending otp {:?}", login_res.unwrap_err());
        // }
    }
    Ok(())
}
