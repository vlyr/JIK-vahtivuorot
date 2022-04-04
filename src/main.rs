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

const BREAK_PLACES: &[BreakPlace; 6] = &[
    BreakPlace::IikoonLinna,
    BreakPlace::Downstairs,
    BreakPlace::Upstairs,
    BreakPlace::FrontYard,
    BreakPlace::EPlusS,
    BreakPlace::D,
];

pub enum GetScheduleKind {
    Personnel,
    Teacher,
}

mod event;
use event::{BreakPlace, Event, BREAK_STARTS};

mod parser;

#[derive(Debug, Clone)]
struct WilmaClient {
    base_url: String,
    client: Client,
}

impl WilmaClient {
    async fn new(username: &str, password: &str, server: &str) -> Result<Self, Box<dyn Error>> {
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
            .map(|event| {
                println!("{}", event);
                serde_json::from_value::<Event>(event).unwrap()
            })
            .collect()
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
        let events = client.get_schedule(id, GetScheduleKind::Teacher).await;

        for event in events {
            events_vec.push(event);
        }
    }

    for id in client.get_personnel().await {
        let events = client.get_schedule(id, GetScheduleKind::Personnel).await;

        for event in events {
            events_vec.push(event);
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

    // gotta redo this lmao
    let mut breaks: Vec<Vec<Vec<&Event>>> = vec![vec![vec![]]; 5];

    let mut current_weekday_idx = 0;
    let mut current_start_idx: usize = 0;

    events_vec.iter().for_each(|ev| {
        let weekday_idx = WEEKDAYS
            .into_iter()
            .position(|x| x == ev.weekday())
            .unwrap();

        let break_start_idx = BREAK_STARTS
            .into_iter()
            .position(|x| *x as usize == *ev.start() as usize)
            .unwrap();

        if weekday_idx != current_weekday_idx {
            current_weekday_idx += 1;
            current_start_idx = 0;
        } else if break_start_idx != current_start_idx {
            breaks[current_weekday_idx].push(vec![]);
            current_start_idx += 1;
        }

        breaks[current_weekday_idx][current_start_idx].push(ev);
    });

    for day in breaks {
        for b in day {
            println!(
                "{} | {}-{}",
                b[0].weekday(),
                format_time(*b[0].start()),
                format_time(*b[0].end())
            );

            for monitor in &b {
                println!(
                    "  {} ({})",
                    monitor.place().to_string(),
                    monitor.teachers().join(", ")
                );
            }

            let missing: Vec<String> = BREAK_PLACES
                .iter()
                .filter(|place| b.iter().filter(|ev| ev.place() == **place).count() == 0)
                .map(|place| place.to_string())
                .collect();

            println!("puuttuu: {}\n", missing.join(", "));
        }
    }
}
