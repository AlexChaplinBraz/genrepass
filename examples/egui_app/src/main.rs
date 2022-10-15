use copypasta_ext::{prelude::ClipboardProvider, x11_bin::ClipboardContext};
use eframe::{
    egui::{Button, CentralPanel, DragValue, Key, Layout, ScrollArea, TextEdit, TopBottomPanel},
    emath::Align,
    run_native, App, NativeOptions,
};
use genrepass::PasswordSettings;
use rfd::FileDialog;

fn main() {
    let ctx = ClipboardContext::new().unwrap();

    let native_options = NativeOptions::default();

    run_native(
        "genrepass GUI",
        native_options,
        Box::new(|_cc| Box::new(Gui::new(ctx))),
    );
}

struct Gui {
    settings: PasswordSettings,
    passwords: Vec<String>,
    clipboard: ClipboardContext,
    manual_input: String,
}

impl Gui {
    fn new(clipboard: ClipboardContext) -> Self {
        Gui {
            settings: Default::default(),
            passwords: Default::default(),
            clipboard,
            manual_input: Default::default(),
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Readable Password Generator");
            ui.separator();

            ui.label("Amount of passwords to generate");
            ui.add(DragValue::new(&mut self.settings.pass_amount).speed(1));
            ui.separator();

            ui.label("Words");
            ui.horizontal(|ui| {
                ui.label("Input manually:");
                let text_edit_response = ui.add(TextEdit::singleline(&mut self.manual_input));
                let button_response = ui.button("Add words");

                if button_response.clicked()
                    || text_edit_response.lost_focus() && ui.input().key_pressed(Key::Enter)
                {
                    self.settings.get_words_from_str(&self.manual_input);
                    self.manual_input.clear();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Load words from files").clicked() {
                    if let Some(paths) = FileDialog::new().pick_files() {
                        for path in paths {
                            self.settings.get_words_from_path(path).unwrap();
                        }
                    }
                }

                if ui.button("Load words from directories").clicked() {
                    if let Some(paths) = FileDialog::new().pick_folders() {
                        for path in paths {
                            self.settings.get_words_from_path(path).unwrap();
                        }
                    }
                }
            });

            let words = self.settings.get_words();
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for word in words {
                        ui.label(word);
                    }
                });
            });
        });
        TopBottomPanel::bottom("passwords")
            .min_height(100.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    if self.settings.get_words().len() <= 1 {
                        ui.add_enabled(false, Button::new("Generate"))
                            .on_disabled_hover_text("Must have more than one word for generation");
                    } else if ui.button("Generate").clicked() {
                        self.passwords = self.settings.generate().unwrap();
                    }
                    if !self.passwords.is_empty() {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for password in &self.passwords {
                                    if ui.button(password).on_hover_text("Click to copy").clicked()
                                    {
                                        self.clipboard.set_contents(password.to_owned()).unwrap();
                                    }
                                }
                            });
                        });
                    }
                });
            });
    }
}
