use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

pub struct Object {
    pub data: Vec<u8>,
    pub tags: HashSet<Tag>,
    pub form: Form,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Tag {
    Category(String),
    Exif {
        tag: String,
        value: String,
    },
    Title(String),
    Author(String),
    Date {
        value: DateTime,
        concerns: DateConcerns,
    },
    OtherUnknown {
        tag: String,
        value: String,
    },
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Category(s) => f.write_fmt(format_args!("Category | {s}")),
            Self::Exif {tag, value} => f.write_fmt(format_args!("EXIF {tag} | {value}")),
            Self::Title(s) => f.write_fmt(format_args!("Title | {s}")),
            Self::Author(s) => f.write_fmt(format_args!("Author | {s}")),
            Self::Date{value, concerns} => f.write_fmt(format_args!("Date {concerns} | {value}")),
            Self::OtherUnknown{tag, value} => f.write_fmt(format_args!("Other {tag} | {value}")),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum DateConcerns {
    Created, // When an object was created.
    Added, // When the object was added to the system.
    Edited, // When the object was last edited.
    OtherUnknown(String),
}

impl Display for DateConcerns {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Created => f.write_str("Created"),
            Self::Added => f.write_str("Added"),
            Self::Edited => f.write_str("Edited"),
            Self::OtherUnknown(s) => f.write_str(s.as_str()),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Form {
    Empty,
    PlainText,
    TypesetText,
    Binary,
    App,
    Photo,
    Sound,
    Video,
    Model3D,
    Archive,
    OtherUnknown(String),
}

#[derive(Eq, Hash, PartialEq)]
pub struct DateTime {
    year: Option<i32>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}-{}-{} {}:{}:{}",
                                 if let Some(v) = self.year {
                                     v.to_string()
                                 } else {
                                     "####".into()
                                 },
                                 if let Some(v) = self.month {
                                     v.to_string()
                                 } else {
                                     "##".into()
                                 },
                                 if let Some(v) = self.day {
                                     v.to_string()
                                 } else {
                                     "##".into()
                                 },
                                 if let Some(v) = self.hour {
                                     v.to_string()
                                 } else {
                                     "##".into()
                                 },
                                 if let Some(v) = self.minute {
                                     v.to_string()
                                 } else {
                                     "##".into()
                                 },
                                 if let Some(v) = self.second {
                                     v.to_string()
                                 } else {
                                     "##".into()
                                 },
        ))
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct Location {
    place: Option<String>,
    lat_long: (u64, u64), // *10000
}

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
        match &self {
            x if x.form == Form::Empty => {
                println!("--- empty object ---");
            }
            x if x.form == Form::PlainText => {
                println!("{}",String::from_utf8_lossy(x.data.as_slice()));
            }
            _ => println!("Contents --- \n{}\nTags --- \n{}",readablificate(&self.data), &self.tags.iter().map(|tag| format!("{}",tag)).collect::<Vec<_>>().join("\n")),
        }
    }
}

fn readablificate(mut data: &Vec<u8>) -> String {
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