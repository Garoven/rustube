use std::collections::HashMap;
use regex::Regex;
use serde_json::Value;

#[derive(Debug)]
pub struct Playlist {
    pub links: Vec<String>,
}

pub async fn get_playlist(link: &str) -> Playlist {
    let html: String = get_html(link).await;
    let json: String = parse_for_js(html);
    let video_vec: Vec<String> = json_to_vec_videos(&json);
    let final_exp: Playlist = Playlist {
        links: video_vec,
    };
    final_exp
}

// gets the html from the given link
async fn get_html(link: &str) -> String {
    let val = reqwest::get(link).await
        .unwrap().text().await.unwrap();
    val
}

// turns the json into a vec of the videos
fn json_to_vec_videos(to_json: &String) -> Vec<String> {
    // new vector that will contain the links
    let mut return_vals: Vec<String> = Vec::new();
    // turn the json(string) into json(serde_json::Value)
    let obj: Value = serde_json::from_str(to_json.as_str()).expect("The json was invalid");
    // this is an array of the playlist contents
    let vals = &obj["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]["itemSectionRenderer"]["contents"][0]["playlistVideoListRenderer"]["contents"];
    let bruh = vals.as_array().unwrap();
    for video in bruh {
        let id = match &video["playlistVideoRenderer"]["videoId"].as_str() {
            Some(id) => id.to_owned(),
            None => break
        };
        let string = format!["https://youtube.com/watch?v={}", id];

        return_vals.push(string)
    }
    return return_vals

}

// parses the html looking for the json object
fn parse_for_js(html: String) -> String {
    // regex pattern to find the "ytInitialData = " string that signifies the json obj
    let pattern = r#"ytInitialData\s*=\s*"#;
    // unwrap the pattern
    let re = Regex::new(pattern).unwrap();
    // finds the only instance of this, if not found in the html, a panic occurs
    let result = re.find(&html).expect("Pattern not found!");
    // get the end of the found pattern. This will give the char position in the html where
    // the obj begins
    let start_index = result.end();
    // now we call the function that will loop over that html (form that start_index) and get the obj
    find_object_from_startpoint(&html, start_index)
}

// main loop that will find the exactly bounds of the json
fn find_object_from_startpoint(old_html: &str, starting: usize) -> String {
    // defines the new html as from the starting point (beginning of json)
 let html = &old_html[starting..];
    // defines html as a vector of chars, easier to operate with
    let html: Vec<char> = html.chars().collect();
    // starting index. Skipping 0 because first letter must be an open brace, so it is placed in stack
    let mut i: usize = 1;
    // making sure that first char is either a [ or { (seems to always be a '{' )
    if html[0] != '{' && html[0] != '[' {
        // panics if it isnt either
        panic!["Invalid start point!"]
    }
    // first char, will be added to the stack
    let first_temp: char = html[0];
    // create the stack (adding the first char in there)
    let mut stack: Vec<char> = vec![first_temp];
    // context closes used during iteration
    let context_closers: HashMap<char, char> = HashMap::from([
        ('{', '}'),
        ('[', ']'),
        ('\"', '\"')
    ]);
    while i < html.len() {
        // if that stack length == 0 that means we have reached the end of the object because
        // there are no more context closers (aka keeping tack of how many braces there are)
        if stack.len() == 0 {
            break
        }
        // updates the current char
        let curr_char: char = html[i];
        // curr_context = the last item in the stack
        let curr_context = stack[stack.len() -1];
        // first if statement is a guard against a panic! (if curr_char == context_closers[curr_context]
        if context_closers.contains_key(&curr_context) {
            // so if it is contained in it, and curr_char == it, pop one off the stack
            if curr_char == context_closers[&curr_context] {
                stack.pop().unwrap();
                i += 1;
                continue
            };
        }

        // "Strings require special context handling because they can contain context openers *and* closers"
        if curr_context == '\"' {
            if curr_char == '\\' {
                i += 2;
                continue
            }
        }
        else {
            // "Non-string contexts are when we need to look for context openers."
            if context_closers.contains_key(&curr_char) {
                stack.push(curr_char)
            }
        }
        // add one after each iteration :)
        i += 1
    }
    // define the json, and return it as a string !
    let full_obj: &[char] = &html[..i];
    let _ret_obj: String = full_obj.into_iter().collect();
    return full_obj.into_iter().collect()
}
