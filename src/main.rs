use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::io::{stdout, IsTerminal};

const DOCKER_SPECIFIED_SUBCOMMAND: &str = "docker-cli-plugin-metadata";
const DOCKERHUB_API_PREFIX: &str = "https://registry.hub.docker.com/v2/repositories";
const DOCKER_CLI_META_DATA: MetaData = MetaData {
    SchemaVersion: "0.1.0",
    Vendor: env!("CARGO_PKG_AUTHORS"),
    Version: env!("CARGO_PKG_VERSION"),
    ShortDescription: env!("CARGO_PKG_DESCRIPTION"),
};

fn main() -> std::result::Result<(), ()> {
    // Parse cmdline args
    let mut args = Args::parse();

    // Change image if this program is used by docker plugin (ex: docker tags "IMAGE_NAME")
    if args.image == "tags" {
        if args.image_sub.is_some() {
            args.image = args.image_sub.unwrap();
        } else {
            Args::parse_from(vec![""]);
            return Err(());
        }
    }

    // Print metadata if "docker-cli-plugin-metadata" is specified
    if args.image == DOCKER_SPECIFIED_SUBCOMMAND {
        let metadata_json = serde_json::to_string(&DOCKER_CLI_META_DATA).unwrap();
        println!("{}", metadata_json);
        return Ok(());
    }

    // Format URI with specified image nmae
    let mut uri = match args.image.chars().filter(|c| *c == '/').count() {
        0 => format!(
            "{}/library/{}/tags?page_size=10000",
            DOCKERHUB_API_PREFIX, args.image
        ),
        1 => format!(
            "{}/{}/tags?page_size=10000",
            DOCKERHUB_API_PREFIX, args.image
        ),
        2 => {
            eprintln!("Registry other than DockerHub not supported yet");
            return Err(());
        }
        _ => {
            eprintln!("{} is invalid image name", args.image);
            return Err(());
        }
    };

    let is_terminal = stdout().is_terminal();
    let mut progress: Option<ProgressBar> = None;
    let mut tags: Vec<Tag> = vec![];
    loop {
        // Get tag data from specified URI
        let res = reqwest::blocking::get(uri).unwrap();

        // Exit if response is not ok
        if res.status() != 200 {
            eprintln!("Failed to fetch tags");
            eprintln!("{}", res.text().unwrap());
            return Err(());
        }

        // Deserialize response into Res struct
        let body = res.text().unwrap();
        let mut res: Res = serde_json::from_str(&body).unwrap();

        // Show progress bar if it is not created and stdin is terminal
        if progress.is_none() && is_terminal {
            let pb = ProgressBar::new(res.count).with_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}",
                    )
                    .unwrap(),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(10));
            progress = Some(pb);
        }

        // Increment progress with the number of fetched tags
        if let Some(pb) = &progress {
            pb.inc(res.results.len().try_into().unwrap());
        }

        // Store fetched tags
        tags.append(&mut res.results);

        // Break loop if all tags are fetched
        if res.next.is_none() {
            break;
        }

        // Continue next loop with new URI
        uri = res.next.unwrap()
    }

    // Clear progress bar
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    // Print all tags
    for tag in tags {
        if args.update {
            let last_updated: Vec<&str> = tag.last_updated.split('T').collect();
            println!("{} ({})", tag.name, last_updated[0]);
        } else {
            println!("{}", tag.name);
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, author)]
struct Args {
    /// Image name (ex: alpine, library/ubuntu)
    image: String,

    #[arg(hide = true)]
    image_sub: Option<String>,

    #[arg(short = 'u', long = "print-updated")]
    /// Print last updated time of image
    update: bool,
}

#[derive(Deserialize, Debug)]
struct Res {
    count: u64,
    next: Option<String>,
    results: Vec<Tag>,
}

#[derive(Deserialize, Debug)]
struct Tag {
    last_updated: String,
    name: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct MetaData {
    SchemaVersion: &'static str,
    Vendor: &'static str,
    Version: &'static str,
    ShortDescription: &'static str,
}
