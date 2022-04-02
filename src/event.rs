use serde::{Deserialize, Serialize};

// 525 = 8:45
// 585 = 9:45
// 645 = 1. rk alku (10:45)
// 660 = 1. rk puoliväli (11:00)
// 705 = 2. rk alku (11:45)
// 720 = 2. rk puoliväli (12:00)
const BREAK_STARTS: &[i32; 6] = &[525, 585, 645, 660, 705, 720];

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LongText {
    #[serde(rename(deserialize = "0"))]
    main: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Text {
    #[serde(rename(deserialize = "0"))]
    main: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Event {
    long_text: LongText,
    text: Text,
    start: u32,
    end: u32,
}

impl Event {
    pub fn long_text(&self) -> &String {
        &self.long_text.main
    }

    pub fn text(&self) -> &String {
        &self.text.main
    }

    pub fn start(&self) -> &u32 {
        &self.start
    }

    pub fn end(&self) -> &u32 {
        &self.end
    }
}
