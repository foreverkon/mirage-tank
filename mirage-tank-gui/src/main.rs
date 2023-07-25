#![allow(unused)]

mod utils;

use std::{
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use image::{DynamicImage, GenericImageView};
use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Rgba8Pixel};

fn pick_image(
    ui: &AppWindow,
    container_width: f32,
    container_height: f32,
) -> Result<(Image, DynamicImage), Box<dyn std::error::Error>> {
    let path = utils::pick_img();

    if let Some(path) = path {
        let global_images = Images::get(&ui);
        let image = image::io::Reader::open(path)?
            .with_guessed_format()?
            .decode()?
            .into_rgb8();

        let image_width = image.width() as f32;
        let image_height = image.height() as f32;

        let container_width = container_width as f32;
        let container_height = container_height as f32;

        let scale = {
            let scale_x = container_width / image_width;
            let scale_y = container_height / image_height;
            scale_x.min(scale_y).min(1.0)
        };

        global_images.set_width(image_width * scale);
        global_images.set_height(image_height * scale);

        let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
            image.as_raw(),
            image.width(),
            image.height(),
        );

        Ok((Image::from_rgb8(buffer), DynamicImage::ImageRgb8(image)))
    } else {
        Err("No image selected".into())
    }
}

fn generate(
    wimg: DynamicImage,
    bimg: DynamicImage,
    wlight: f32,
    blight: f32,
    wcolor: f32,
    bcolor: f32,
    resize: mirage_tank::Resize,
    colorful: bool,
    checkboard: bool,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut tank = mirage_tank::MirageTank::new(wimg, bimg);
    if colorful {
        if let Ok(img) = tank.color_car(
            wlight.into(),
            blight.into(),
            checkboard,
            resize,
            wcolor.into(),
            bcolor.into(),
        ) {
            Ok(img)
        } else {
            Err("Error generating image".into())
        }
    } else {
        if let Ok(img) = tank.grey_car(
            wlight.into(),
            blight.into(),
            checkboard,
            resize
        ) {
            Ok(img)
        } else {
            Err("Error generating image".into())
        }
    }
}

slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let white_img: Arc<Mutex<Option<DynamicImage>>> = Arc::new(Mutex::new(None));
    let black_img: Arc<Mutex<Option<DynamicImage>>> = Arc::new(Mutex::new(None));
    let out_img: Arc<Mutex<Option<DynamicImage>>> = Arc::new(Mutex::new(None));

    let ui = AppWindow::new()?;

    ui.on_pickWhiteImg({
        let ui = ui.as_weak();
        let white_img = white_img.clone();
        move |container_width, container_height| {
            let ui = ui.unwrap();
            if let Ok((image, raw_img)) = pick_image(&ui, container_width, container_height) {
                let mut white_img = white_img.lock().unwrap();
                (*white_img).replace(raw_img);
                Images::get(&ui).set_wimage(image);
            }
        }
    });

    ui.on_pickBlackImg({
        let ui = ui.as_weak();
        let black_img = black_img.clone();
        move |container_width, container_height| {
            let ui = ui.unwrap();
            if let Ok((image, raw_img)) = pick_image(&ui, container_width, container_height) {
                let mut black_img = black_img.lock().unwrap();
                (*black_img).replace(raw_img);
                Images::get(&ui).set_bimage(image);
            }
        }
    });

    ui.on_generate({
        let ui = ui.as_weak();
        let white_img = white_img.clone();
        let black_img = black_img.clone();
        let out_img = out_img.clone();

        move |wlight, blight, wcolor, bcolor, resize, colorful, checkboard| {
            let ui = ui.unwrap();
            let global_images = Images::get(&ui);

            let white_img = white_img.lock().unwrap();
            let black_img = black_img.lock().unwrap();

            if white_img.is_some() && black_img.is_some() {
                let white_img = (*white_img).as_ref().unwrap();
                let black_img = (*black_img).as_ref().unwrap();

                let resize = if resize {
                    mirage_tank::Resize::To(
                        global_images.get_width() as u32,
                        global_images.get_height() as u32,
                    )
                } else {
                    mirage_tank::Resize::Auto
                };

                if let Ok(img) = generate(
                    white_img.clone(),
                    black_img.clone(),
                    wlight,
                    blight,
                    wcolor,
                    bcolor,
                    resize,
                    colorful,
                    checkboard,
                ) {
                    let img = img.into_rgba8();
                    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        img.as_raw(),
                        img.width(),
                        img.height(),
                    );
                    let mut out_img = out_img.lock().unwrap();
                    (*out_img).replace(DynamicImage::ImageRgba8(img));
                    Images::get(&ui).set_oimage(Image::from_rgba8(buffer));
                }
            }
        }
    });

    ui.on_save({
        let out_img = out_img.clone();
        move || {
            let out_img = out_img.lock().unwrap();
            if out_img.is_some() {
                if let Some(path) = utils::save_img() {
                    let out_img = (*out_img).as_ref().unwrap();
                    if let Err(e) = out_img.save(path) {
                        println!("Error saving image: {}", e);
                    }
                }
            }
        }
    });

    ui.run()
}
