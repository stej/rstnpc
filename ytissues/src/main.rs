struct ConnectionInfo {
    token: String,
    base_url: String,
}

impl ConnectionInfo {
    fn get_issues_url(&self, top: i32, query: &str, fields: &str) -> String {
        format!("{}/api/issues?query={}&%24top={}&fields={}", self.base_url, query, top, fields)
    }

    fn read() -> ConnectionInfo {
        let connection_info = std::fs::read_to_string("token.txt").expect("Failed to read token.txt");
        let mut lines = connection_info.lines();
        let token = lines.next().unwrap();
        let baseurl = lines.next().unwrap();
        ConnectionInfo { token: String::from(token), base_url: String::from(baseurl) }
    }
}

async fn query_youtrack(connection_info: &ConnectionInfo, query: &str, max_count: i32) -> Result<String, reqwest::Error> {
    let fields = {
        let fields_simple = "idReadable,created,updated,resolved,numberInProject,summary,description,usesMarkdown,wikifiedDescription,isDraft,visibility,votes,commentsCount,tags,externalIssue";
        let fields_inside = "links(direction,issues(idReadable,summary),linkType(name,sourceToTarget,directed)),subtasks(direction,issues(idReadable,summary)),parent(idReadable,summary),\
                  \nwatchers(user(login)),updater(login,fullName),reporter(login,fullName),draftOwner(login,fullName),voters(original(login,fullName)),\
                  \ncustomFields(type,name,value(name,login,bundle,id)),\
                  \nattachments(name,author(login),created),project(name,shortName,description,leader(login))";
        //let fields_comments = "comments(text,created,author(login))";

        // sample multiline string without indentation

        format!("{},{}", fields_simple, fields_inside)
    };
    let url = connection_info.get_issues_url(max_count, query, &fields);
    let response = reqwest::Client::new()
         .get(&url)
         .header("Authorization", format!("Bearer {}", connection_info.token))
         .send()
         .await;
    match response {
        Ok(response) => Ok(response.text().await.unwrap()),
        Err(error) => Err(error),
    }
}

fn prepare_query(query: &str) -> String {
    urlencoding::encode(query).to_string()
}

#[tokio::main]
async fn main() {
    let connection_info = ConnectionInfo::read();

    let query = prepare_query("#Bug #Resolved order by: updated desc");

    match query_youtrack(&connection_info, &query, 1).await {
        Ok(response) => {
            // response is json, pretty print the response
            let json: serde_json::Value = serde_json::from_str(&response).unwrap();
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        },
        Err(error) => println!("Error: {:?}", error),
    }
}
