pub mod combinator_config;

use std::{fs::{self, File}, io::{BufWriter, Write}};
use combinator_config::CombinatorConfig;
use eyre::eyre;
use image::{io::Reader as ImageReader, GenericImageView, DynamicImage};

const NUM_INI: u32 = 19;
const NUM_MID: u32 = 21;
const NUM_FIN: u32 = 28;

fn copy_sqr(from_img: &DynamicImage, from_x: u32, from_y: u32, to: &mut [u8; 32]) {
    for x in 0..16 {
        for y in 0..16 {
            let pixel = from_img.get_pixel(from_x + x, from_y + y);
            let to_idx = x + y * 16;
            if pixel[0] < 64 && pixel[1] < 64 && pixel[2] < 64 && pixel[3] >= 192 {
                to[(to_idx / 8) as usize] |= 1u8 << (7 - to_idx % 8);
            }
        }
    }
}

fn copy_ini(config: &CombinatorConfig, from_img: &DynamicImage, ini: u32, variant: u32, to: &mut [u8; 32]) {
    let row = config.ini_row_start + variant;
    let col = ini;
    copy_sqr(from_img, col * 16, row * 16, to);
}

fn copy_mid(config: &CombinatorConfig, from_img: &DynamicImage, mid: u32, variant: u32, to: &mut [u8; 32]) {
    let row = config.mid_row_start + variant;
    let col = mid;
    copy_sqr(from_img, col * 16, row * 16, to);
}

fn copy_fin(config: &CombinatorConfig, from_img: &DynamicImage, fin: u32, variant: u32, to: &mut [u8; 32]) {
    if fin == 0 {
        return
    }
    let row = config.fin_row_start + variant;
    let col = fin - 1;
    copy_sqr(from_img, col * 16, row * 16, to);
}

fn syllable_codepoint(ini: u32, mid: u32, fin: u32) -> u32 {
    ('가' as u32) + ini * NUM_MID * NUM_FIN + mid * NUM_FIN + fin
}

fn find_ini_variant(config: &CombinatorConfig, ini: u32, mid: u32, fin: u32) -> Option<u32> {
    config.ini_variants.iter().enumerate()
        .find(|(_, cond)| cond.matches(ini, mid, fin))
        .map(|(idx, _)| idx as u32)
}

fn find_mid_variant(config: &CombinatorConfig, ini: u32, mid: u32, fin: u32) -> Option<u32> {
    config.mid_variants.iter().enumerate()
        .find(|(_, cond)| cond.matches(ini, mid, fin))
        .map(|(idx, _)| idx as u32)
}

fn find_fin_variant(config: &CombinatorConfig, ini: u32, mid: u32, fin: u32) -> Option<u32> {
    config.fin_variants.iter().enumerate()
        .find(|(_, cond)| cond.matches(ini, mid, fin))
        .map(|(idx, _)| idx as u32)
}

fn main() -> eyre::Result<()> {
    let config: CombinatorConfig = toml::from_str(&fs::read_to_string("config.toml")?)?;

    let from_img = ImageReader::open("hanmo-base.bmp")?.decode()?;
    let mut out = BufWriter::new(File::create("out.hex")?);

    for ini in 0..NUM_INI {
        for mid in 0..NUM_MID {
            for fin in 0..NUM_FIN {
                let ini_variant = find_ini_variant(&config, ini, mid, fin).ok_or(eyre!("couldn't find ini variant for {ini} {mid} {fin}"))?;
                let mid_variant = find_mid_variant(&config, ini, mid, fin).ok_or(eyre!("couldn't find ini variant for {ini} {mid} {fin}"))?;
                let fin_variant = find_fin_variant(&config, ini, mid, fin).ok_or(eyre!("couldn't find ini variant for {ini} {mid} {fin}"))?;

                let mut to_buf = [0u8; 32];
                copy_ini(&config, &from_img, ini, ini_variant, &mut to_buf);
                copy_mid(&config, &from_img, mid, mid_variant, &mut to_buf);
                copy_fin(&config, &from_img, fin, fin_variant, &mut to_buf);

                let syllable = syllable_codepoint(ini, mid, fin);
                write!(out, "{syllable:04X}:")?;
                for b in to_buf {
                    write!(out, "{b:02X}")?;
                }
                writeln!(out)?;
            }
        }
    }

    return Ok(());
}
