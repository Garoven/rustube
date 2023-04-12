use std::collections::HashMap;
use regex::Regex;
use serde_json::Value;

use crate::{Video, Id, Result};

pub async fn get_by_name(name: &str) -> Result<Video> {
    let html = get_html(&name).await?;
    let json = parse_for_js(&html);
    let id_raw = match Id::from_string(get_id(&json)?) {
        Ok(id) => id,
        Err(_) => return Err(crate::Error::BadIdFormat)
    };
    Ok(Video::from_id(id_raw).await?)
}

fn get_id(json: &str) -> Result<String> {
    let obj: Value = serde_json::from_str(json).unwrap();
    let list = match obj["contents"]["twoColumnSearchResultsRenderer"]["primaryContents"]["sectionListRenderer"]["contents"].as_array() {
        Some(l) => l,
        None => return Err(crate::Error::Internal("Error parsing html"))
    };
    for ele in list {
        if let Some(ele) = ele["itemSectionRenderer"]["contents"].as_array()
        {
            for e in ele {
                if let Some(x) = e["videoRenderer"]["navigationEndpoint"]["watchEndpoint"]["videoId"].as_str() {
                    return Ok(x.to_string())
                }
            }
        }
    }
    Err(crate::Error::BadIdFormat)
}

// gets the html from the given name
async fn get_html(name: &str) -> Result<String> {
    let link = format!("https://www.youtube.com/results?search_query={}", name.replace(" ", "+"));
    let val = reqwest::get(link).await?
        .text().await?;
    Ok(val)
}

// parses the html looking for the json object
fn parse_for_js(html: &str) -> String {
    // regex pattern to find the "ytInitialData = " string that signifies the json obj
    let pattern = r#"ytInitialData\s*=\s*"#;
    // unwrap the pattern
    let re = Regex::new(pattern).unwrap();
    // finds the only instance of this, if not found in the html, a panic occurs
    let result = re.find(html).expect("Pattern not found!");
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
