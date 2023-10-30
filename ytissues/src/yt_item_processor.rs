use std::error::Error;
use std::fmt;
use std::vec::Vec;
use chrono::{DateTime, Utc};

pub type UtcDateTime = DateTime<Utc>;

#[allow(dead_code)]
pub struct YtItem {
    id: String,
    created: UtcDateTime,
    updated: UtcDateTime,
    summary: String,
    reporter_login: String,
}

//pub type YtItems = Vec<YtItem>;
//pub struct YtItems(Vec<YtItem>);
pub struct YtItems{ 
    pub items: Vec<YtItem>
}

impl YtItem {
    const YT_ITEM_FIELD_IDREADABLE: &str = "idReadable";
    const YT_ITEM_FIELD_CREATED: &str = "created";
    const YT_ITEM_FIELD_UPDATED: &str = "updated";
    const YT_ITEM_FIELD_SUMMARY: &str = "summary";
    const YT_ITEM_FIELD_REPORTERLOGIN: &str = "reporter/login";

    fn get_nested<'l>(item: &'l serde_json::Value, field_path: &str) -> &'l serde_json::Value {
        match field_path.split_once("/") {
            None => &item[field_path],
            Some((parent, rest)) => Self::get_nested(&item[parent], rest)
        }
    }

    fn field_to_datetime(field_name: &str, item: &serde_json::Value) -> Result<UtcDateTime, Box<dyn Error>> {
        let item = Self::get_nested(item, field_name);
        let ms = item
            .as_u64()
            .ok_or(format!("Unable to parse date. Item: {field_name}"))?;
        Ok(DateTime::UNIX_EPOCH + std::time::Duration::from_millis(ms))
    }
    fn field_to_string(field_name: &str, item: &serde_json::Value) -> Result<String, Box<dyn Error>> {
        let item = Self::get_nested(item, field_name);
        let s = item
                        .as_str()
                        .ok_or(format!("Unable to parse string. Item: {field_name}"))?;
        Ok(s.into())
    }

    fn parse(item: &serde_json::Value) -> YtItem {
        let id = Self::field_to_string(Self::YT_ITEM_FIELD_IDREADABLE, item).expect("Unable to parse id");
        let summary = Self::field_to_string(Self::YT_ITEM_FIELD_SUMMARY, item).expect("Unable to parse summary");
        let created = Self::field_to_datetime(Self::YT_ITEM_FIELD_CREATED, item).expect("Unable to parse date");
        let updated = Self::field_to_datetime(Self::YT_ITEM_FIELD_UPDATED, item).expect("Unable to parse date");
        let reporter_login = Self::field_to_string(Self::YT_ITEM_FIELD_REPORTERLOGIN, item).expect("Unable to parse reporter");
        YtItem { 
                id: id,
                created: created,
                updated: updated,
                summary: summary,
                reporter_login: reporter_login
        }
    }
}
impl fmt::Display for YtItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{: >9}, c/u: {}/{} by: {:>12}: {}", self.id, self.created, self.updated, self.reporter_login, self.summary)
    }
}

impl fmt::Display for YtItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_string = self.items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", as_string)
    }
}

fn item_as_array(item: &serde_json::Value) -> &Vec<serde_json::Value> {
    let Some(arr) = item.as_array() else {
        println!("item: {:?}", item);
        panic!("item is not an array!");
    };
    arr
}

pub fn parse_items(item: &serde_json::Value) -> YtItems {
    let items = item_as_array(item)
        .iter()
        .map(YtItem::parse)
        .collect();
    YtItems { items: items }
}

// pub fn describe_yt_items(items: &YtItems) {
//     for item in items.items.iter() {
//         println!("{}", item);
//     }
// }