use std::error::Error;
use std::fmt;
use std::vec::Vec;
use chrono::{DateTime, Utc};
use std::ops::{Deref, DerefMut};

pub type UtcDateTime = DateTime<Utc>;

#[allow(dead_code)]
pub struct YtItem {
    id: String,
    created: UtcDateTime,
    updated: UtcDateTime,
    summary: String,
    reporter_login: String,
    project: String,
    project_short: String,
    assignee: String,
    severity: String,
}

pub struct YtItems(Vec<YtItem>);

impl YtItem {
    const YT_ITEM_FIELD_IDREADABLE: &str = "idReadable";
    const YT_ITEM_FIELD_CREATED: &str = "created";
    const YT_ITEM_FIELD_UPDATED: &str = "updated";
    const YT_ITEM_FIELD_SUMMARY: &str = "summary";
    const YT_ITEM_FIELD_REPORTERLOGIN: &str = "reporter/login";
    const YT_ITEM_FIELD_PROJECT: &str = "project/name";
    const YT_ITEM_FIELD_PROJECTSHORT: &str = "project/shortName";
    const YT_ITEM_FIELD_ASSIGNEELOGIN: &str = "customFields/*name=Assignee/value/login";
    const YT_ITEM_FIELD_SEVERITY: &str = "customFields/*name=Defect Severity/value/name";

    fn get_nested<'l>(item: &'l serde_json::Value, field_path: &str) -> &'l serde_json::Value {
        fn find_in_array<'b>(item: &'b serde_json::Value, condition: &str) -> Option<&'b serde_json::Value> {
            //println!("Array: {:?}", item);
            let (field_name, field_value) = condition.split_once("=").unwrap();
            let arr = item_as_array(item);
            let item = arr
                .iter()
                .find(|item| item[field_name] == field_value)?;
            //println!("Found: {:?}", item);
            Some(item)
        }
        match field_path.split_once("/") {
            None => &item[field_path],
            Some((parent, rest)) => { 
                if !parent.is_empty() && matches!(parent.chars().nth(0), Some('*')) { 
                    let item_in_array = find_in_array(item, &parent[1..]).expect("Unable to find item in array");
                    Self::get_nested(item_in_array, rest)
                } else { 
                    Self::get_nested(&item[parent], rest)
                }
            }
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
        let project = Self::field_to_string(Self::YT_ITEM_FIELD_PROJECT, item).expect("Unable to parse project");
        let project_short = Self::field_to_string(Self::YT_ITEM_FIELD_PROJECTSHORT, item).expect("Unable to parse short project name");
        let assignee_login = "".into(); //Self::field_to_string(Self::YT_ITEM_FIELD_ASSIGNEELOGIN, item).expect("Unable to parse assignee");
        let severity = Self::field_to_string(Self::YT_ITEM_FIELD_SEVERITY, item).expect("Unable to parse severity");
        YtItem { 
                id,
                created,
                updated,
                summary,
                reporter_login,
                project,
                project_short,
                assignee: assignee_login,
                severity
        }
    }
}
impl fmt::Display for YtItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_short: &str = self.severity.split_once(" ").map_or(&self.severity, |(l,r)| l);    //.to_string()
        write!(f, "#{: >9}({}) {} c/u: {}/{} by: {:>12}, @{:>12}: {}", self.id, self.project_short, severity_short, self.created, self.updated, self.reporter_login, self.assignee, self.summary)
    }
}

impl fmt::Display for YtItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_string = self
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", as_string)
    }
}

impl DerefMut for YtItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Deref for YtItems {
    type Target = Vec<YtItem>;
    fn deref(& self) -> &Self::Target {
        &self.0
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
    YtItems(items)
}