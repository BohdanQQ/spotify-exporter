use crate::{CmdRange, Commands, OutputType};
use rspotify::clients::OAuthClient;
use rspotify::model::{FullTrack, TimeRange};
use rspotify::{AuthCodeSpotify, ClientError};
use serde_json::json;
use std::fs::File;
use std::io::{LineWriter, Write};

pub trait Command {
    fn execute(&mut self, spotify: &AuthCodeSpotify) -> Result<(), ClientError>;
    fn output(&self, file: &File, format: &OutputType) -> Result<(), String>;
}

impl From<CmdRange> for TimeRange {
    fn from(value: CmdRange) -> Self {
        match value {
            CmdRange::Short => TimeRange::ShortTerm,
            CmdRange::Medium => TimeRange::MediumTerm,
            CmdRange::Long => TimeRange::LongTerm,
        }
    }
}

fn join_vec(vec: &[String], sep: &str) -> String {
    let mut res = "".to_string();
    for i in 0..vec.len() {
        res = res + &vec[i];
        if i != vec.len() - 1 {
            res += sep
        }
    }
    res
}

pub struct TopTracksCommand {
    time: TimeRange,
    count: u8,
    saved_result: Vec<FullTrack>,
}

impl TopTracksCommand {
    fn new(time: CmdRange, count: u8) -> TopTracksCommand {
        let real_count = if count > 50 { 50 } else { count };
        TopTracksCommand {
            count: real_count,
            time: TimeRange::from(time),
            saved_result: vec![],
        }
    }
}

struct ConvertedExportData {
    name: String,
    artists: Vec<String>,
    url: Option<String>,
    preview_url: Option<String>,
    album: String,
    counter: u8,
}

trait ToOutputType {
    fn to_output_type(&self, t: OutputType) -> String;
}

impl ToOutputType for ConvertedExportData {
    fn to_output_type(&self, t: OutputType) -> String {
        format!(
            "{}. {} ({})\n* Album: {}\n{}{}\n\n",
            self.counter,
            self.name,
            join_vec(&self.artists, ", "),
            self.album,
            match &self.url {
                Some(url) => format!("* [Spotify Link]({})", &url),
                None => "".to_string(),
            },
            match &self.preview_url {
                Some(url) => format!("* [Preview]({})\n", url),
                None => "".to_string(),
            }
        );
        match t {
            OutputType::Markdown => format!("{}. {} ({})\n* Album: {}\n{}{}\n\n", self.counter, self.name, join_vec(&self.artists, ", "), self.album,
                                            match &self.preview_url {
                                                Some(url) => format!("* [Preview]({})\n", url),
                                                None => "".to_string()
                                            },
                                            match &self.url {
                                                Some(url) => format!("* [Spotify Link]({})", &url),
                                                None => "".to_string()
                                            }),
            OutputType::HTML => format!(
                "<li>{}</li>\n<ul><li>By: {}</li>\n<li>Album: {}</li>\n{}{}</ul>",
                html_escape::encode_safe(&self.name),
                html_escape::encode_safe(&join_vec(&self.artists, ", ")),
                html_escape::encode_safe(&self.album),
                match &self.url {
                    Some(url) => format!("<li> <a href=\"{}\">Spotify Link</a></li>", html_escape::encode_safe(&url)),
                    None => "".to_string()
                },
                match &self.preview_url {
                    Some(url) => format!("<li><audio controls><source src=\"{}\" type=\"audio/mpeg\">Your browser does not support the audio element.</audio></li>", html_escape::encode_safe(&url)),
                    None => "".to_string()
                }
            ),
            OutputType::MarkdownWWW => match &self.url {
                Some(url) => format!("{}. [{} ({})]({})\n", self.counter, self.name, join_vec(&self.artists, ", "), url),
                None => format!("{}. {} ({})\n", self.counter, self.name, join_vec(&self.artists, ", "), )
            },
            OutputType::JSON => json!({
                        "position": self.counter,
                        "name": self.name,
                        "artists": join_vec(&self.artists, ", "),
                        "url": self.url,
                        "preview_url": self.preview_url
                    }).to_string()
        }
    }
}

impl Command for TopTracksCommand {
    fn execute(&mut self, spotify: &AuthCodeSpotify) -> Result<(), ClientError> {
        let stream = spotify.current_user_top_tracks(Some(&self.time));
        for item in stream {
            match item {
                Ok(track) => {
                    self.saved_result.push(track);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    fn output(&self, file: &File, format: &OutputType) -> Result<(), String> {
        let mut writer = LineWriter::new(file);

        let mut counter = 0;
        let data_strings = &self
            .saved_result
            .iter()
            .take(usize::from(self.count))
            .map(|x| {
                counter += 1;
                ConvertedExportData {
                    name: x.name.to_string(),
                    artists: x
                        .artists
                        .iter()
                        .map(|x| x.name.clone())
                        .collect::<Vec<String>>(),
                    preview_url: x.preview_url.clone(),
                    url: x.external_urls.get("spotify").map(|x| x.to_string()),
                    album: x.album.name.clone(),
                    counter,
                }
            })
            .map(|x| x.to_output_type(*format))
            .collect::<Vec<String>>();

        let to_write = match format {
            OutputType::Markdown => join_vec(data_strings, ""),
            OutputType::MarkdownWWW => join_vec(data_strings, ""),
            OutputType::HTML => format!("<ol>{}</ol>", join_vec(data_strings, "")),
            OutputType::JSON => format!("[{}]", join_vec(data_strings, ",\n")),
        };

        match writer.write(to_write.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn make_command(command: Commands) -> Box<dyn Command> {
    match command {
        Commands::TopTracks { time, count } => Box::new(TopTracksCommand::new(time, count)),
    }
}
