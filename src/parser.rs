use scraper::{Html, Selector};

const SCHEDULE_LINE: &str = "<script data-cfasync=\"false\" src=\"/cdn-cgi/scripts/5c5dd728/cloudflare-static/email-decode.min.js\"></script><script type=\"text/javascript\">";

pub fn parse_identity(document: &str) -> String {
    let mut lines = document.split("\n");
    let line = utils::filter_line("text-style-link", &mut lines).unwrap();

    let fragment = Html::parse_fragment(line);
    let selector = Selector::parse("a").unwrap();
    let stuff = fragment.select(&selector).next().unwrap();

    let mut identity = stuff.value().attr("href").unwrap().to_string();
    identity.remove(0);
    identity
}

pub fn schedule(document: &str) -> Vec<serde_json::Value> {
    let lines = document.split("\n");
    let pos = utils::line_pos(SCHEDULE_LINE, &mut lines.clone()).unwrap();
    let lines_vec: Vec<&str> = lines.collect();
    let json_line = lines_vec.get(pos + 1).unwrap().to_string();

    let json_str = utils::remove_invalid_json(&json_line);
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    json["Events"].as_array().unwrap().to_vec()
}

pub fn parse_teachers(document: &str) -> Vec<u32> {
    let mut lines = document.split("\n");

    utils::filter_lines("class=\"profile-link ", &mut lines)
        .into_iter()
        .map(|line| line.replace(">", "/>").to_owned())
        .map(|line| {
            let frag = Html::parse_fragment(&line);
            let selection = frag.select(&Selector::parse("a").unwrap()).next().unwrap();
            selection.value().attr("href").unwrap().to_string()
        })
        .map(|line| line.split('/').collect::<Vec<&str>>()[4].to_string())
        .map(|id| id.trim().parse::<u32>().unwrap())
        .collect()
}

mod utils {
    pub fn filter_line<'a, I>(pattern: &str, lines: &mut I) -> Option<&'a str>
    where
        I: Iterator<Item = &'a str>,
    {
        lines.find(|l| l.contains(pattern))
    }

    pub fn filter_lines<'a, I>(pattern: &str, lines: &mut I) -> Vec<&'a str>
    where
        I: Iterator<Item = &'a str>,
    {
        lines.filter(|l| l.contains(pattern)).collect()
    }

    pub fn line_pos<'a, I>(pattern: &str, lines: &mut I) -> Option<usize>
    where
        I: Iterator<Item = &'a str>,
    {
        lines.position(|l| l.contains(pattern))
    }

    pub fn remove_invalid_json(json_line: &str) -> String {
        let s = ("{".to_owned() + &json_line[81..])
            .to_owned()
            .replace("Events", "\"Events\"");

        s.replace(
            ", ActiveTyyppi: \"\", ActiveId: \"\", DialogEnabled: 0};",
            "}",
        )
    }
}
