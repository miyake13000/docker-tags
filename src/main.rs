use clap::Parser;
use futures::{stream, StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::io::{stdout, IsTerminal};
use std::process::exit;
use std::time::Duration;

const DOCKER_SPECIFIED_SUBCOMMAND: &str = "docker-cli-plugin-metadata";
const DOCKERHUB_API_PREFIX: &str = "https://registry.hub.docker.com/v2/repositories";
const DOCKER_CLI_META_DATA: MetaData = MetaData {
    SchemaVersion: "0.1.0",
    Vendor: env!("CARGO_PKG_AUTHORS"),
    Version: env!("CARGO_PKG_VERSION"),
    ShortDescription: env!("CARGO_PKG_DESCRIPTION"),
};
const N_TAGS_PER_FETCH: u64 = 100;
const MAX_CONCURRENT_FETCHES: usize = 8;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {}", error);
        exit(1);
    }
}

async fn run() -> Result<(), String> {
    // Parse cmdline args
    let raw_args: Vec<String> = args().collect();
    let args = if raw_args.get(1).is_some_and(|arg| arg == "tags") {
        Args::parse_from(&raw_args[1..])
    } else {
        Args::parse_from(&raw_args)
    };

    // Print metadata if "docker-cli-plugin-metadata" is specified
    if args.image == DOCKER_SPECIFIED_SUBCOMMAND {
        let metadata_json = serde_json::to_string(&DOCKER_CLI_META_DATA)
            .map_err(|error| format!("Failed to serialize plugin metadata: {error}"))?;
        println!("{}", metadata_json);
        return Ok(());
    }

    // Format URI with specified image nmae
    let uri = match args.image.chars().filter(|c| *c == '/').count() {
        0 => format!("{}/library/{}/tags", DOCKERHUB_API_PREFIX, args.image),
        1 => format!("{}/{}/tags", DOCKERHUB_API_PREFIX, args.image),
        2 => {
            return Err("Registry other than DockerHub not supported yet".into());
        }
        _ => {
            return Err(format!("{} is invalid image name", args.image));
        }
    };

    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))?;

    // Get response from specified URI
    let first_uri = format!("{}?page_size=1", uri);
    let res = fetch_tags(&client, &first_uri).await?;

    // Show progress bar if it is not created and stdin is terminal
    let progress = if stdout().is_terminal() {
        let pb = ProgressBar::new(res.count).with_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")
                .map_err(|error| format!("Failed to configure progress bar: {error}"))?,
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(10));
        Some(pb)
    } else {
        None
    };

    let mut tasks = Vec::new();
    let last_page = res.count.div_ceil(N_TAGS_PER_FETCH);
    for i in 0..last_page {
        let client = &client;
        let uri = &uri;
        let progress = &progress;
        let task = async move {
            // Set own URI
            let uri = format!("{}?page_size={}&page={}", uri, N_TAGS_PER_FETCH, i + 1);

            // Fetch tags with asynchronously
            let res = fetch_tags(client, &uri).await?;

            // Increment progress with the number of fetched tags
            if let Some(pb) = &progress {
                let count = u64::try_from(res.results.len())
                    .map_err(|error| format!("Invalid tag count: {error}"))?;
                pb.inc(count);
            }

            Ok::<_, String>(res.results)
        };

        tasks.push(task);
    }

    // Fetch up to eight pages concurrently while preserving page order
    let pages: Result<Vec<Vec<Tag>>, String> = stream::iter(tasks)
        .buffered(MAX_CONCURRENT_FETCHES)
        .try_collect()
        .await;

    // Clear progress bar
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let tags: Vec<Tag> = pages?.into_iter().flatten().collect();
    let output = format_tags(&tags, &args)?;
    for line in output.lines {
        println!("{line}");
    }
    if output.unverified_tags > 0 {
        eprintln!(
            "warning: {} tags was skipped because their digest is empty",
            output.unverified_tags
        );
    }
    Ok(())
}

#[derive(Debug)]
struct TagOutput {
    lines: Vec<String>,
    unverified_tags: usize,
}

fn format_tags(tags: &[Tag], args: &Args) -> Result<TagOutput, String> {
    let show = args
        .show_tag
        .as_deref()
        .or(args.show_latest.then_some("latest"));
    let filter = args
        .filter_tag
        .as_deref()
        .or(args.filter_latest.then_some("latest"));

    let reference_digest = |name: &str| -> Result<&str, String> {
        let tag = tags
            .iter()
            .find(|tag| tag.name == name)
            .ok_or_else(|| format!("tag \"{name}\" not found"))?;
        tag.digest
            .as_deref()
            .filter(|digest| !digest.trim().is_empty())
            .ok_or_else(|| format!("digest unavailable for tag \"{name}\""))
    };
    let show_digest = show.map(&reference_digest).transpose()?;
    let filter_digest = filter.map(&reference_digest).transpose()?;

    let unverified_tags = if show.is_some() || filter.is_some() {
        tags.iter()
            .filter(|tag| {
                tag.digest
                    .as_deref()
                    .is_none_or(|digest| digest.trim().is_empty())
            })
            .count()
    } else {
        0
    };

    let mut lines = Vec::new();
    for tag in tags {
        if let Some(name) = filter {
            if tag.name == name || tag.digest.as_deref() != filter_digest {
                continue;
            }
        }
        let mut line = tag.name.clone();
        if let Some(name) = show {
            if tag.name != name && tag.digest.as_deref() == show_digest {
                line.push_str(&format!(" ({name})"));
            }
        }
        if args.update {
            let date = tag.last_updated.split('T').next().unwrap_or("");
            line.push_str(&format!(" ({date})"));
        }
        lines.push(line);
    }
    Ok(TagOutput {
        lines,
        unverified_tags,
    })
}

async fn fetch_tags(client: &reqwest::Client, uri: &str) -> Result<Res, String> {
    let response = client
        .get(uri)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch tags from {uri}: {error}"))?;
    let status = response.status();
    if status != reqwest::StatusCode::OK {
        let body = response.text().await.map_err(|error| {
            format!(
                "Failed to fetch tags from {uri}: HTTP {status}; failed to read response: {error}"
            )
        })?;
        return Err(format!(
            "Failed to fetch tags from {uri}: HTTP {status}\n{body}"
        ));
    }
    response
        .json()
        .await
        .map_err(|error| format!("Failed to decode tag response from {uri}: {error}"))
}

#[derive(Parser, Debug)]
#[command(version, author)]
struct Args {
    /// Image name (ex: alpine, library/ubuntu)
    image: String,

    #[arg(short = 'u', long = "print-updated")]
    /// Print last updated time of image
    update: bool,

    /// Annotate tags sharing the specified tag's digest
    #[arg(long, value_name = "TAG", conflicts_with = "show_latest")]
    show_tag: Option<String>,

    /// Annotate tags sharing the latest tag's digest
    #[arg(short = 's', long)]
    show_latest: bool,

    /// List other tags sharing the specified tag's digest
    #[arg(long, value_name = "TAG", conflicts_with = "filter_latest")]
    filter_tag: Option<String>,

    /// List other tags sharing the latest tag's digest
    #[arg(short = 'l', long)]
    filter_latest: bool,
}

#[allow(dead_code)]
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
    digest: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct MetaData {
    SchemaVersion: &'static str,
    Vendor: &'static str,
    Version: &'static str,
    ShortDescription: &'static str,
}
