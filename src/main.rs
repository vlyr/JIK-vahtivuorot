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

const WEEKDAYS: &[&'static str; 5] = &[
    "Maanantai",
    "Tiistai",
    "Keskiviikko",
    "Torstai",
    "Perjantai",
];

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

        parser::parse_teachers(res.split("\n"))
    }

    pub async fn get_teacher_schedule(&self, id: u32) -> Vec<Event> {
        let url = &format!("{}profiles/teachers/{}/schedule", &self.base_url, id);

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
            .map(|event| serde_json::from_value::<Event>(event).unwrap())
            .collect();

        events
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

    let mut events_vec = vec![];

    for id in client.get_teachers().await {
        let events = client.get_teacher_schedule(id).await;

        for event in events {
            events_vec.push(event);
            /*println!(
                "{}: {}, {}-{} ({}-{}) | {}",
                event.teacher(),
                event.text(),
                format_time(*event.start()),
                format_time(*event.end()),
                event.start(),
                event.end(),
                event.weekday()
            );*/
        }
    }

    let mut events_vec = events_vec
        .into_iter()
        .filter(|ev| ev.text().contains("Valvonta"))
        .collect::<Vec<Event>>();

    events_vec.sort_by_key(|ev| *ev.start());

    events_vec.sort_by(|a, b| {
        let mut weekdays = WEEKDAYS.into_iter();
        let mut weekdays_other = WEEKDAYS.into_iter();

        weekdays
            .position(|x| x == a.weekday())
            .unwrap()
            .cmp(&weekdays_other.position(|x| x == b.weekday()).unwrap())
    });

    events_vec.iter().for_each(|ev| {
        println!(
            "{} @ {}, {}-{} | {}",
            ev.teacher(),
            ev.text(),
            format_time(*ev.start()),
            format_time(*ev.end()),
            ev.weekday()
        )
    })
    //client.get_teacher_schedule(113).await;
}
