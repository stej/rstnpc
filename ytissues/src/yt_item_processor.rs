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
    assignees: Vec<String>,
    severity: String,
}

pub struct YtItems(Vec<YtItem>);

#[allow(dead_code)]
impl YtItem {
    const YT_ITEM_FIELD_IDREADABLE: &str = "idReadable";
    const YT_ITEM_FIELD_CREATED: &str = "created";
    const YT_ITEM_FIELD_UPDATED: &str = "updated";
    const YT_ITEM_FIELD_SUMMARY: &str = "summary";
    const YT_ITEM_FIELD_REPORTERLOGIN: &str = "reporter/login";
    const YT_ITEM_FIELD_PROJECT: &str = "project/name";
    const YT_ITEM_FIELD_PROJECTSHORT: &str = "project/shortName";
    const YT_ITEM_FIELD_ASSIGNEELOGIN: &str = "customFields/?name=Assignee/value/*login";
    const YT_ITEM_FIELD_SEVERITY: &str = "customFields/?name=Defect Severity/value/name";

    fn get_nested<'l>(item: &'l serde_json::Value, field_path: &str) -> &'l serde_json::Value {
        fn find_in_array<'b>(item: &'b serde_json::Value, condition: &str) -> Option<&'b serde_json::Value> {
            //println!("Array: {:?}", item);
            let (field_name, field_value) = condition.split_once("=").unwrap();
            let arr = item_as_json_array(item);
            let item = arr
                .iter()
                .find(|item| item[field_name] == field_value)?;
            //println!("Found: {:?}", item);
            Some(item)
        }
        match field_path.split_once("/") {
            None => &item[field_path],
            Some((parent, rest)) => { 
                if !parent.is_empty() && matches!(parent.chars().nth(0), Some('?')) { 
                    let item_in_array = find_in_array(item, &parent[1..]).expect("Unable to find item in array");
                    Self::get_nested(item_in_array, rest)
                } else { 
                    Self::get_nested(&item[parent], rest)
                }
            }
        }
    }

    fn field_to_datetime(item: &serde_json::Value, field_name: &str) -> Result<UtcDateTime, Box<dyn Error>> {
        let item = Self::get_nested(item, field_name);
        let ms = item
            .as_u64()
            .ok_or(format!("Unable to parse date. Item: {field_name}"))?;
        Ok(DateTime::UNIX_EPOCH + std::time::Duration::from_millis(ms))
    }
    fn field_to_string(item: &serde_json::Value, field_name: &str) -> Result<String, Box<dyn Error>> {
        let item = Self::get_nested(item, field_name);
        let s = item
                        .as_str()
                        .ok_or(format!("Unable to parse string. Item: {field_name}"))?;
        Ok(s.into())
    }

    fn parse(item: &serde_json::Value) -> YtItem {
        let id = Self::field_to_string(item, Self::YT_ITEM_FIELD_IDREADABLE).expect("Unable to parse id");
        let summary = Self::field_to_string(item, Self::YT_ITEM_FIELD_SUMMARY).expect("Unable to parse summary");
        let created = Self::field_to_datetime(item, Self::YT_ITEM_FIELD_CREATED).expect("Unable to parse date");
        let updated = Self::field_to_datetime(item, Self::YT_ITEM_FIELD_UPDATED).expect("Unable to parse date");
        let reporter_login = Self::field_to_string(item, Self::YT_ITEM_FIELD_REPORTERLOGIN).expect("Unable to parse reporter");
        let project = Self::field_to_string(item, Self::YT_ITEM_FIELD_PROJECT).expect("Unable to parse project");
        let project_short = Self::field_to_string(item, Self::YT_ITEM_FIELD_PROJECTSHORT).expect("Unable to parse short project name");
        //let assignees = Self::fields_to_strings(item, Self::YT_ITEM_FIELD_ASSIGNEELOGIN).expect("Unable to parse assignees");
        let assignees = get_nested_vec(item, Self::YT_ITEM_FIELD_ASSIGNEELOGIN)
                                        .iter()
                                        .map(|o| o.as_str().expect("Unable to parse assignees").into())
                                        .collect();
        //let assignees = vec![];
        let severity = Self::field_to_string(item, Self::YT_ITEM_FIELD_SEVERITY).expect("Unable to parse severity");
        YtItem { 
                id,
                created,
                updated,
                summary,
                reporter_login,
                project,
                project_short,
                assignees,
                severity
        }
    }
}

// sample: "customFields/?name=Assignee/value/*login"
#[allow(unused_variables, dead_code)]
fn get_nested_vec<'l>(item: &'l serde_json::Value, field_path: &str) -> Vec<&'l serde_json::Value> {
    enum PathSegment {
        Field{ name: String, rest: String },
        EqCondition { name: String, value: String, rest: String },
        ArrayIterator{ name: String, rest: String }, 
        This
    }
    fn categorize_segment(path: &str) -> PathSegment {
        fn string_to_segment_part(str: &str, rest: &str) -> PathSegment {
            match str.chars().nth(0) {
                Some('?') => {
                    let (field_name, field_value) = str.split_once("=").unwrap();
                    PathSegment::EqCondition{name: field_name[1..].into(), value: field_value.into(), rest: rest.into() }
                },
                Some('*') => PathSegment::ArrayIterator{ name: str[1..].into(), rest: rest.into() },
                _ => PathSegment::Field{ name: str.into(), rest: rest.into() }
            }
        }
        if path.is_empty() {
            return PathSegment::This;
        }
        path.split_once("/")
            .map(|(parent, rest)| {
                string_to_segment_part(parent, rest)
            })
            .unwrap_or(string_to_segment_part(path, ""))
    }
    fn find_in_array<'a>(item: &'a serde_json::Value, field_name: &str, field_value: &str) -> Option<&'a serde_json::Value> {
        //println!("Array: {:?}", item);
        //println!("Find: {}={}", field_name, field_value);
        let arr = item_as_json_array(item);
        let item = arr
            .iter()
            .find(|item| item[field_name] == field_value)?;
            //.find(|item| { println!("{} == {}?", item[field_name], field_value); item[field_name] == field_value})?;
        Some(item)
    }
    fn inner<'a, 'b>(item: &'a serde_json::Value, field_path: &str, result: &'b mut Vec<&'a serde_json::Value>) {
        //println!("Item: {:?}", item);
        match categorize_segment(field_path) {
            PathSegment::Field{name, rest} => {
                inner(&item[name], &rest, result);
            },
            PathSegment::EqCondition{name, value, rest } => {
                let item_in_array = find_in_array(item, &name, &value).expect("Unable to find item in array");
                inner(item_in_array, &rest, result);
            },
            PathSegment::ArrayIterator { name, rest } => {
                for obj in item_as_json_array(item).iter() {
                    inner(&obj[&name], &rest, result);
                }
            },
            PathSegment::This => {
                result.push(item);
            }
        }
    }
    let mut result = vec!();
    inner(item, field_path, &mut result);
    result
}

#[allow(unused_variables, dead_code)]
fn field_to_vec<T>(item: &serde_json::Value, field_path: &str, transform: impl Fn(&serde_json::Value) -> T) -> Result<Vec<String>, Box<dyn Error>> {
    let values = get_nested_vec(item, field_path);
    Ok(vec!["".into()])
}

impl YtItems {    
    pub fn parse(item: &serde_json::Value) -> YtItems {
        let items = item_as_json_array(item)
            .iter()
            .map(YtItem::parse)
            .collect();
        YtItems(items)
    }
}

impl fmt::Display for YtItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_short: &str = self.severity.split_once(" ").map_or(&self.severity, |(l,_r)| l);    //.to_string()
        let assignees = self.assignees.join(",");
        write!(f, "#{: >9}({}) {} c/u: {}/{} by: {:>12}, @{:>12}: {}", self.id, self.project_short, severity_short, self.created, self.updated, self.reporter_login, assignees, self.summary)
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

fn item_as_json_array(item: &serde_json::Value) -> &Vec<serde_json::Value> {
    let Some(arr) = item.as_array() else {
        println!("item: {:?}", item);
        panic!("item is not an array!");
    };
    arr
}

// add tests for YtItem::parse
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ISSUE_FILE : &str = r"testing\sampleissue.json";

    fn get_sample_issue() -> serde_json::Value {
        let json_string = std::fs::read_to_string(SAMPLE_ISSUE_FILE).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        json
    }

    #[test]
    fn can_read_sample_file() {
        let json = get_sample_issue();
        assert!(json.is_array());

        let first = &json[0];
        assert!(first["idReadable"].is_string());
        assert_eq!(first["idReadable"].as_str().unwrap(), "ABC-10617");
    }

    #[test]
    fn can_parse_sample() {
        let items = YtItems::parse(&get_sample_issue());
        let first = &items[0];

        assert_eq!(first.id, "ABC-10617");
        assert_eq!(first.summary, "Unable to import data created in 2017 or sooner")
    }

    #[test]
    fn tadaaa() {
        let json = get_sample_issue();
        let vec = get_nested_vec(&json, "*customFields/?name=Assignee/value/*login");

        assert!(vec.len() == 1);
        assert_eq!(vec[0].as_str().unwrap(), "u.ser");
    }

    #[test]
    fn write_all_customfield_simple() {
        let json = get_sample_issue();
        let vec = get_nested_vec(&json, "*customFields/?$type=SimpleIssueCustomField");
        
        assert!(vec.len() > 0);
        for o in vec {
            println!("{}: {}", o["name"], o["value"]);
        }
    }
}

