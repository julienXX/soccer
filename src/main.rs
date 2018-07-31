#![deny(warnings)]
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use hyper::{Body, Client, Request};
use hyper::header::HeaderValue;
use hyper::rt::{self, Future, Stream};

extern crate chrono;
use chrono::prelude::*;

const API_URL: &'static str = "http://api.football-data.org/v2/competitions/";

#[derive(Deserialize, Debug)]
struct CompetitionsRoot {
    competitions: Vec<Competition>,
}

#[derive(Deserialize, Debug)]
struct Competition {
    id: u16,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Team {
    id: u16,
    name: String,
}

#[derive(Deserialize, Debug)]
struct StandingsRoot {
    standings: Vec<StandingTable>,
}

#[derive(Deserialize, Debug)]
struct StandingTable {
    table: Vec<Standing>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Standing {
    position: u8,
    team: Team,
    playedGames: u8,
    won: u8,
    draw: u8,
    lost: u8,
    points: u8,
    goalsFor: u8,
    goalsAgainst: u8,
    goalDifference: i8
}

fn main() {
    match env::var("API_KEY") {
        Ok(_key) => {
            create_gophermap().unwrap();
            println!("Done.")
        }
        Err(e) => println!("Couldn't read API_KEY ({})", e),
    };

}

fn create_gophermap() -> std::io::Result<()> {
    let mut f = File::create("gophermap")?;
    let gophermap = competitions_to_gophermap();
    f.write_all(&gophermap.as_bytes())?;
    Ok(())
}

fn competitions() -> Vec<Competition> {
    vec![ Competition { id: 2016, name: "Championship".to_string() },
          Competition { id: 2021, name: "Premier League".to_string() },
          Competition { id: 2015, name: "Ligue 1".to_string() },
          Competition { id: 2002, name: "Bundesliga".to_string() },
          Competition { id: 2019, name: "Serie A".to_string() },
          Competition { id: 2003, name: "Eredivisie".to_string() },
          Competition { id: 2017, name: "Primeira Liga".to_string() },
    ]
}

fn competitions_to_gophermap() -> String {
    let mut gophermap = String::new();
    gophermap.push_str(&main_title());
    for competition in competitions() {
        println!("Building competition: {}", competition.name);

        let competition_line = format!("0{}\t{}.txt\n", competition.name, competition.id);
        gophermap.push_str(&competition_line);

        build_standings_for(competition);
    }
    gophermap.push_str(&termination_line());
    gophermap
}

fn build_standings_for(competition: Competition) {
    let url = format!("{}{}/standings", API_URL, &competition.id).parse().unwrap();
    let fut = fetch_standings(url)
        .map(|standings_root| {
            let standings = extract_standings(standings_root.standings);
            build_standings_page(competition, standings);
        })
        .map_err(|e| {
            match e {
                FetchError::Http(e) => eprintln!("http error: {}", e),
                FetchError::Json(e) => eprintln!("json parsing error: {}", e),
            }
        });

    rt::run(fut);
}

fn build_standings_page(competition: Competition, standings: Vec<Standing>) -> () {
    let mut f = File::create(format!("{}.txt", competition.id)).unwrap();
    let table = create_table(standings, competition);
    f.write_all(&table.as_bytes()).expect("could not write file");
}

fn fetch_standings(url: hyper::Uri) -> impl Future<Item=StandingsRoot, Error=FetchError> {
    let api_key = env::var("API_KEY").unwrap();
    let header_value = HeaderValue::from_str(&api_key);
    let req = Request::builder()
        .uri(url)
        .header("X-Auth-Token", header_value.unwrap())
        .body(Body::empty())
        .unwrap();

    Client::new()
        .request(req)
        .and_then(|res| {
            res.into_body().concat2()
        })
        .from_err::<FetchError>()
        .and_then(|body| {
            let body_string = std::str::from_utf8(&body).unwrap();
            println!("{:?}", body_string);
            let standings: StandingsRoot = serde_json::from_str(&body_string)?;
            Ok(standings)
        })
        .from_err()
}

fn extract_standings(standing_tables: Vec<StandingTable>) -> Vec<Standing> {
    standing_tables.into_iter().nth(1).unwrap().table
}

fn create_table(standings: Vec<Standing>, competition: Competition) -> String {
    let mut t = String::new();
    t.push_str(&format!("{}\n---\n\n", competition.name));
    t.push_str(&format!("{0: <3} | {1: <26} | {2: <2} | {3: <3} | {4: <1} | {5: <1} | {6: <1} | {7: <1} | {8: <2} | {9: <2}\n",
                        "Pos", "Team", "GP", "PTS", "W", "D", "L", "G", "GA", "GD"));

    for standing in standings {
        t.push_str(&format!("{0: <3} | {1: <26} | {2: <2} | {3: <3} | {4: <1} | {5: <1} | {6: <1} | {7: <1} | {8: <2} | {9: <2}\n",
                            standing.position,
                            standing.team.name,
                            standing.playedGames,
                            standing.points,
                            standing.won,
                            standing.draw,
                            standing.lost,
                            standing.goalsFor,
                            standing.goalsAgainst,
                            standing.goalDifference))
    }
    t.push_str(&termination_line());
    t
}

fn main_title() -> String {
    let utc = Utc::now().format("%a %b %e %T %Y").to_string();
    format!("

   |             |             |
   |___          |          ___|
   |_  |         |         |  _|
  .| | |.       ,|.       .| | |.
  || | | )     ( | )     ( | | ||
  '|_| |'       `|'       `| |_|'
   |___|         |         |___|
   |             |             |
   |_____________|_____________|

See how your team is doing with some nice soccer standings.
Sync happens every 1 hour or so.

Last updated {}

", utc)
}

fn termination_line() -> String {
    "\r\n.".to_owned()
}

enum FetchError {
    Http(hyper::Error),
    Json(serde_json::Error),
}

impl From<hyper::Error> for FetchError {
    fn from(err: hyper::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> FetchError {
        FetchError::Json(err)
    }
}
