use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{prelude::*, stdin};

mod defaults {
    pub(super) const fn cooldown() -> usize {7}
    pub(super) const fn timeout() -> usize {10}
    pub(super) const fn r#true() -> bool {true}
    pub(super) fn database_file() -> String {String::from("giveaways.json")}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeiliSearchConfig {
    pub host: String,
    pub index: String,
    pub key: String,
    #[serde(default = "defaults::r#true")]
    pub init_on_launch: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BackupConfig {
    pub interval: usize,
    pub folder: String,
    pub max: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "defaults::cooldown")]
    pub cooldown: usize,
    #[serde(default)]
    pub update: usize,
    #[serde(default = "defaults::timeout")]
    pub timeout: usize,
    #[serde(default)]
    pub blame_useless_pages: bool,
    #[serde(default = "defaults::database_file")]
    pub database_file: String,
    pub backups: Option<BackupConfig>,
    pub meilisearch: Option<MeiliSearchConfig>,
}

pub fn read_config(path: &str) -> Config {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Failed to open the configuration file: {:?}", e);
        }
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config: Config = match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            panic!("Your configuration file is not valid: {}\nYou may want to use the `configurate` command to generate a configuration file.", e);
        }
    };
    config
}

pub fn configurate() {
    fn ask(question: &str, recommended: bool) -> bool {
        loop {
            println!("{} ({})", question, if recommended {"Y/n"} else {"N/y"});
            let mut answer = String::new();
            stdin().read_line(&mut answer).unwrap();
            match answer.trim().to_lowercase().as_str() {
                "y" | "yes" => return true,
                "n" | "no" => return false,
                answer => {
                    println!("The answer {:?} is not recognized. Please answer with YES or NO.", answer);
                    continue;
                },
            }
        }
    }

    fn input(question: &str) -> String {
        println!("{}", question);
        let mut answer = String::new();
        stdin().read_line(&mut answer).unwrap();
        answer.trim().to_string()
    }

    fn input_usize(question: &str) -> usize {
        loop {
            println!("{}", question);
            let mut answer = String::new();
            stdin().read_line(&mut answer).unwrap();
            match answer.trim().parse() {
                Ok(n) => return n,
                Err(_e) => {
                    println!("Please enter a number.");
                    continue;
                }
            }
        }
    }

    println!("Welcome! Let's generate your configuration file.\n");

    let database_file = input("In which file do you want to save the data (json)?");
    println!("Great! Data will be saved in {:?}.", database_file);
    println!();

    let meilisearch = if ask("Do you want to use a MeiliSearch database?", true) {
        let host = input("Which MeiliSearch server do you want to use?");
        let key = input("What is the key of this server (requires write permission)?");
        let index = input("Which index name do you want to use?");
        let init_on_launch = ask("Do you want to init the index each time you restart the crawler?", true);
        println!();
        Some(MeiliSearchConfig {
            host,
            index,
            key,
            init_on_launch,
        })
    } else {
        None
    };

    let backups = if ask("Do you want to store backups?", true) {
        let folder = input("In which folder?");
        let interval = input_usize("How often do you want to make backups? (in hours)");
        let max = input_usize("What is the maximum number of backups you want to store?");
        println!();
        Some(BackupConfig {
            folder,
            interval,
            max
        })
    } else {
        None
    };

    let (timeout, cooldown) = if ask("Do you want to use custom values for timeout and cooldown?", false) {
        let timeout = input_usize("Enter the maximum duration of an HTTP request in seconds.");
        let cooldown = input_usize("Enter the time to wait between two HTTP requests to the same domain in seconds.");
        (timeout, cooldown)
    } else {
        (10, 7)
    };

    let update = input_usize("How many giveaways to you want to update per hour? (can be 0)");
    let blame_useless_pages = ask("Do you want the crawler to report useless pages?", false);
    println!();

    let config = Config {
        timeout,
        cooldown,
        update,
        blame_useless_pages,
        database_file,
        backups,
        meilisearch,
    };

    let mut file = File::create("config.toml").expect("Unable to open config file");
    let data = toml::to_string(&config).expect("Unable to serialize config");
    file.write_all(data.as_bytes()).expect("Unable to write data to the config file");

    println!("SUCCESS: All settings are set and the crawler is ready!");
}