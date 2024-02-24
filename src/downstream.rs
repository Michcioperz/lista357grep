use rocket::fairing::{AdHoc, Fairing};
use rocket_db_pools::{sqlx, Connection, Database};
use serde::Serialize;
use sqlx::{query, query_scalar, SqlitePool};
use time::Date;

use crate::{upstream::Listing, utils::MyError};

#[derive(Database)]
#[database("main")]
pub(crate) struct Db(SqlitePool);

pub(crate) fn stage() -> impl Fairing {
    AdHoc::try_on_ignite("database", |rocket| async {
        Ok(rocket.attach(Db::init()).attach(AdHoc::try_on_ignite(
            "database migrations",
            |rocket| async {
                let Some(db) = Db::fetch(&rocket) else {
                    return Err(rocket);
                };
                let Ok(()) = sqlx::migrate!().run(&**db).await else {
                    return Err(rocket);
                };
                Ok(
                    rocket.attach(AdHoc::try_on_ignite("database backfill", |rocket| async {
                        let Some(db) = Db::fetch(&rocket) else {
                            return Err(rocket);
                        };
                        let Ok(()) = backfill(&**db).await else {
                            return Err(rocket);
                        };
                        Ok(rocket)
                    })),
                )
            },
        )))
    })
}

async fn backfill(db: &SqlitePool) -> color_eyre::Result<()> {
    let mut db = db.begin().await?;

    let latest_local = query_scalar!(
        "
            SELECT id
            FROM listing
            ORDER BY id DESC
            LIMIT 1
        "
    )
    .fetch_optional(&mut *db)
    .await?
    .map_or(0u16, |v| v.try_into().unwrap());

    let client = reqwest::Client::new();
    let mut listing = Listing::fetch_latest(&client).await?;
    loop {
        let int_no = listing.int_no()?;
        if latest_local < int_no {
            query!(
                "INSERT INTO listing (id, date) VALUES (?, ?)",
                int_no,
                listing.published_at_date,
            )
            .execute(&mut *db)
            .await?;
            for section in listing.results.values() {
                for song in &section.items {
                    query!(
                        "INSERT INTO song (id, title, artist) VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
                        song.id,
                        song.name,
                        song.artist,
                    ).execute(&mut *db).await?;
                    let waiting_room = section.label.is_some();
                    query!(
                        "INSERT INTO position (song_id, listing_id, position, waiting_room) VALUES (?, ?, ?, ?)",
                        song.id,
                        int_no,
                        song.position,
                        waiting_room,
                    ).execute(&mut *db).await?;
                }
            }
        }

        if let Some(prev_listing) = listing.fetch_previous(&client).await? {
            listing = prev_listing;
        } else {
            break;
        }
    }
    db.commit().await?;
    Ok(())
}



pub(crate) async fn get_results(
    mut db: Connection<Db>,
    max_pos: Option<u16>,
    exclude_waiting_room: Option<bool>,
    limit: Option<u32>,
) -> Result<Vec<ApiItem>, MyError> {
    let max_pos = max_pos.unwrap_or(u16::MAX);
    let waiting_room = if exclude_waiting_room.unwrap_or(false) {
        0
    } else {
        1
    };
    let limit = limit.unwrap_or(u32::MAX);
    let items = sqlx::query!(
        "
            SELECT
                song.id AS id,
                song.title AS title,
                song.artist AS artist,
                listing.date AS min_date,
                listing.id AS initial_listing,
                p0.position AS initial_position
            FROM
                position p0
                LEFT OUTER JOIN position p1 ON p0.song_id = p1.song_id AND p1.listing_id < p0.listing_id
                INNER JOIN song on p0.song_id = song.id
                INNER JOIN listing ON p0.listing_id = listing.id
            WHERE
                p1.id IS NULL
                AND p0.position <= ?
                AND p0.waiting_room <= ?
            ORDER BY min_date DESC, p0.position ASC
            LIMIT ?
        ",
        max_pos,
        waiting_room,
        limit,
    ).map(|row| ApiItem {
        id: row.id.try_into().unwrap(),
        title: row.title,
        artist: row.artist,
        min_date: row.min_date,
        initial_listing: row.initial_listing.try_into().unwrap(),
        initial_position: row.initial_position.try_into().unwrap()
    })
    .fetch_all(&mut **db)
    .await?;
    Ok(items)
}

#[derive(Serialize, Debug)]
pub(crate) struct ApiItem {
    pub(crate) id: u32,
    pub(crate) title: String,
    pub(crate) artist: String,
    #[serde(serialize_with = "crate::utils::to_isoish_date")]
    pub(crate) min_date: Date,
    pub(crate) initial_listing: u32,
    pub(crate) initial_position: u16,
}
