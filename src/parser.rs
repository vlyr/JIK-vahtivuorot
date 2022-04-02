use scraper::{Html, Selector};

pub fn parse_identity<'a, I>(document: &mut I) -> String
where
    I: Iterator<Item = &'a str>,
{
    let line = filter_line("text-style-link", document).unwrap();

    let fragment = Html::parse_fragment(line);
    let selector = Selector::parse("a").unwrap();
    let stuff = fragment.select(&selector).next().unwrap();

    let mut identity = stuff.value().attr("href").unwrap().to_string();
    identity.remove(0);
    identity
}

pub fn filter_line<'a, I>(pattern: &str, lines: &mut I) -> Option<&'a str>
where
    I: Iterator<Item = &'a str>,
{
    lines.find(|l| l.contains(pattern))
}

pub fn line_pos<'a, I>(pattern: &str, lines: &mut I) -> Option<usize>
where
    I: Iterator<Item = &'a str>,
{
    lines.position(|l| l.contains(pattern))
}

pub fn teacher_schedule<'a, I>(lines: I)
where
    I: Iterator<Item = &'a str> + Clone,
{
    let pos = line_pos("<script data-cfasync=\"false\" src=\"/cdn-cgi/scripts/5c5dd728/cloudflare-static/email-decode.min.js\"></script><script type=\"text/javascript\">", &mut lines.clone()).unwrap();
    let lines_vec: Vec<&str> = lines.collect();
    let json_line = lines_vec.get(pos + 1).unwrap().to_string();

    // Remove invalid JSON.
    let s = ("{".to_owned() + &json_line[81..])
        .to_owned()
        .replace("Events", "\"Events\"");

    let s_finished = s.replace(
        ", ActiveTyyppi: \"\", ActiveId: \"\", DialogEnabled: 0};",
        "}",
    );

    println!("{}", s_finished);

    let json: serde_json::Value = serde_json::from_str(&s_finished).unwrap();
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
