use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Deserializer};
use time::Date;

#[derive(Deserialize, Debug)]
pub(crate) struct Listing {
    // name: String,
    no: String,
    previous_no: Option<String>,
    // next_no: Option<String>,
    #[serde(deserialize_with = "deserialize_polish_date")]
    pub(crate) published_at_date: Date,
    // document: String,
    // title: String,
    // title_template: String,
    // summary: Summary,
    pub(crate) results: HashMap<String, Section>,
}

impl Listing {
    pub(crate) async fn fetch_latest(client: &Client) -> color_eyre::Result<Self> {
        Self::fetch(client, "latest").await
    }
    pub(crate) async fn fetch(client: &Client, id: &str) -> color_eyre::Result<Self> {
        Ok(client
            .get(format!("https://wyniki.radio357.pl/api/charts/lista/{id}"))
            .send()
            .await?
            .json()
            .await?)
    }
    pub(crate) async fn fetch_previous(&self, client: &Client) -> color_eyre::Result<Option<Self>> {
        let Some(previous_no) = self.previous_no.as_ref() else {
            return Ok(None);
        };
        Self::fetch(client, &previous_no).await.map(|r| Some(r))
    }
    pub(crate) fn int_no(&self) -> color_eyre::Result<u16> {
        Ok(self.no.parse()?)
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct Section {
    pub(crate) label: Option<String>,
    pub(crate) items: Vec<Item>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Item {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) artist: String,
    pub(crate) position: u16,
    // is_new: bool,
    // last_position: usize,
    // times_on_chart: usize,
    // change: isize | false,
}

fn deserialize_polish_date<'de, D: Deserializer<'de>>(de: D) -> Result<Date, D::Error> {
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Date;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a string representing a dd.mm.yyyy date")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Date::parse(v, time::macros::format_description!("[day].[month].[year]"))
                .map_err(serde::de::Error::custom)
        }
    }
    de.deserialize_str(Visitor)
}
