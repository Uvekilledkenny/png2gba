#![allow(dead_code)]

extern crate image;

use self::image::RgbImage;
use self::image::Pixel;

const PALETTE_SIZE: usize = 256;

struct ImgGBA {
    data: RgbImage,
    tile: bool,
    r: u32,
    c: u32,
    tr: u32,
    tc: u32,
}

impl ImgGBA {
    fn new(img: RgbImage, tile: bool) -> ImgGBA {
        ImgGBA {
            data: img,
            tile: tile,
            r: 0,
            c: 0,
            tr: 0,
            tc: 0,
        }
    }
}

impl Iterator for ImgGBA {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        if self.r == self.data.height() {
            return None;
        }

        let pixel = self.data.get_pixel(self.c, self.r).channels();

        if !self.tile {
            self.c += 1;

            if self.c == self.data.width() {
                self.r += 1;
                self.c = 0;
            }

        } else {
            self.c += 1;
            self.tc += 1;

            if self.tc == 8 {
                self.r += 1;
                self.tr += 1;
                self.c -= 8;
                self.tc = 0;

                if self.tr == 8 {
                    self.r -= 8;
                    self.tr = 0;
                    self.c += 8;
                }

                if self.c >= self.data.width() {
                    self.tc = 0;
                    self.tr = 0;
                    self.c = 0;
                    self.r += 8;
                }
            }
        }

        Some(to_bgr((pixel[0] as u32) << 16 | (pixel[1] as u32) << 8 | pixel[2] as u32))
    }
}

fn to_bgr(rgb: u32) -> u16 {
    let (r, g, b) = (rgb >> 16, (rgb >> 8) & 0xFF, rgb & 0xFF);
    let (rb, gb, bb) = (r >> 3, g >> 3, b >> 3);
    ((bb << 10) | (gb << 5) | rb) as u16
}

fn insert_palette(color: u16, palette: &mut Vec<u16>) -> Result<u8, &'static str> {
    for (i, color_pal) in (*palette).iter().enumerate() {
        if *color_pal == color {
            return Ok(i as u8);
        }
    }

    if palette.len() == (PALETTE_SIZE - 1) {
        return Err("Palette Overflow");
    }

    palette.push(color);
    Ok((palette.len() - 1) as u8)
}

pub fn to_data(data: &[u8], tile: bool) -> Vec<u16> {
    let e = image::load_from_memory(data).unwrap().to_rgb();
    let x = ImgGBA::new(e, tile);

    let mut data = Vec::<u16>::new();

    for (_, rgb) in x.enumerate() {
        data.push(rgb);
    }

    data
}

pub fn to_data_palette(data: &[u8],
                       alpha: u32,
                       tile: bool)
                       -> Result<(Vec<u8>, Vec<u16>), &'static str> {
    let e = image::load_from_memory(data).unwrap().to_rgb();
    let x = ImgGBA::new(e, tile);

    let mut data = Vec::<u8>::new();
    let mut palette = Vec::<u16>::new();

    palette.push(to_bgr(alpha));

    for (_, rgb) in x.enumerate() {
        let i = insert_palette(rgb, &mut palette)?;
        data.push(i);
    }

    for _ in 0..(PALETTE_SIZE - palette.len()) {
        palette.push(0);
    }

    Ok((data, palette))
}