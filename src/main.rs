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

        parser::teacher_schedule(res.split("\n"));
    }
}

#[tokio::main]
async fn main() {
    let username = env::var("USERNAME").unwrap();
    let password = env::var("PASSWORD").unwrap();
    let server = env::var("SERVER").unwrap();

    let client = WilmaClient::new(&username, &password, &server)
        .await
        .unwrap();

    client.get_teacher_schedule(89).await;
}
