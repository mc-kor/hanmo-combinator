use std::{collections::HashMap, fs, path::PathBuf, array};

use image::{io::Reader as ImageReader, DynamicImage};

use itertools::{iproduct, Itertools};
use regex::Regex;
use serde::Deserialize;

use crate::hangul::{NUM_INI, NUM_FIN, NUM_MID, INI_CHARS, MID_CHARS, FIN_CHARS};

pub struct Workspace {
    pub path: PathBuf,

    pub global_config: GlobalConfig,

    pub ini_configs: [GlyphConfig; NUM_INI as usize],
    pub mid_configs: [GlyphConfig; NUM_MID as usize],
    pub fin_configs: [GlyphConfig; NUM_FIN as usize],

    pub ini_glyphs: [Option<DynamicImage>; NUM_INI as usize],
    pub mid_glyphs: [Option<DynamicImage>; NUM_MID as usize],
    pub fin_glyphs: [Option<DynamicImage>; NUM_FIN as usize],
}

#[derive(Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_size")]
    pub size: u32,
}

fn default_size() -> u32 { 16 }

pub struct GlyphConfig {
    pub conditions: Vec<ConditionEntry>,
}

impl Default for GlyphConfig {
    fn default() -> Self {
        Self { conditions: vec![ConditionEntry {
            condition: Condition::Always,
            priority: 0,
            variant: None,
        }] }
    }
}

pub struct ConditionEntry {
    pub condition: Condition,
    pub priority: u32,
    pub variant: Option<u32>,
}

pub enum Condition {
    Regex(Regex),
    Always,
}

impl Condition {
    pub fn matches(&self, ini: u32, mid: u32, fin: u32) -> bool {
        let Some(match_s) = to_match_str(ini, mid, fin) else { return false; };

        match self {
            Condition::Regex(exp) => exp.is_match(&match_s),
            Condition::Always => true
        }
    }
}

pub fn to_match_str(ini: u32, mid: u32, fin: u32) -> Option<String> {
    if ini >= NUM_INI || mid >= NUM_MID || fin >= NUM_FIN {
        return None;
    }

    let ini_c = INI_CHARS[ini as usize];
    let mid_c = MID_CHARS[mid as usize];
    let fin_c = FIN_CHARS[fin as usize];

    Some(format!("{ini_c}{mid_c}{fin_c}"))
}

pub fn matches_regex(regex: &Regex, ini: u32, mid: u32, fin: u32) -> bool {
    let Some(match_s) = to_match_str(ini, mid, fin) else { return false; };

    regex.is_match(&match_s)
}

impl<'de> Deserialize<'de> for GlyphConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct GlyphConfig_ {
            regex: HashMap<String, u32>,
        }
        let map = GlyphConfig_::deserialize(deserializer)?;
        let regexes = map.regex.into_iter()
            .map(|(s, variant)| {
                let regex = Regex::new(&s)
                    .map_err(|e| serde::de::Error::custom(e))?;
                let priority = iproduct!(0..NUM_INI, 0..NUM_MID, 0..NUM_FIN)
                    .filter(|(ini, mid, fin)| matches_regex(&regex, *ini, *mid, *fin))
                    .count() as u32;
                Ok(ConditionEntry {
                    condition: Condition::Regex(regex),
                    priority,
                    variant: Some(variant),
                })
            }).try_collect()?;
        return Ok(GlyphConfig {
            conditions: regexes
        })
    }
}

impl Workspace {
    pub fn load(path: PathBuf) -> eyre::Result<Workspace> {
        let global_config = toml::from_str::<GlobalConfig>(&fs::read_to_string(path.join("config.toml"))?)?;

        let ini_configs = array::from_fn(|ini| {
            let path = path.join(format!("src/ini/{}/config.toml", INI_CHARS[ini as usize]));
            let Ok(glyph_config_str) = fs::read_to_string(path) else { return GlyphConfig::default() };
            let Ok(glyph_config) = toml::from_str::<GlyphConfig>(&glyph_config_str) else { return GlyphConfig::default() };
            glyph_config
        });
        let mid_configs = array::from_fn(|mid| {
            let path = path.join(format!("src/mid/{}/config.toml", MID_CHARS[mid as usize]));
            let Ok(glyph_config_str) = fs::read_to_string(path) else { return GlyphConfig::default() };
            let Ok(glyph_config) = toml::from_str::<GlyphConfig>(&glyph_config_str) else { return GlyphConfig::default() };
            glyph_config
        });
        let fin_configs = array::from_fn(|fin| {
            let path = path.join(format!("src/fin/{}/config.toml", FIN_CHARS[fin as usize]));
            let Ok(glyph_config_str) = fs::read_to_string(path) else { return GlyphConfig::default() };
            let Ok(glyph_config) = toml::from_str::<GlyphConfig>(&glyph_config_str) else { return GlyphConfig::default() };
            glyph_config
        });

        let ini_glyphs = array::from_fn(|ini| {
            let path = path.join(format!("src/ini/{}/glyphs.bmp", INI_CHARS[ini as usize]));
            let Ok(reader) = ImageReader::open(path) else { return None; };
            let Ok(image) = reader.decode() else { return None; };
            Some(image)
        });
        let mid_glyphs = array::from_fn(|mid| {
            let path = path.join(format!("src/mid/{}/glyphs.bmp", MID_CHARS[mid as usize]));
            let Ok(reader) = ImageReader::open(path) else { return None; };
            let Ok(image) = reader.decode() else { return None; };
            Some(image)
        });
        let fin_glyphs = array::from_fn(|fin| {
            let path = path.join(format!("src/fin/{}/glyphs.bmp", FIN_CHARS[fin as usize]));
            let Ok(reader) = ImageReader::open(path) else { return None; };
            let Ok(image) = reader.decode() else { return None; };
            Some(image)
        });

        Ok(Workspace {
            path,
            global_config,
            ini_configs,
            mid_configs,
            fin_configs,
            ini_glyphs,
            mid_glyphs,
            fin_glyphs,
        })
    }

    pub fn find_ini_variant(&self, ini: u32, mid: u32, fin: u32) -> Option<u32> {
        self.ini_configs[ini as usize].conditions.iter()
            .find(|ConditionEntry {condition, ..}| condition.matches(ini, mid, fin))
            .and_then(|ConditionEntry {variant, ..}| variant.clone())
    }

    pub fn find_mid_variant(&self, ini: u32, mid: u32, fin: u32) -> Option<u32> {
        self.mid_configs[mid as usize].conditions.iter()
            .find(|ConditionEntry {condition, ..}| condition.matches(ini, mid, fin))
            .and_then(|ConditionEntry {variant, ..}| variant.clone())
    }

    pub fn find_fin_variant(&self, ini: u32, mid: u32, fin: u32) -> Option<u32> {
        self.fin_configs[fin as usize].conditions.iter()
            .find(|ConditionEntry {condition, ..}| condition.matches(ini, mid, fin))
            .and_then(|ConditionEntry {variant, ..}| variant.clone())
    }
}
