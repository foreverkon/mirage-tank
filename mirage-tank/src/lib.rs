use std::path::Path;

use image::{DynamicImage, Pixel};

pub struct MirageTank {
    white_img: DynamicImage,
    black_img: DynamicImage,
}

pub enum Resize {
    No,
    Auto,
    To(u32, u32),
}

fn resize(src: &DynamicImage, nwidth: u32, nheight: u32) -> DynamicImage {
    let (width, height) = (src.width(), src.height());
    if nwidth < width && nheight < height {
        src.thumbnail_exact(nwidth, nheight)
    } else if nwidth != width || nheight != height {
        src.resize_exact(nwidth, nheight, image::imageops::CatmullRom)
    } else {
        src.clone()
    }
}

fn demensions(img: &DynamicImage) -> (u32, u32) {
    (img.width(), img.height())
}

impl MirageTank {
    const WHITE_RGB: image::Rgb<u8> = image::Rgb([255, 255, 255]);
    const BLACK_RGB: image::Rgb<u8> = image::Rgb([0, 0, 0]);

    pub fn resize_auto(&mut self) {
        let width = self.white_img.width().min(self.black_img.width());
        let height = self.white_img.height().min(self.black_img.height());
        self.white_img = resize(&self.white_img, width, height);
        self.black_img = resize(&self.black_img, width, height);
    }

    pub fn resize_to(&mut self, width: u32, height: u32) {
        self.white_img = resize(&self.white_img, width, height);
        self.black_img = resize(&self.black_img, width, height);
    }

    pub fn new(white_img: DynamicImage, black_img: DynamicImage) -> Self {
        Self {
            white_img,
            black_img,
        }
    }

    pub fn new_from_file<P>(white_img_path: P, black_img_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let white_img = image::io::Reader::open(white_img_path)?
            .with_guessed_format()?
            .decode()?;

        let black_img = image::io::Reader::open(black_img_path)?
            .with_guessed_format()?
            .decode()?;

        Ok(Self {
            white_img,
            black_img,
        })
    }

    pub fn grey_car(
        &mut self,
        white_ligth: f64,
        black_light: f64,
        checkboard: bool,
        resize: Resize,
    ) -> anyhow::Result<DynamicImage> {
        match resize {
            Resize::No => {}
            Resize::Auto => self.resize_auto(),
            Resize::To(w, h) => self.resize_to(w, h),
        }

        if demensions(&self.white_img) != demensions(&self.black_img) {
            anyhow::bail!("The images are not the same size");
        };

        let (width, height) = demensions(&self.white_img);

        let white_rgb = self
            .white_img
            .as_rgb8()
            .ok_or_else(|| anyhow::anyhow!("Can not be convert to RGB"))?;

        let black_rgb = self
            .black_img
            .as_rgb8()
            .ok_or_else(|| anyhow::anyhow!("Can not be convert to RGB"))?;

        let mut output = image::RgbaImage::new(width, height);

        for (((x, y, mut white), mut black), car) in white_rgb
            .enumerate_pixels()
            .zip(black_rgb.pixels())
            .zip(output.pixels_mut())
        {
            if checkboard {
                if (x + y) % 2 == 0 {
                    black = &Self::WHITE_RGB;
                } else {
                    white = &Self::BLACK_RGB;
                }
            }

            let wc = f64::from(white.to_luma().0[0]) * white_ligth;
            let bc = f64::from(black.to_luma().0[0]) * black_light;

            let a = (255.0 - wc + bc).min(255.0).max(0.0);
            let r = (bc / a * 255.0).min(255.0);

            let a = a.round() as u8;
            let r = r.round() as u8;
            *car = image::Rgba([r, r, r, a]);
        }

        Ok(DynamicImage::ImageRgba8(output))
    }

    pub fn color_car(
        &mut self,
        white_ligth: f64,
        black_light: f64,
        checkboard: bool,
        resize: Resize,
        white_color: f64,
        black_color: f64,
    ) -> anyhow::Result<DynamicImage> {
        match resize {
            Resize::No => {}
            Resize::Auto => self.resize_auto(),
            Resize::To(w, h) => self.resize_to(w, h),
        }

        if demensions(&self.white_img) != demensions(&self.black_img) {
            anyhow::bail!("The images are not the same size");
        };

        let (width, height) = demensions(&self.white_img);

        let white_rgb = self
            .white_img
            .as_rgb8()
            .ok_or_else(|| anyhow::anyhow!("Can not be convert to RGB"))?;

        let black_rgb = self
            .black_img
            .as_rgb8()
            .ok_or_else(|| anyhow::anyhow!("Can not be convert to RGB"))?;

        let mut output = image::RgbaImage::new(width, height);

        let mut wrgb: [f64; 3] = [0., 0., 0.];
        let mut brgb: [f64; 3] = [0., 0., 0.];
        let mut drgb: [f64; 3] = [0., 0., 0.];

        for (((x, y, mut white), mut black), car) in white_rgb
            .enumerate_pixels()
            .zip(black_rgb.pixels())
            .zip(output.pixels_mut())
        {
            if checkboard {
                if (x + y) % 2 == 0 {
                    black = &Self::WHITE_RGB;
                } else {
                    white = &Self::BLACK_RGB;
                }
            }

            let wgrey = f64::from(white.to_luma().0[0]);
            for i in 0..3 {
                wrgb[i] = f64::from(white.0[i]) * white_color + wgrey * (1.0 - white_color);
                wrgb[i] *= white_ligth;
            }

            let bgrey = f64::from(black.to_luma().0[0]);
            for i in 0..3 {
                brgb[i] = f64::from(black.0[i]) * black_color + bgrey * (1.0 - black_color);
                brgb[i] *= black_light;
            }

            for i in 0..3 {
                drgb[i] = 255.0 - wrgb[i] + brgb[i];
            }

            let a = (drgb[0] * 0.2126 + drgb[1] * 0.7152 + drgb[2] * 0.0722)
                .max(brgb[0].max(brgb[1]).max(brgb[2]))
                .min(255.0);

            let r = ((brgb[0] / a).min(1.0) * 255.0).round() as u8;
            let g = ((brgb[1] / a).min(1.0) * 255.0).round() as u8;
            let b = ((brgb[2] / a).min(1.0) * 255.0).round() as u8;
            let a = a.round() as u8;

            *car = image::Rgba([r, g, b, a]);
        }

        Ok(DynamicImage::ImageRgba8(output))
    }
}

#[cfg(test)]
mod tests {}
