use eframe::egui;
use std::fs;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "HIG-Inspired Filer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    path: String,
}

struct MyApp {
    current_path: String,
    search_query: String,
    entries: Vec<FileEntry>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut app = Self {
            current_path: "C:\\".to_owned(),
            search_query: String::new(),
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
                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let path = entry.path().to_string_lossy().to_string();

                    self.entries.push(FileEntry { name, is_dir, path });
                }
            }
            // フォルダを上に、ファイルを下にするソート
            self.entries
                .sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_data = fs::read("C:\\Windows\\Fonts\\meiryo.ttc")
        .or_else(|_| fs::read("C:\\Windows\\Fonts\\msgothic.ttc"));

    if let Ok(data) = font_data {
        fonts.font_data.insert(
            "japanese".to_owned(),
            egui::FontData::from_owned(data).into(),
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
        ui.horizontal(|ui| {
            // ---------------------------------------------------
            // 左側のサイドバー領域
            // ---------------------------------------------------
            ui.allocate_ui(egui::vec2(140.0, ui.available_height()), |ui| {
                ui.add_space(10.0);
                ui.heading("場所");
                ui.add_space(5.0);

                let shortcuts = [("Cドライブ", "C:\\"), ("Dドライブ", "D:\\")];
                for (name, path) in shortcuts {
                    if ui
                        .selectable_label(self.current_path == path, name)
                        .clicked()
                    {
                        self.current_path = path.to_owned();
                        self.search_query.clear();
                        self.refresh_entries();
                    }
                }
            });

            ui.separator();

            // ---------------------------------------------------
            // 右側のメイン画面領域
            // ---------------------------------------------------
            ui.allocate_ui(ui.available_size(), |ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("パス:");
                    let path_res = ui.add_sized(
                        [ui.available_width() - 200.0, 24.0],
                        egui::TextEdit::singleline(&mut self.current_path),
                    );
                    if path_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.refresh_entries();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized(
                            [150.0, 24.0],
                            egui::TextEdit::singleline(&mut self.search_query).hint_text("🔍 検索"),
                        );
                    });
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let query = self.search_query.to_lowercase();

                        // 【修正ポイント1】移動先のパスを一時保存する変数を準備
                        let mut next_dir = None;

                        let filtered_entries = self
                            .entries
                            .iter()
                            .filter(|e| query.is_empty() || e.name.to_lowercase().contains(&query));

                        for entry in filtered_entries {
                            let icon = if entry.is_dir { "📁" } else { "📄" };
                            let label = format!("{} {}", icon, entry.name);

                            let btn = egui::Button::new(label)
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false);

                            if ui.add(btn).clicked() {
                                if entry.is_dir {
                                    // 【修正ポイント2】ここでは直接更新せず、パスをメモするだけ
                                    next_dir = Some(entry.path.clone());
                                } else {
                                    println!("ファイルを開きます: {}", entry.path);
                                }
                            }
                        }

                        // 【修正ポイント3】ループ（読み込み）が安全に終わった後で更新処理（書き込み）を行う
                        if let Some(path) = next_dir {
                            self.current_path = path;
                            self.search_query.clear();
                            self.refresh_entries();
                        }
                    });
            });
        });
    }
}
