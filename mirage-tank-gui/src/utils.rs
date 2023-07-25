use std::path::PathBuf;

use rfd::FileDialog;

pub fn pick_img() -> Option<PathBuf> {
    let dialog = FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "bmp"]);
    dialog.pick_file()
}

pub fn save_img() -> Option<PathBuf> {
    let dialog = FileDialog::new().add_filter("Image", &["png"]);
    dialog.save_file()
}
