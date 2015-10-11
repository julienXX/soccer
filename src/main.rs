extern crate hyper;
extern crate rustc_serialize;

use std::io::Read;
use std::env;

use hyper::Client;
use hyper::header::Connection;

use rustc_serialize::json::{Json};

const API_URL: &'static str = "http://api.football-data.org/alpha/soccerseasons/";

fn main() {
    match env::args().nth(1) {
        Some(league) => {
            let (name, ranking) = get_ranking(league);
            println!("{}\n\n{}", name, ranking);
        }
        None => {
            println!("Usage: soccer <league>");
            return;
        }
    };
}

fn get_ranking(league: String) -> (String, String) {
    let url = API_URL.to_string() + &league.to_string() + "/leagueTable";
    let client = Client::new();

    let mut res = client.get(&url)
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let json_body = Json::from_str(&body).unwrap();
    let json_object = json_body.as_object().unwrap();
    let caption = json_object.get("leagueCaption").unwrap();
    let ranking = json_object.get("standing").unwrap();

    (caption.to_string(), ranking.to_string())
}
