use serde::{Deserialize, Serialize};

// 525 = 8:45
// 585 = 9:45
// 645 = 1. rk alku (10:45)
// 660 = 1. rk puoliväli (11:00)
// 705 = 2. rk alku (11:45)
// 720 = 2. rk puoliväli (12:00)
pub const BREAK_STARTS: &[u32; 6] = &[525, 585, 645, 660, 705, 720];

#[derive(PartialEq, Clone, Debug)]
pub enum BreakPlace {
    IikoonLinna,
    Downstairs,
    Upstairs,
    FrontYard,
    WingAndShed, // katos?
    D,           // takapiha?
    Wing,        // ??
}

impl<T> From<T> for BreakPlace
where
    T: AsRef<str>,
{
    fn from(data: T) -> Self {
        use BreakPlace::*;
        match data.as_ref() {
            "Valvonta YK" => Upstairs,
            "Valvonta AK" => Downstairs,
            "Valvonta E + S" => WingAndShed,
            "Valvonta S" => Wing,
            "Valvonta P" => FrontYard,
            "Valvonta D" => D,
            "Valvonta Iikoon linna" => IikoonLinna,
            _ => unreachable!(),
        }
    }
}

impl ToString for BreakPlace {
    fn to_string(&self) -> String {
        use BreakPlace::*;

        match self {
            IikoonLinna => "Iikoon linna",
            Downstairs => "Alakerta",
            Upstairs => "Yläkerta",
            FrontYard => "Etupiha/Parkkipaikka",
            WingAndShed => "Ruokalarakennuksen katos ja siipirakennus",
            D => "Takapiha? (D)",
            Wing => "Siipirakennus",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LongText {
    #[serde(alias = "0")]
    main: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpeInfo {
    #[serde(alias = "0")]
    inner: OpeInfoInner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpeInfoInner {
    #[serde(alias = "0")]
    inner: Option<OpeInfoInnerInner>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct OpeInfoInnerInner {
    #[serde(alias = "nimi")]
    name: String,
    #[serde(alias = "lyhenne")]
    abbreviation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Text {
    #[serde(alias = "0")]
    main: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    #[serde(alias = "LongText")]
    long_text: LongText,
    #[serde(alias = "Text")]
    text: Text,
    #[serde(alias = "Start")]
    start: u32,
    #[serde(alias = "End")]
    end: u32,
    #[serde(alias = "OpeInfo")]
    ope_info: OpeInfo,
    #[serde(alias = "HenkiloInfo")]
    henkilo_info: OpeInfo,
    #[serde(alias = "ViikonPaiva")]
    weekday: String,
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

    pub fn teachers(&self) -> Vec<String> {
        let ope_name = self
            .ope_info
            .inner
            .inner
            .as_ref()
            .unwrap_or(&OpeInfoInnerInner::default())
            .name
            .clone();

        let hlo_name = self
            .henkilo_info
            .inner
            .inner
            .as_ref()
            .unwrap_or(&OpeInfoInnerInner::default())
            .name
            .clone();

        vec![ope_name, hlo_name]
            .iter()
            .filter(|x| !x.is_empty())
            .map(|x| x.clone())
            .collect()
    }

    pub fn weekday(&self) -> &String {
        &self.weekday
    }

    pub fn place(&self) -> BreakPlace {
        BreakPlace::from(self.text())
    }
}
