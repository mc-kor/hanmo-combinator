#![feature(fs_try_exists)]
#![feature(array_try_from_fn)]

pub mod workspace;
pub mod hangul;

use std::{fs::{File, self}, io::{BufWriter, Write}, env::current_dir};
use workspace::Workspace;
use hangul::{NUM_INI, NUM_MID, NUM_FIN, syllable_codepoint};
use image::{GenericImageView, DynamicImage};

fn copy_sqr(size: u32, from_img: &DynamicImage, from_x: u32, from_y: u32, to: &mut [u8; 32]) {
    for x in 0..size {
        for y in 0..size {
            let pixel = from_img.get_pixel(from_x + x, from_y + y);
            let to_idx = x + y * size;
            if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 && pixel[3] >= 192 {
                to[(to_idx / 8) as usize] |= 1u8 << (7 - to_idx % 8);
            }
        }
    }
}

fn copy_ini(workspace: &Workspace, ini: u32, variant: u32, to: &mut [u8; 32]) {
    let Some(from_img) = workspace.ini_glyphs[ini as usize].as_ref() else { return; };
    let num_col = from_img.width() / workspace.global_config.size;
    let row = variant / num_col;
    let col = variant % num_col;
    copy_sqr(workspace.global_config.size, &from_img, col * workspace.global_config.size, row * workspace.global_config.size, to);
}

fn copy_mid(workspace: &Workspace, mid: u32, variant: u32, to: &mut [u8; 32]) {
    let Some(from_img) = workspace.mid_glyphs[mid as usize].as_ref() else { return; };
    let num_col = from_img.width() / workspace.global_config.size;
    let row = variant / num_col;
    let col = variant % num_col;
    copy_sqr(workspace.global_config.size, &from_img, col * workspace.global_config.size, row * workspace.global_config.size, to);
}

fn copy_fin(workspace: &Workspace, fin: u32, variant: u32, to: &mut [u8; 32]) {
    let Some(from_img) = workspace.fin_glyphs[fin as usize].as_ref() else { return; };
    let num_col = from_img.width() / workspace.global_config.size;
    let row = variant / num_col;
    let col = variant % num_col;
    copy_sqr(workspace.global_config.size, &from_img, col * workspace.global_config.size, row * workspace.global_config.size, to);
}

fn main() -> eyre::Result<()> {
    let workspace = Workspace::load(current_dir()?)?;

    let out_dir = workspace.path.join(&workspace.global_config.out_dir);
    fs::create_dir_all(&out_dir)?;

    let mut out = BufWriter::new(File::create(out_dir.join("out.hex"))?);
    let mut out_json = BufWriter::new(File::create(out_dir.join("selection.json"))?);

    write!(out_json, "[")?;

    for ini in 0..NUM_INI {
        for mid in 0..NUM_MID {
            for fin in 0..NUM_FIN {
                let ini_variant = workspace.find_ini_variant(ini, mid, fin);
                let mid_variant = workspace.find_mid_variant(ini, mid, fin);
                let fin_variant = workspace.find_fin_variant(ini, mid, fin);

                if workspace.global_config.warn_no_match {
                    if ini_variant.is_none() { eprintln!("couldn't find ini variant for {ini} {mid} {fin}") };
                    if mid_variant.is_none() { eprintln!("couldn't find mid variant for {ini} {mid} {fin}") };
                    if fin != 0 && fin_variant.is_none() { eprintln!("couldn't find fin variant for {ini} {mid} {fin}") };
                }

                if ini != 0 || mid != 0 || fin != 0 { write!(out_json, ",")?; }
                write!(out_json, "[{},{},{}]",
                    ini_variant.map_or("null".to_owned(), |variant| variant.to_string()),
                    mid_variant.map_or("null".to_owned(), |variant| variant.to_string()),
                    fin_variant.map_or("null".to_owned(), |variant| variant.to_string()),
                )?;

                let mut to_buf = [0u8; 32];
                if let Some(ini_variant) = ini_variant { copy_ini(&workspace, ini, ini_variant, &mut to_buf); }
                if let Some(mid_variant) = mid_variant { copy_mid(&workspace, mid, mid_variant, &mut to_buf); }
                if let Some(fin_variant) = fin_variant { copy_fin(&workspace, fin, fin_variant, &mut to_buf); }

                let syllable = syllable_codepoint(ini, mid, fin);
                write!(out, "{syllable:04X}:")?;
                for b in to_buf {
                    write!(out, "{b:02X}")?;
                }
                writeln!(out)?;
            }
        }
    }

    write!(out_json, "]")?;

    return Ok(());
}
