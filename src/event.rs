use serde::{Deserialize, Serialize};

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
