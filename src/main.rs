#![cfg_attr(all(not(debug_assertions), feature = "gui"), windows_subsystem = "windows")]

mod args;
mod core;
mod ui;

#[cfg(feature = "gui")]
mod gui_main;

#[cfg(not(feature = "gui"))]
mod cli_main;

fn main() {
    #[cfg(feature = "gui")]
    {
        use rfd::MessageDialog;
        if let Err(e) = gui_main::run() {
            MessageDialog::new()
                .set_title("エラー")
                .set_description(&e)
                .show();
        }
    }

    #[cfg(not(feature = "gui"))]
    {
        if let Err(e) = cli_main::run() {
            eprintln!("エラー: {}", e);
            std::process::exit(1);
        }
    }
}
