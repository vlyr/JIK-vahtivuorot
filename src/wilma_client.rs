use reqwest::{
    cookie::{Cookie, Jar},
    redirect::Policy,
    Client, Url,
};

use crate::{event::Event, parser};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct WilmaClient {
    base_url: String,
    client: Client,
}

pub enum GetScheduleKind {
    Personnel,
    Teacher,
}

impl WilmaClient {
    pub async fn new(username: &str, password: &str, server: &str) -> Result<Self, Box<dyn Error>> {
        let builder = reqwest::Client::builder().redirect(Policy::none());
        let client = builder.build()?;

        let mut url = format!("https://{}/", server);

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

        let identity = parser::parse_identity(&res);

        url += &identity;

        Ok(Self {
            client,
            base_url: url.to_string(),
        })
    }

    pub async fn get_teachers(&self) -> Vec<u32> {
        let url = &format!("{}profiles/teachers", &self.base_url);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        parser::parse_teachers(&res)
    }

    pub async fn get_personnel(&self) -> Vec<u32> {
        let url = &format!("{}profiles/personnel", &self.base_url);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        parser::parse_teachers(&res)
    }

    pub async fn get_schedule(&self, id: u32, kind: GetScheduleKind) -> Vec<Event> {
        let path = match kind {
            GetScheduleKind::Personnel => "personnel",
            GetScheduleKind::Teacher => "teachers",
        };

        let url = &format!("{}profiles/{}/{}/schedule", &self.base_url, path, id);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        parser::schedule(&res)
            .into_iter()
            .map(|event| serde_json::from_value::<Event>(event).unwrap())
            .collect()
    }
}
