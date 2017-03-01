extern crate clap;
extern crate hyper;
extern crate queryst;
extern crate regex;

use clap::{App, Arg};
use hyper::client::Client;
use queryst::parse;
use regex::Regex;
use std::io::Read;

fn main() {
    let parseargs = App::new("videofetch")
        .about("CLI for retrieving direct video links from Youtube")
        .version("0.1")
        .arg(Arg::with_name("url")
            .long("url")
            .short("u")
            .takes_value(true)
            .required(true)
            .help("The url of the video to be retrieved"))
        .get_matches();

    let urlregex = Regex::new(r"watch\\?\?v\\?=([a-zA-Z0-9_-]{11})").unwrap();
    let urlin = parseargs.value_of("url").unwrap();

    let result = match urlregex.captures(&urlin) {
        Some(s) => s.get(1).unwrap().as_str(),
        None => return,
    };
  
    let api_url = "http://youtube.com/get_video_info?video_id=".to_string();
    let request_url = api_url + result;

    let httpclient = Client::new();
    let getrequest = httpclient.get(&request_url);
   
    match getrequest.send().map(|i| i) {

        Ok(s) => if s.status == hyper::Ok {
            parse_response(s)
        },

        Err(..) => println!("No response"),
    }
}

fn parse_response(mut response: hyper::client::Response) {
        let mut buffer = String::new();

        if response.read_to_string(&mut buffer).ok().is_some() {
            let parsed = parse(&buffer);

            if parsed.is_ok() {
                match parsed.unwrap().as_object().unwrap().get("url_encoded_fmt_stream_map").map(|fmt| fmt) {

                        Some(s) => get_fmt_map(s.to_string()),

                        None => println!("Error getting url_encoded_fmt_stream_map"),
                }
            }

            else {
                println!("Parsing failed");
            }
        }
}

fn get_fmt_map(videoinfo: String) {
        let streammap = parse(&videoinfo);

        if streammap.is_ok() {
            let streamm = streammap.unwrap();
            let urlset = streamm.as_object().unwrap().get("url");

            match urlset.map(|o| o) {

                Some(s) => {
                    let urlarr = s.as_array().unwrap();
                    let mut urlstring = urlarr[0].to_string();
                    urlstring.remove(0);
                    urlstring.pop();
                    println!("{}", &urlstring);
                },

                None => println!("No url found"),
            }
        }
}
