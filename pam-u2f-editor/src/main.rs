mod ui;

use eframe::NativeOptions;

fn main() {
    tracing_subscriber::fmt().init();
    eframe::run_native(
        "PAM U2F Editor",
        NativeOptions::default(),
        Box::new(|_creation_context| Box::new(ui::Editor::new())),
    )
}
