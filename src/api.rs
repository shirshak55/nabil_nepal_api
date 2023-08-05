use crate::config;
use crate::IResult;
use chrono::prelude::*;
use std::ops::Sub;

const DEVICE_DETAIL: &'static str = r#"{"androidVersion":"11","appVersionCode":"136","appVersionName":"5.5.66","developerOptionsEnabled":false,"deviceRooted":false,"dontKeepActivitiesEnabled":false,"manufacturer":"Xiaomi","model":"Samsung","sdkVersion":30}","deviceId":"123123123123"#;

pub struct LoginInfo<'a> {
    pub token: &'a str,
    pub account_number: &'a str,
}

pub fn login_info_file(config: &config::Config) -> String {
    format!("cache/s_{}.json", config.account().phone)
}

pub async fn request_otp(client: reqwest::Client, config: &config::Config) -> IResult<()> {
    let otp_endpont = "https://banksmart.nabilbank.com/banksmartv5/request/bookResetDeviceId";

    println!("Requesting OTP");

    let resp = client
        .post(otp_endpont)
        .json(&serde_json::json!({
            "appRegistrationId": "appRegistrationId",
            "deviceDetail": "{}",
            "deviceId": config.account().device_id,
            "osType": "Android",
            "password": config.account().password,
            "txnPassword": config.account().pin,
            "username": config.account().phone
        }))
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    println!("Response: {:#?}", json);
    Ok(())
}

pub async fn allow_device_to_login(
    client: reqwest::Client,
    config: &config::Config,
) -> IResult<()> {
    let otp_verify_end_point = "https://banksmart.nabilbank.com/banksmartv5/request/resetDeviceId";
    println!(
        "Ability to login from this device id: {:?}",
        config.account().device_id
    );

    if config.account().otp.is_none() {
        return Err("Please enter otp in config.toml file".into());
    }
    let resp = client
        .post(otp_verify_end_point)
        .json(&serde_json::json!({
            "appRegistrationId": "appRegistrationId",
            "deviceDetail": "{}",
            "deviceId": config.account().device_id,
            "osType": "Android",
            "otpCode": config.account().otp,
            "password": config.account().password,
            "txnPassword": config.account().pin,
            "username": config.account().phone
        }))
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    println!("Response: {:#?}", json);
    Ok(())
}

pub async fn login(client: reqwest::Client, config: &config::Config) -> IResult<()> {
    let login_endpoint = "https://banksmart.nabilbank.com/banksmartv5/customer/customerInfo";
    println!(
        "Logging in from this device id: {:?}",
        config.account().device_id
    );

    let resp = client
        .post(login_endpoint)
        .form(&[
            ("password", &config.account().password[..]),
            ("versionId", "2.0.0"),
            ("deviceDetail", "{}"),
            ("osType", "Android"),
            ("userName", &config.account().phone),
            ("deviceId", &config.account().device_id),
        ])
        .send()
        .await?;

    let text = resp.text().await?;
    std::fs::write(login_info_file(&config), &text).expect("Unable to write response to file");

    let customer_name = serde_json::from_str::<serde_json::Value>(&text)?;
    println!("Response: {:#?}", customer_name["customerName"].as_str());

    Ok(())
}

pub async fn get_full_statements<'a>(
    client: reqwest::Client,
    config: &config::Config,
    li: &LoginInfo<'_>,
) -> IResult<()> {
    let full_statement_endpoint = "https://banksmart.nabilbank.com/middleware/api/v1/fullStatement";
    println!("Get Full Statements: {:?}", config.account().device_id);

    let now: DateTime<Local> = Local::now();
    let mut days_to_add: i64 = 0;

    const DAYS_TO_OFFSET: i64 = 29;
    let mut all = vec![];

    for _ in 0..10 {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        for _ in 0..3 {
            days_to_add += DAYS_TO_OFFSET;
            let to = now
                .sub(chrono::Duration::days(days_to_add - DAYS_TO_OFFSET))
                .format("%Y-%m-%d")
                .to_string();
            let from = now
                .sub(chrono::Duration::days(days_to_add))
                .format("%Y-%m-%d")
                .to_string();

            println!("FROM {} TO {}", from, to);

            let current = now.format("%d-%m-%Y %H:%M:%S").to_string();
            let json_input = serde_json::json!({
                "fromDate": from,
                "toDate": to,
                "accountNumber": li.account_number,
                "date": current,
                "token": li.token,
                "deviceDetail": DEVICE_DETAIL,
                "deviceId": config.account().device_id
            });

            tokio::spawn(parallel_balance_fetch_loop(
                client.clone(),
                &full_statement_endpoint,
                json_input,
                tx.clone(),
                days_to_add,
            ));
        }
        drop(tx);
        loop {
            let message = rx.recv().await;
            if message.is_none() {
                break;
            }

            all.push(message.unwrap());
        }
    }
    all.sort_by(|a, b| a.0.cmp(&b.0));

    use simple_excel_writer::*;
    let export_file = format!("cache/statements_{}.xlsx", config.account().phone);
    let mut wb = Workbook::create(&export_file);
    let mut sheet = wb.create_sheet("SheetName");
    sheet.add_column(Column { width: 10.0 });
    sheet.add_column(Column { width: 10.0 });
    sheet.add_column(Column { width: 10.0 });
    sheet.add_column(Column { width: 10.0 });
    sheet.add_column(Column { width: 10.0 });

    wb.write_sheet(&mut sheet, move |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row!["date", "amount", "balance", "particular", "txn_type"])?;

        for statement in all {
            let statement = statement.1.as_object().unwrap();

            sw.append_row(row![
                statement["date"].as_str().unwrap(),
                statement["amount"].as_str().unwrap(),
                statement["balance"].as_str().unwrap(),
                statement["particular"].as_str().unwrap(),
                statement["txnType"].as_str().unwrap()
            ])?;
        }

        Ok(())
    })
    .expect("write excel error!");

    wb.close().expect("close excel error!");

    Ok(())
}

async fn parallel_balance_fetch_loop(
    client: reqwest::Client,
    full_statement_endpoint: &str,
    json_input: serde_json::Value,
    tx: tokio::sync::mpsc::UnboundedSender<(i64, serde_json::Value)>,
    days_to_add: i64,
) -> IResult<()> {
    let resp = client
        .post(full_statement_endpoint)
        .json(&json_input)
        .send()
        .await?;

    let text = &resp.text().await?;

    let json: serde_json::Value = serde_json::from_str(&text)?;

    let statements = json["statements"].as_array();

    if let Some(statements) = statements {
        for statement in statements {
            let res = tx.send((days_to_add, statement.clone()));

            if res.is_err() {
                println!("error on sending data over channel")
            }
        }
    }

    Ok(())
}

pub async fn send_esewa_payment<'a>(
    client: reqwest::Client,
    config: &config::Config,
    li: &LoginInfo<'_>,
) -> IResult<()> {
    let load_esewa_endpoint = "https://banksmart.nabilbank.com/banksmartv5/payment/merchantPayment";
    println!("Load Esewa: {:?}", config.account().device_id);
    let now: DateTime<Local> = Local::now();
    let current_date = now.format("%d-%m-%Y %H:%M:%S").to_string();
    // Todo(shirshak55): harded coded need to use config later.
    let resp = client
        .post(load_esewa_endpoint)
        .json(&serde_json::json!({
            "merchantCode": "ESEWA",
            "featureCode": "WALLET",
            "fields": [
                {
                "paramOrder": "0",
                "label": "Amount",
                "paramValue": "5000"
                },
                {
                "paramOrder": "1",
                "label": "eSewa ID",
                "paramValue": "asdfasdf@gmail.com"
                },
                {
                "paramOrder": "2",
                "label": "Purpose",
                "paramValue": "Utility Payment"
                }
            ],
            "accountNumber": li.account_number,
            "txnPassword": "1234",
            "token": li.token,
            "date": current_date,
            "deviceDetail": DEVICE_DETAIL,
            "deviceId":  config.account().device_id
        }))
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    println!("Response: {:#?}", json);
    Ok(())
}

pub async fn send_bank_payment<'a>(
    client: reqwest::Client,
    config: &config::Config,
    li: &LoginInfo<'_>,
) -> IResult<()> {
    let load_esewa_endpoint =
        "https://banksmart.nabilbank.com/banksmartv5/payment/internalTransfer";
    println!("Load Bank: {:?}", config.account().device_id);
    let now: DateTime<Local> = Local::now();
    let current_date = now.format("%d-%m-%Y %H:%M:%S").to_string();
    let resp = client
        .post(load_esewa_endpoint)
        .json(&serde_json::json!({
                "bankCode": "NARBNPKA",
                "amount": "1000",
                "accountNumber": li.account_number,
                "txnPassword": "7755",
                "remarks": "reason",
                "date": current_date,
                "token": li.token,
                "deviceDetail": DEVICE_DETAIL,
                "deviceId":  config.account().device_id
        }))
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    println!("Response: {:#?}", json);
    Ok(())
}
