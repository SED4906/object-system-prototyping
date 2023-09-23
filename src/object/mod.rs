use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

pub mod builders;

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub struct Location {
    place: Option<String>,
    lat_long: (u64, u64), // *10000
}

impl Object {
    pub fn search(&self, query: String) -> bool {
        match self.form {
            Form::Empty => false,
            Form::Binary => {
                hex::encode(&self.data.as_slice()).contains(&query)
            }
            Form::Photo => {
                for tag in &self.tags {
                    match tag {
                        Tag::Exif {value,..} => {
                            if value.contains(&query) {
                                return true;
                            }
                        },
                        _ => {
                            return false;
                        }
                    };
                }
                if self.tags.is_empty() && query.is_empty() {
                    return true;
                }
                false
            }
            Form::PlainText => String::from_utf8_lossy(self.data.as_slice()).contains(&query),
            _ => false
        }
    }
}