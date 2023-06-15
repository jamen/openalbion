slint::include_modules!();

use rfd::FileDialog;

fn main() -> Result<(), slint::PlatformError> {
    let directory = FileDialog::new().pick_folder();

    println!("{directory:?}");

    let ui = App::new()?;
    let ui_handle = ui.as_weak();

    ui.run()
}
