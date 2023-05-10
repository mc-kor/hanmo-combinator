pub mod workspace;
pub mod hangul;

use std::{fs::File, io::{BufWriter, Write}, env::current_dir};
use workspace::Workspace;
use eyre::eyre;
use hangul::{NUM_INI, NUM_MID, NUM_FIN, syllable_codepoint};
use image::{GenericImageView, DynamicImage};

fn copy_sqr(size: u32, from_img: &DynamicImage, from_x: u32, from_y: u32, to: &mut [u8; 32]) {
    for x in 0..size {
        for y in 0..size {
            let pixel = from_img.get_pixel(from_x + x, from_y + y);
            let to_idx = x + y * size;
            if pixel[0] < 64 && pixel[1] < 64 && pixel[2] < 64 && pixel[3] >= 192 {
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

    let mut out = BufWriter::new(File::create("out.hex")?);

    for ini in 0..NUM_INI {
        for mid in 0..NUM_MID {
            for fin in 0..NUM_FIN {
                let ini_variant = workspace.find_ini_variant(ini, mid, fin).ok_or(eyre!("couldn't find ini variant for {ini} {mid} {fin}"))?;
                let mid_variant = workspace.find_mid_variant(ini, mid, fin).ok_or(eyre!("couldn't find ini variant for {ini} {mid} {fin}"))?;
                let fin_variant = workspace.find_fin_variant(ini, mid, fin).ok_or(eyre!("couldn't find ini variant for {ini} {mid} {fin}"))?;

                let mut to_buf = [0u8; 32];
                copy_ini(&workspace, ini, ini_variant, &mut to_buf);
                copy_mid(&workspace, mid, mid_variant, &mut to_buf);
                copy_fin(&workspace, fin, fin_variant, &mut to_buf);

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
