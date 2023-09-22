use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use crate::object::{Form, Object};
use crate::object::builders::*;
use eframe::egui;

mod object;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
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
    objects: HashSet<Object>,
    imgs: Vec<egui_extras::RetainedImage>,
    allowed_to_close: bool,
    show_confirmation_dialog: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut objects = load_objects("object_store");
        import_file("diagmetr.tif",&mut objects).unwrap();
        Self {
            objects,
            imgs: vec![],
            allowed_to_close: false,
            show_confirmation_dialog: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::new([true, true]).show(ui, |ui| {
                if ui.button("Refresh").clicked() {
                    self.imgs.clear();
                    for object in self.objects.iter().filter(|o| o.form == Form::Photo) {
                        let img = egui_extras::RetainedImage::from_image_bytes("img",&object.data).unwrap();
                        self.imgs.push(img);
                    }
                }
                for img in self.imgs.iter() {
                    ui.image(img.texture_id(ui.ctx()), img.size_vec2());
                }
            });
        });
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

pub fn import_file(path: &str, objects: &mut HashSet<Object>) -> Result<(), magic::MagicError> {
    if let Some(mut file) = File::open(path).ok() {
        let mut data = vec![];
        let cookie_mime = magic::Cookie::open(magic::CookieFlags::ERROR | magic::CookieFlags::MIME_TYPE)?;
        cookie_mime.load::<&str>(&[])?;
        if let _ = file.read_to_end(&mut data).ok() {
            match cookie_mime.buffer(data.as_slice())? {
                x if x.starts_with("text/plain") => objects.insert(plain_text(String::from_utf8_lossy(data.as_slice()).to_string())),
                x if x.starts_with("image/") => objects.insert(photo(data)),
                _ => objects.insert(binary(data)),
            };
        }
    }
    Ok(())
}