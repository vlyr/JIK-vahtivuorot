use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::process;

const WEEKDAYS: &[&'static str; 5] = &[
    "Maanantai",
    "Tiistai",
    "Keskiviikko",
    "Torstai",
    "Perjantai",
];

const DATA_PATH: &str = "/.local/share/kauppalaskin.json";

const BREAK_PLACES: &[BreakPlace; 6] = &[
    BreakPlace::IikoonLinna,
    BreakPlace::Downstairs,
    BreakPlace::Upstairs,
    BreakPlace::FrontYard,
    BreakPlace::WingAndShed,
    BreakPlace::D,
];

use kauppalaskin::{
    event::{BreakPlace, Event, BREAK_STARTS},
    utils::*,
    wilma_client::{GetScheduleKind, WilmaClient},
};

fn read_data() -> Result<Vec<Vec<Vec<Event>>>, Box<dyn Error>> {
    let data_file_path = format!("{}{}", env::var("HOME").unwrap(), DATA_PATH);

    let data = fs::read_to_string(data_file_path)?;
    let events_vec: Vec<Event> = serde_json::from_str::<Vec<Value>>(&data)
        .expect("Failed converting file data to JSON")
        .iter()
        .map(|elem| serde_json::from_value(elem.clone()).unwrap())
        .collect();

    let mut breaks: Vec<Vec<Vec<Event>>> = vec![vec![vec![]]; 5];

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

        breaks[current_weekday_idx][current_start_idx].push(ev.clone());
    });

    Ok(breaks)
}

#[tokio::main]
async fn main() {
    let mut args = env::args();
    let binary_name = args.next(); // smart for error messages

    match args.next() {
        Some(arg) => match arg.as_ref() {
            "get" => {
                let day = args.next().expect("No input for day.");
                let start = start_str_to_number(&args.next().expect("No input for break start."));

                let weekday = match day.as_ref() {
                    "ma" => "Maanantai",
                    "ti" => "Tiistai",
                    "ke" => "Keskiviikko",
                    "to" => "Torstai",
                    "pe" => "Perjantai",

                    _ => {
                        println!("Invalid input for day, expected ma/ti/ke/to/pe");
                        process::exit(1);
                    }
                };

                let events = read_data().expect("Failed reading data from data file.");

                let weekday_idx = WEEKDAYS.iter().position(|x| x == &weekday).unwrap();

                let break_start_idx = BREAK_STARTS
                    .iter()
                    .position(|x| x == &start)
                    .expect("Invalid start time provided.");

                let selected_break = &events[weekday_idx][break_start_idx];

                for event in selected_break {
                    println!(
                        "{} ({})",
                        event.place().to_string(),
                        event.teachers().join(", ")
                    );
                }

                let missing: Vec<String> = BREAK_PLACES
                    .iter()
                    .filter(|place| {
                        selected_break
                            .iter()
                            .filter(|ev| ev.place() == **place)
                            .count()
                            == 0
                    })
                    .map(|place| place.to_string())
                    .collect();

                println!("\nPUUTTUU: {}", missing.join(", "));
            }

            "update" => {
                let username = env::var("USERNAME").unwrap();
                let password = env::var("PASSWORD").unwrap();
                let server = env::var("SERVER").unwrap();

                let client = WilmaClient::new(&username, &password, &server)
                    .await
                    .unwrap();

                let mut events_vec = vec![];

                println!("Updating...");

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

                let vec_json = serde_json::to_string(&events_vec).unwrap();

                let data_file_path = format!("{}{}", env::var("HOME").unwrap(), DATA_PATH);

                fs::write(data_file_path, vec_json).unwrap();

                println!("Done.");
            }
            _ => (),
        },

        None => {
            let events = read_data().expect("Failed reading data from data file.");

            for day in events {
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

                    println!("  PUUTTUU: {}\n", missing.join(", "));
                }
            }
        }
    }
}
