use clap::*;
mod stats;
mod crawler;
mod config;
mod google;
mod gleam;
mod meilisearch;
mod database;
mod backup;
use config::*;
use stats::*;
use crawler::launch;
use meilisearch::init_meilisearch;
use config::configurate;
use backup::backup;

#[tokio::main]
async fn main() {
    let matches = clap_app!(myapp =>
        (version: "4.0")
        (author: "Mubelotix <mubelotix@gmail.com>")
        (about: "Crawl the web to find gleam.io links")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
        (@subcommand stats =>
            (about: "Display stats about the database")
        )
        (@subcommand init_meilisearch =>
            (about: "Init the meilisearch index")
        )
        (@subcommand configurate =>
            (about: "Build a configuration file")
        )
        (@subcommand backup =>
            (about: "Backup the database")
        )
        (@subcommand launch =>
            (about: "Launch the bot")
            (@arg fast: -f --f "Do not load gleam.io pages and do not save them")
        )
    ).get_matches();

    let config = || {
        read_config(matches.value_of("CONFIG").unwrap_or("config.toml"))
    };

    match matches.subcommand() {
        ("stats", Some(_args)) => stats(config()),
        ("init_meilisearch", Some(_args)) => init_meilisearch(&config()).await,
        ("configurate", Some(_args)) => configurate(),
        ("backup", Some(_args)) => backup(&config().database_file, &config().backups.expect("Please configurate backups")),
        ("launch", Some(args)) => {
            let fast: bool = args.value_of("fast").unwrap_or("false").parse().unwrap();
            launch(config(), fast).await;
        },
        (name, Some(_args)) => {
            println!("Unknown subcommand: {:?}", name);
        }
        (_name, None) => {
            println!("No subcommand, no action taken");
        }
    }
}
