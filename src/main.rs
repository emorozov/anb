use std::error::Error;

use base64::encode;
use clap::{App, Arg};
use console::style;
use directories::BaseDirs;
use regex::RegexBuilder;
use reqwest::header;
use serde_json::Value;
use subprocess::{Exec, Redirection};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Annotate branches")
        .version("0.1.0")
        .author("Evgenii Morozov <jmv@emorozov.net>")
        .about("Displays git branch list with corresponding JIRA task names")
        .arg(
            Arg::with_name("status")
                .short("s")
                .long("status")
                .takes_value(true)
                .help("Display only tasks with the specified status"),
        )
        .get_matches();

    let mut settings = config::Config::default();
    if let Some(user_dirs) = BaseDirs::new() {
        let config = format!("{}/anb.toml", user_dirs.preference_dir().to_str().unwrap());
        settings
            .merge(config::File::with_name(&config).required(false))
            .ok();
    }

    settings
        .merge(config::Environment::with_prefix("ANB"))
        .unwrap();

    let branches = Exec::shell("git branch")
        .stdout(Redirection::Pipe)
        .capture()?
        .stdout_str();
    let prefix = settings.get::<String>("prefix")?;
    let task_name_re = RegexBuilder::new(&format!(r"{}-\d+", prefix))
        .case_insensitive(true)
        .build()
        .unwrap();
    let captures = task_name_re.captures_iter(&branches).collect::<Vec<_>>();
    let branches: Vec<&str> = captures
        .iter()
        .map(|c| c.get(0).unwrap().as_str())
        .collect();

    let mut headers = header::HeaderMap::new();
    let username = settings.get::<String>("username")?;
    let password = settings.get::<String>("password")?;
    let auth = format!("Basic {}", encode(format!("{}:{}", username, password)));
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&auth).unwrap(),
    );
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let server = settings.get::<String>("server")?;
    let required_status = matches.value_of("status");
    for branch in branches.iter() {
        let data = client
            .get(&format!(
                "https://{}/rest/api/latest/issue/{}",
                server, branch
            ))
            .send()
            .unwrap()
            .text()?;
        let task: Value = serde_json::from_str(&data)?;
        let status: &str = &task["fields"]["status"]["name"].as_str().unwrap();
        match required_status {
            Some(required_status) if required_status.to_lowercase() == status.to_lowercase() => {
                println!(
                    "{:20} {} ({})",
                    style(branch).green(),
                    task["fields"]["summary"].as_str().unwrap(),
                    style(status).cyan()
                );
            }
            Some(_) => {}
            None => {
                println!(
                    "{:20} {} ({})",
                    style(branch).green(),
                    task["fields"]["summary"].as_str().unwrap(),
                    style(status).cyan()
                );
            }
        }
    }
    Ok(())
}
