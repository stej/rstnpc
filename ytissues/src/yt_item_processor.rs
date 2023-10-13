use std::fmt;

struct YtItem {
    id: String,
    created: u64,   // datetime v budoucnu
}

impl YtItem {
    const YT_ITEM_FIELD_IDREADABLE: &str = "idReadable";
    const YT_ITEM_FIELD_CREATED: &str = "created";

    fn parse(item: &serde_json::Value) -> YtItem {
        let id = item[Self::YT_ITEM_FIELD_IDREADABLE].as_str().unwrap();
        let created = item[Self::YT_ITEM_FIELD_CREATED].as_u64().unwrap();
        YtItem { id: String::from(id), created: created }
    }
}
impl fmt::Display for YtItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id: {: >9}, created: {}", self.id, self.created)
    }
}

fn item_as_array(item: &serde_json::Value) -> &Vec<serde_json::Value> {
    let Some(arr) = item.as_array() else {
        println!("item: {:?}", item);
        panic!("item is not an array!");
    };
    arr
}

pub fn describe_yt_items(item: &serde_json::Value) {
    //println!("{}", serde_json::to_string_pretty(&item).unwrap()); 

    for item in item_as_array(item) {
        let item = YtItem::parse(item);
        println!("{}", item);
    }
}