use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Mul;
use crate::object::{Form, Object};
use crate::object::builders::*;
use eframe::egui;
use eframe::egui::Vec2;
use crate::magic_identify::magic_identify;

mod object;
mod magic_identify;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(650.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Object Viewer",
        options,
        Box::new(|_cc| {
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    query: String,
    objects: HashSet<Object>,
    imgs: Vec<(egui_extras::RetainedImage, Object)>,
    ptxts: Vec<(String, Object)>,
    bins: Vec<(String, Object)>,
    picked: Option<usize>,
    picktype: Form,
    allowed_to_close: bool,
    show_confirmation_dialog: bool,
    dropped_files: Vec<egui::DroppedFile>,
    ask_to_delete: bool,
    starting_up: bool,
    size: usize,
    show_imgs: bool,
    show_ptxts: bool,
    show_bins: bool,
    show_show: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let objects = load_objects("object_store");
        Self {
            query: String::new(),
            objects,
            imgs: vec![],
            ptxts: vec![],
            bins: vec![],
            picked: None,
            picktype: Form::Empty,
            allowed_to_close: false,
            show_confirmation_dialog: false,
            dropped_files: vec![],
            ask_to_delete: false,
            starting_up: true,
            size: 128,
            show_ptxts: true,
            show_imgs: true,
            show_bins: true,
            show_show: false,
        }
    }
}

impl MyApp {
    fn refresh_images(&mut self) {
        self.imgs.clear();
        self.picked = None;
        self.picktype = Form::Empty;
        for object in self.objects.iter().filter(|o| o.form == Form::Photo).filter(|o| o.search(self.query.clone())) {
            let img = egui_extras::RetainedImage::from_image_bytes("img",&object.data).unwrap();
            self.imgs.push((img, object.clone()));
        }
    }

    fn refresh_plaintext(&mut self) {
        self.ptxts.clear();
        self.picked = None;
        self.picktype = Form::Empty;
        for object in self.objects.iter().filter(|o| o.form == Form::PlainText).filter(|o| o.search(self.query.clone())) {
            self.ptxts.push((String::from_utf8_lossy(object.data.as_slice()).to_string(),object.clone()));
        }
    }

    fn refresh_bins(&mut self) {
        self.bins.clear();
        self.picked = None;
        self.picktype = Form::Empty;
        for object in self.objects.iter().filter(|o| o.form == Form::Binary).filter(|o| o.search(self.query.clone())) {
            self.bins.push((hex::encode(&object.data),object.clone()));
        }
    }

    fn refresh(&mut self) {
        self.refresh_images();
        self.refresh_plaintext();
        self.refresh_bins();
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.starting_up {
            self.refresh();
            self.starting_up = false;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(if self.picked.is_none() {"Refresh"} else {"Back"}).clicked() {
                    self.refresh();
                }
                if ui.text_edit_singleline(&mut self.query).changed() {
                    self.refresh();
                }
                if self.picked.is_none() {
                    ui.add(egui::Slider::new(&mut self.size, 32..=256).text("Size"));
                    ui.checkbox(&mut self.show_show, "Show/hide");

                }
                if self.picked.is_some() {
                    if ui.button("🗑").clicked() {
                        self.ask_to_delete = true;
                    }
                }
            });

            if !self.dropped_files.is_empty() {
                for file in &self.dropped_files {
                    import_file(file.path.clone().unwrap().as_path().to_str().unwrap(), &mut self.objects);
                }
                self.refresh();
                self.dropped_files.clear();
            }

            preview_files_being_dropped(ctx);

            // Collect dropped files:
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    self.dropped_files = i.raw.dropped_files.clone();
                }
            });

            egui::ScrollArea::new([true, true]).show(ui, |ui| {
                if let Some(picked) = self.picked {
                    match self.picktype {
                        Form::Photo => {
                            ui.image(self.imgs[picked].0.texture_id(ui.ctx()),  self.imgs[picked].0.size_vec2());
                            ui.label(&self.imgs[picked].1.tags.iter().map(|tag| format!("{}",tag)).collect::<Vec<_>>().join("\n"));
                        }
                        Form::Empty => {
                            ui.label("--- Empty object ---");
                        },
                        Form::PlainText => {
                            ui.label(&self.ptxts[picked].0);
                        },
                        Form::Binary => {
                            ui.label(&self.bins[picked].0);
                        }
                        _ => {}
                    }

                } else {

                    let mut index = 0;
                    if self.show_imgs {
                        for img in self.imgs.iter() {
                            ui.group(|ui| {
                                ui.image(img.0.texture_id(ui.ctx()), if img.0.size_vec2().max_elem() > self.size as f32 {
                                    img.0.size_vec2().normalized().mul(Vec2{x: self.size as f32, y: self.size as f32})
                                } else {
                                    img.0.size_vec2()
                                });
                                if let Some((index, form)) = more_info_bar(ui, index, &Form::Photo) {
                                    self.picked = Some(index);
                                    self.picktype = form;
                                }
                            });
                            index += 1;
                        }
                    }
                    if self.show_ptxts {
                        index = 0;
                        for ptxt in self.ptxts.iter() {
                            ui.group(|ui| {
                                ui.set_max_height(256.0);
                                ui.label(truncate_dotted(ptxt.0.clone(), self.size));
                                if let Some((index, form)) = more_info_bar(ui, index, &Form::PlainText) {
                                    self.picked = Some(index);
                                    self.picktype = form;
                                }
                            });
                            index += 1;
                        }
                    }
                    if self.show_bins {
                        index = 0;
                        for bin in self.bins.iter() {
                            ui.group(|ui| {
                                ui.set_max_height(256.0);
                                ui.label(truncate_dotted(bin.0.clone(), self.size));
                                if let Some((index, form)) = more_info_bar(ui, index, &Form::Binary) {
                                    self.picked = Some(index);
                                    self.picktype = form;
                                }
                            });
                            index += 1;
                        }
                    }
                }
            });
        });

        if self.ask_to_delete {
            if self.picked.is_none() {
                self.ask_to_delete = false;
            }
            // Show confirmation dialog:
            egui::Window::new("Really delete?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("No").clicked() {
                            self.ask_to_delete = false;
                        }

                        if ui.button("Yes!").clicked() {
                            match self.picktype {
                                Form::Photo => {
                                    self.objects.remove(&self.imgs[self.picked.unwrap()].1);
                                    self.picked = None;
                                    self.picktype = Form::Empty;
                                }
                                Form::Empty => {
                                },
                                Form::PlainText => {
                                    self.objects.remove(&self.ptxts[self.picked.unwrap()].1);
                                    self.picked = None;
                                    self.picktype = Form::Empty;
                                },
                                Form::Binary => {
                                    self.objects.remove(&self.bins[self.picked.unwrap()].1);
                                    self.picked = None;
                                    self.picktype = Form::Binary;
                                }
                                _ => {}
                            }
                            self.refresh();
                        }
                    });
                });
        }

        if self.show_show {
            egui::Window::new("Show/hide")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.show_imgs, "Photos");
                        ui.checkbox(&mut self.show_ptxts, "Plain texts");
                        ui.checkbox(&mut self.show_bins, "Binaries");
                    });
                });
        }

        if self.show_confirmation_dialog {
            // Show confirmation dialog:
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_confirmation_dialog = false;
                        }

                        if ui.button("Yes!").clicked() {
                            self.allowed_to_close = true;
                            save_objects(&self.objects, "object_store");
                            frame.close();
                        }
                    });
                });
        }
    }

    fn on_close_event(&mut self) -> bool {
        self.show_confirmation_dialog = true;
        self.allowed_to_close
    }
}

fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

pub fn load_objects(store: &str) -> HashSet<Object> {
    let mut objects: HashSet<Object> = HashSet::new();
    if let Some(mut file) = File::open(store).ok() {
        let mut data = vec![];
        if let Some(_) = file.read_to_end(&mut data).ok() {
            objects = serde_json::from_str(String::from_utf8_lossy(data.as_slice()).to_string().as_str()).unwrap();
        }
    }
    objects
}

pub fn save_objects(objects: &HashSet<Object>, store: &str) {
    if let Some(mut file) = File::create(store).ok() {
        let data = serde_json::to_string(&objects).unwrap();
        let _ = file.write(data.as_bytes()).unwrap();
    }
}

pub fn import_file(path: &str, objects: &mut HashSet<Object>) -> Option<()> {
    let mut file = File::open(path).ok()?;
    let mut data = vec![];
    let mut _len = file.read_to_end(&mut data).ok()?;
    import_file_bytes(data, objects)
}

pub fn import_file_bytes(data: Vec<u8>, objects: &mut HashSet<Object>) -> Option<()> {
    let form = magic_identify(data.as_slice());
    match form {
        Form::PlainText => objects.insert(plain_text(String::from_utf8_lossy(data.as_slice()).to_string())),
        Form::Photo => objects.insert(photo(data)),
        _ => objects.insert(binary(data)),
    };
    Some(())
}

pub fn truncate_dotted(s: String, to: usize) -> String {
    if to > 3 && s.len() > to {
        let mut s = s.chars().take(to-3).collect::<String>();
        s.push_str("…");
        s
    } else {
        s
    }
}

fn more_info_bar(ui: &mut egui::Ui, index: usize, form: &Form) -> Option<(usize, Form)>{
    let mut result = None;
    ui.horizontal(|ui| {
        if ui.button("🔧").clicked() {
            result = Some((index, form.clone()));
        }
        match form.clone() {
            Form::PlainText => ui.label("Text (Plain)"),
            Form::Photo => ui.label("Photo"),
            Form::Binary => ui.label("Binary (Unknown format)"),
            _ => ui.label("Unknown"),
        };
    });
    result
}