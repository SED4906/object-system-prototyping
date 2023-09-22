use std::collections::HashSet;



use crate::object::{Form, Object, Tag};


pub fn empty() -> Object {
    Object {data: vec![], tags: HashSet::new(), form: Form::Empty}
}

pub fn plain_text(data: String) -> Object {
    Object {data: data.as_bytes().to_vec(), tags: HashSet::new(), form: Form::PlainText}
}

pub fn binary(data: Vec<u8>) -> Object {
    Object {data: data.clone(), tags: HashSet::new(), form: Form::Binary}
}

pub fn photo(data: Vec<u8>) -> Object {
    let mut object = Object {data: data.clone(), tags: HashSet::new(), form: Form::Photo};
    if let Some(fields) = exif::parse_exif(data.as_slice()).ok() {
        for field in fields.0 {
            let tag = field.tag.description().unwrap_or(field.tag.number().to_string().as_str()).to_string();
            let value = field.display_value().to_string();
            object.tags.insert(Tag::Exif {tag, value});
        }
    }
    object
}

impl Object {
    pub fn display(&self) {
        match &self.form {
            Form::Empty => {
                println!("--- empty object ---");
            }
            Form::Photo => {
                println!("Contents --- \n(image data)\nTags --- \n{}",&self.tags.iter().map(|tag| format!("{}",tag)).collect::<Vec<_>>().join("\n"));
            }
            Form::PlainText => {
                println!("{}",String::from_utf8_lossy(&self.data.as_slice()));
            }
            _ => println!("Contents --- \n{}\nTags --- \n{}",readablificate(&self.data), &self.tags.iter().map(|tag| format!("{}",tag)).collect::<Vec<_>>().join("\n")),
        }
    }
}

fn readablificate(data: &Vec<u8>) -> String {
    let mut string = String::new();
    let mut position = 0;
    loop {
        let result = String::from_utf8(data.split_at(position).1.to_vec());
        match result {
            Err(e) => {
                let offset = e.utf8_error().valid_up_to();
                string.push_str(String::from_utf8(data.split_at(position).1[..offset].to_vec()).unwrap().as_str());
                string.push('\\');
                string.push_str(data.split_at(position).1.to_vec()[offset].escape_ascii().to_string().as_str());
                position += offset + 1;
            }
            Ok(s) => {
                string.push_str(s.split('\\').collect::<Vec<_>>().join("\\\\").as_str());
                break;
            }
        }
    }
    string
}