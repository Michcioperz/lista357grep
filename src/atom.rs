use rocket::http::uri::Absolute;
use serde::Serialize;
use time::{macros::time, OffsetDateTime};
use time_tz::{timezones, PrimitiveDateTimeExt};

use crate::downstream::ApiItem;

#[derive(Serialize, Debug)]
#[serde(rename = "feed")]
pub(crate) struct Feed {
    #[serde(rename = "@xmlns")]
    namespace: &'static str,
    pub(crate) title: String,
    pub(crate) id: String,
    #[serde(serialize_with = "crate::utils::to_isoish_time")]
    pub(crate) updated: OffsetDateTime,

    #[serde(rename = "entry")]
    pub(crate) entries: Vec<Entry>,
}

impl Default for Feed {
    fn default() -> Self {
        Self {
            title: "lista357grep".to_string(),
            id: "urn:uuid:4FC09A65-97AD-4E75-AAC3-04B2320153D3".to_string(),
            entries: Default::default(),
            namespace: "http://www.w3.org/2005/Atom",
            updated: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) author: Author,
    #[serde(serialize_with = "crate::utils::to_isoish_time")]
    pub(crate) published: OffsetDateTime,
    #[serde(serialize_with = "crate::utils::to_isoish_time")]
    pub(crate) updated: OffsetDateTime,
    pub(crate) content: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct Author {
    pub(crate) name: String,
}

impl From<(Absolute<'_>, Vec<ApiItem>)> for Feed {
    fn from((feed_url, items): (Absolute<'_>, Vec<ApiItem>)) -> Self {
        let mut feed = Feed::default();
        feed.id = feed_url.to_string();
        for item in items {
            let published = item
                .min_date
                .with_time(time!(22:00))
                .assume_timezone(timezones::db::europe::WARSAW)
                .unwrap()
                - time::Duration::seconds(item.initial_position.into());
            let entry = Entry {
                id: format!("{}#{}", feed_url, item.id),
                title: item.title,
                author: Author { name: item.artist },
                updated: published.clone(),
                published,
                content: format!(
                    "debiut w notowaniu #{} ({}) na miejscu {}",
                    item.initial_listing, item.min_date, item.initial_position
                ),
            };
            feed.updated = feed.updated.max(entry.published);
            feed.entries.push(entry);
        }
        feed
    }
}
