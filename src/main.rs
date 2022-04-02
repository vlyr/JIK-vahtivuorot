use reqwest::{
    cookie::{Cookie, Jar},
    redirect::Policy,
    Client, Url,
};

use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;

mod event;
use event::Event;

mod parser;

struct WilmaClient {
    base_url: String,
    client: Client,
}

impl WilmaClient {
    async fn new(username: &str, password: &str, server: &str) -> Result<Self, Box<dyn Error>> {
        let builder = reqwest::Client::builder().redirect(Policy::none());
        let client = builder.build()?;

        let mut url = format!("https://{}/", server);

        /*let _wilmas = reqwest::get("https://www.starsoft.fi/wilmat/wilmat.json")
        .await?
        .text()
        .await?;*/

        let res = reqwest::get(url.clone() + "index_json")
            .await?
            .text()
            .await?;

        let res_json: Value = serde_json::from_str(&res)?;

        let session_id = res_json.get("SessionID").unwrap().as_str().unwrap();

        let mut info: HashMap<&str, &str> = HashMap::new();
        info.insert("Login", username);
        info.insert("Password", password);
        info.insert("SESSIONID", session_id);
        info.insert("CompleteJson", "");

        let res = client
            .post(url.clone() + "login")
            .form(&info)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;

        let cookies: Vec<Cookie> = res.cookies().collect();
        let cookie = cookies.iter().find(|c| c.name() == "Wilma2SID").unwrap();

        let jar = Arc::new(Jar::default());
        let cookie_url = url.clone().parse::<Url>()?;
        jar.add_cookie_str(&format!("Wilma2SID={}", cookie.value()), &cookie_url);

        let builder = reqwest::Client::builder().redirect(Policy::none());
        let client = builder.cookie_provider(jar).build()?;

        let res = client.get(url.clone()).send().await?.text().await?;

        let mut lines = res.split("\n");

        let identity = parser::parse_identity(&mut lines);

        url += &identity;

        Ok(Self {
            client,
            base_url: url.to_string(),
        })
    }

    pub async fn get_teacher_schedule(&self, id: u32) {
        let url = &format!("{}profiles/teachers/{}/schedule", self.base_url.clone(), id);
        println!("{}", url);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let events = parser::teacher_schedule(res.split("\n"))
            .into_iter()
            .map(|event| serde_json::from_value::<Event>(event).unwrap());

        for event in events {
            let full_h_start = (*event.start() as f32 / 60.0).floor();
            let full_h_end = (*event.end() as f32 / 60.0).floor();

            println!(
                "{}, {}-{} ({}-{})",
                event.text(),
                format_time(*event.start()),
                format_time(*event.end()),
                event.start(),
                event.end()
            );
        }
    }
}

fn format_time(time: u32) -> String {
    let time_h = (time as f32 / 60.0).floor();
    let time_hm = time as f32 / 60.0;

    let minutes = (time_hm - time_h) * 60.0;

    let minutes_fmt = if minutes < 10.0 {
        format!("0{}", minutes.round())
    } else {
        format!("{}", minutes.round())
    };

    format!("{}:{}", time_h, minutes_fmt)
}

#[tokio::main]
async fn main() {
    let username = env::var("USERNAME").unwrap();
    let password = env::var("PASSWORD").unwrap();
    let server = env::var("SERVER").unwrap();

    let client = WilmaClient::new(&username, &password, &server)
        .await
        .unwrap();

    client.get_teacher_schedule(113).await;
}
