pub fn format_time(time: u32) -> String {
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

pub fn start_str_to_number(time: &str) -> u32 {
    let (h, m) = time.split_once(":").unwrap();

    let hour_minutes = h.parse::<u32>().unwrap() * 60;
    let minutes = m.parse::<u32>().unwrap();
    hour_minutes + minutes
}
