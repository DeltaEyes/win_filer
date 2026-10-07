use eframe::egui;
use std::fs;
use std::sync::Arc; // Arcを使うために追加（.into()を使う場合は不要ですが、より明示的に書く場合）

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Win Filer (Pure Rust)",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    current_path: String,
    entries: Vec<String>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut app = Self {
            current_path: "C:\\".to_owned(),
            entries: Vec::new(),
        };
        app.refresh_entries();
        app
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();

        if let Ok(read_dir) = fs::read_dir(&self.current_path) {
            for entry in read_dir {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let prefix = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        "📁"
                    } else {
                        "📄"
                    };
                    self.entries.push(format!("{} {}", prefix, name));
                }
            }
        } else {
            self.entries
                .push("❌ フォルダを読み込めませんでした".to_string());
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_data = fs::read("C:\\Windows\\Fonts\\meiryo.ttc")
        .or_else(|_| fs::read("C:\\Windows\\Fonts\\msgothic.ttc"));

    if let Ok(data) = font_data {
        // ▼ ここを修正しました ▼
        fonts.font_data.insert(
            "japanese".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(data)), // Arc::new で包むか、.into() を使います
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "japanese".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "japanese".to_owned());

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("爆速ファイルマネージャー");

        ui.horizontal(|ui| {
            ui.label("現在のパス:");
            let response = ui.text_edit_singleline(&mut self.current_path);

            if ui.button("移動").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.refresh_entries();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.entries {
                ui.label(entry);
            }
        });
    }
}
