extern crate hyper;
extern crate rustc_serialize;

use std::io::Read;
use std::env;

use hyper::Client;
use hyper::header::Connection;

use rustc_serialize::json::{self, Json, Decoder, Object};

const API_URL: &'static str = "http://api.football-data.org/alpha/soccerseasons/";

 #[derive(Debug, RustcDecodable)]
pub struct TeamRanking {
    position: u8,
    teamName: String,
    playedGames: u8,
    points: u8,
    goals: u8,
    goalsAgainst: u8,
    goalDifference: i8
}

fn main() {
    match env::args().nth(1) {
        Some(league) => {
            let (name, ranking) = get_ranking(league);
            println!("{}\n\n{:?}", name, ranking);
        }
        None => {
            println!("Usage: soccer <league>");
            return;
        }
    };
}

fn get_ranking(league: String) -> (String, Vec<TeamRanking>) {
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
    let rankings = json_object.get("standing").unwrap();

    let mut team_rankings = Vec::new();
    for ranking in rankings.as_array().unwrap() {
        let team_ranking: TeamRanking = json::decode(&ranking.to_string()).unwrap();
        team_rankings.push(team_ranking);
    }

    (caption.to_string(), team_rankings)
}
