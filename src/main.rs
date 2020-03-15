use std::io::Read;
use std::fs::{File,DirEntry};
use std::fs::{read_dir,create_dir_all,write,copy};
use std::collections::HashSet;
use std::path::{Path,PathBuf};

use clap::{App,Arg};
use handlebars::Handlebars;
use serde_json;

fn main() {
    let matches = App::new("marksman")
        .version("0.1.0")
        .author("Jamen Marz <me@jamen.dev>")
        .about("Make markdown articles into static web pages.")
        .arg(
            Arg::with_name("input")
                .required(true)
                .takes_value(true)
                .long("input")
                .short("i")
                .help("Input directory with markdown articles and media.")
        )
        .arg(
            Arg::with_name("template")
                .required(true)
                .takes_value(true)
                .long("template")
                .short("t")
                .help("HTML template.")
        )
        .arg(
            Arg::with_name("no-media")
                .takes_value(false)
                .long("no-media")
                .short("M")
                .help("Disable copying files in source directory.")
        )
        .arg(
            Arg::with_name("no-digest")
                .takes_value(false)
                .long("no-digest")
                .short("D")
                .help("Disable JSON digests for listing articles on pages.")
        )
        .arg(
            Arg::with_name("output")
                .required(true)
                .takes_value(true)
                .long("output")
                .short("o")
                .help("Output directory.")
        )
        .get_matches();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();

    let template_path = matches.value_of("template").unwrap();
    let mut template_file = File::open(template_path).expect("Failed to open template.");
    let mut template = String::new();
    template_file.read_to_string(&mut template).expect("Failed to read template.");

    let mut entries: Vec<DirEntry> = read_dir(input)
        .expect("Failed to read input directory")
        .map(|x| x.expect("Failed to read directory item."))
        .into_iter()
        .collect();

    let mut made_directories: HashSet<PathBuf> = HashSet::new();

    let mut digest: Vec<serde_json::Value> = Vec::new();

    loop {
        let entry = match entries.pop() {
            Some(entry) => entry,
            None => break
        };

        let path_buf = entry.path();
        let metadata = entry.metadata().expect("Failed to get entry metadata");

        if metadata.is_dir() {
            entries.extend(
                read_dir(&path_buf)
                    .expect("Failed to read entry directory.")
                    .map(|x| x.expect("Failed to read entry directory item."))
                    .into_iter()
            );
        } else if metadata.is_file() {
            match path_buf.extension().unwrap().to_str().unwrap() {
                "md" | "markdown" => {
                    // TODO: Stream larger articles? ¯\_(ツ)_/¯
                    let mut markdown_file = File::open(&path_buf).expect("Failed to open markdown file.");
                    let mut markdown = Vec::new();

                    markdown_file.read_to_end(&mut markdown).expect("Failed to read markdown file.");

                    let (markdown, front_matter) = match opt(parse_front_matter)(&markdown) {
                        Ok((rest, Some(front_matter))) => {
                            // TODO: Probably unnecessary to convert YAML to JSON with serde.
                            let json = serde_json::to_string(&front_matter).expect("Failed to use front matter.");
                            let front_matter = match serde_json::from_str(&json) {
                                Ok(serde_json::Value::Object(map)) => map,
                                _ => serde_json::Map::new(),
                            };
                            (rest, front_matter)
                        }
                        _ => {
                            (markdown.as_slice(), serde_json::Map::new())
                        }
                    };

                    // render markdown to html
                    let markdown = std::str::from_utf8(&markdown).expect("Failed to read markdown as UTF8");

                    let relative_path = &path_buf.strip_prefix(&input).expect("Failed to get relative path.");
                    let relative_path = relative_path.with_extension("html");
                    let adjusted_path = Path::new(output).join(relative_path.clone());
                    let adjusted_path_directory = adjusted_path.parent().expect("Failed to get article's directory.");

                    let mdc_opts = pulldown_cmark::Options::empty();
                    let mdc_parser = pulldown_cmark::Parser::new_ext(&markdown, mdc_opts);

                    let mut article = String::new();
                    pulldown_cmark::html::push_html(&mut article, mdc_parser);

                    let relative_path_str = relative_path
                        .components()
                        .map(|component| {
                            match component {
                                std::path::Component::Normal(str) => str.to_str().unwrap(),
                                _ => panic!("Failed to create valid URL from path."),
                            }
                        })
                        .collect::<Vec<&str>>()
                        .join("/");

                    let mut template_data = front_matter.clone();

                    template_data.insert("article".to_owned(), serde_json::Value::String(article));
                    template_data.insert("url".to_owned(), serde_json::Value::String(relative_path_str));

                    let template_data = serde_json::value::Value::Object(template_data);

                    let handlebars = Handlebars::new();

                    let html = handlebars.render_template(&template, &template_data).expect("Failed to render template.");

                    if matches.value_of("no-digest").is_none() {
                        digest.push(template_data);
                    }

                    if made_directories.get(adjusted_path_directory).is_none() {
                        create_dir_all(adjusted_path_directory).expect("Failed to create article's directory.");
                        made_directories.insert(adjusted_path_directory.to_owned().clone());
                    }

                    write(adjusted_path, html.as_bytes()).expect("Failed to write article.");
                }
                _ => {
                    if matches.value_of("no-media").is_some() {
                        continue
                    }

                    let adjusted_path = &path_buf.strip_prefix(&input).expect("Failed to get relative path.");
                    let adjusted_path = Path::new(output).join(adjusted_path);
                    let adjusted_path_directory = adjusted_path.parent().expect("Failed to get media's directory.");

                    if made_directories.get(adjusted_path_directory).is_none() {
                        create_dir_all(adjusted_path_directory).expect("Failed to create media's directory.");
                        made_directories.insert(adjusted_path_directory.to_owned().clone());
                    }

                    copy(path_buf, adjusted_path).expect("Failed to copy media file.");
                }
            }
        }
    }

    if matches.value_of("no-digest").is_none() {
        // TODO: Create multiple lists after exceeding a certain size?

        create_dir_all(output).expect("Failed to create output directory for digest");

        let digest = serde_json::Value::Array(digest);
        let digest = serde_json::to_string(&digest).expect("Failed to serialize digest.");
        let digest_path = Path::new(output).join("digest0.json");

        write(&digest_path, &digest).expect("Failed to write digest file.");
    }
}

//
// Parse Markdown front-matter
//

use nom::IResult;
use nom::bytes::complete::{tag,take_until};
use nom::multi::many0;
use nom::character::complete::line_ending;
use nom::combinator::opt;

fn parse_front_matter(input0: &[u8]) -> IResult<&[u8], serde_yaml::Value> {
    let (input, _) = tag("---")(input0)?;
    let (input, _) = many0(tag("-"))(input)?;
    let (input, _) = line_ending(input)?;
    let (input, yaml) = take_until("---")(input)?;
    let (input, _) = many0(tag("-"))(input)?;
    let (input, _) = line_ending(input)?;

    match serde_yaml::from_slice(&yaml) {
        Ok(front_matter) => Ok((input, front_matter)),
        Err(_) => Err(nom::Err::Error((input0, nom::error::ErrorKind::ParseTo)))
    }
}