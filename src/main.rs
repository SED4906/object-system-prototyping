use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use crate::object::{binary, Object, photo, plain_text};

mod object;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut objects: HashSet<Object> = HashSet::new();
    objects.insert(object::empty());
    if let Some(file) = File::open("README.md").ok() {
        import_file(file, &mut objects)?;
    }
    if let Some(file) = File::open("diagmetr.tif").ok() {
        import_file(file, &mut objects)?;
    }
    for x in objects.iter() {x.display()};
    Ok(())
}

fn import_file(mut file: File, mut objects: &mut HashSet<Object>) -> Result<(), magic::MagicError> {
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
    Ok(())
}