use regex::Regex;
use serde::Deserialize;

use crate::{NUM_INI, NUM_FIN, NUM_MID};

#[derive(Deserialize)]
pub struct CombinatorConfig {
    pub ini_row_start: u32,
    pub mid_row_start: u32,
    pub fin_row_start: u32,
    pub ini_variants: Vec<Condition>,
    pub mid_variants: Vec<Condition>,
    pub fin_variants: Vec<Condition>,
}

pub enum Condition {
    Regex(Regex),
}

impl Condition {
    pub fn matches(&self, ini: u32, mid: u32, fin: u32) -> bool {
        if ini >= NUM_INI || mid >= NUM_MID || fin >= NUM_FIN {
            return false;
        }

        let ini_c = [
            'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
            ][ini as usize];
        let mid_c = [
            'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'
            ][mid as usize];
        let fin_c = [
            '0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
            ][fin as usize];

        let match_s = format!("{ini_c}{mid_c}{fin_c}");

        match self {
            Condition::Regex(exp) => exp.is_match(&match_s),
        }
    }
}

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrMap {
            String(String),
            Map(ConditionMap),
        }

        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum ConditionMap {
            Regex(String),
        }

        let result = match StringOrMap::deserialize(deserializer)? {
            StringOrMap::String(s) | StringOrMap::Map(ConditionMap::Regex(s)) =>
                Condition::Regex(Regex::new(&format!("^({s})$")).map_err(|e| serde::de::Error::custom(e))?),
        };

        Ok(result)
    }
}
