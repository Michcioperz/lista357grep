use crate::utils::Result;
use downstream::Db;
use rocket::{
    fairing::AdHoc,
    get,
    http::{uri::Absolute, ContentType},
    launch,
    serde::json::Json,
    uri, Config, State,
};
use rocket_db_pools::Connection;

pub(crate) mod atom;
pub(crate) mod downstream;
pub(crate) mod upstream;
pub(crate) mod utils;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![style_css, cute_form, feed, api])
        .attach(AdHoc::try_on_ignite("public url config", |rocket| async {
            let Ok(public_url) = Config::figment().extract_inner("public_url") else {
                return Err(rocket);
            };
            Ok(rocket.manage(PublicUrl(public_url)))
        }))
        .attach(downstream::stage())
}

struct PublicUrl<'a>(Absolute<'a>);

#[get("/?<max_pos>&<exclude_waiting_room>&<limit>")]
async fn cute_form(
    db: Connection<Db>,
    max_pos: Option<u16>,
    exclude_waiting_room: Option<bool>,
    limit: Option<u32>,
) -> Result<maud::Markup> {
    let results = downstream::get_results(db, max_pos, exclude_waiting_room, limit).await?;
    Ok(maud::html! {
        html lang="pl" {
            head {
                meta charset="UTF-8";
                title { "lista357grep" }
                link rel="stylesheet" href=(uri!(style_css()));
                link rel="alternate" type="application/atom+xml" href=(uri!(feed(max_pos, exclude_waiting_room, limit)));
            }
            body.container {
                nav {
                    form action="" method="GET" {
                        .input-group {
                            label for="max_pos" { "tylko utwory które wspięły się do miejsca" }
                            input id="max_pos" name="max_pos" type="number" min="1" value=[max_pos];
                        }
                        .input-group {
                            input id="exclude_waiting_room" name="exclude_waiting_room" type="checkbox" value="true" checked[exclude_waiting_room.is_some_and(|v| v)];
                            label for="exclude_waiting_room" { "tylko utwory które wybiły się z poczekalni" }
                        }
                        .input-group {
                            label for="limit" { "obetnij do najnowszych" }
                            input id="limit" name="limit" type="number" min="1" value=[limit];
                        }
                        button type="submit" { "szukaj" }
                    }
                }
                main {
                    @for entry in results {
                        article {
                            h1 { (entry.title) }
                            h2 { (entry.artist) }
                            p {
                                "debiut w notowaniu #"
                                (entry.initial_listing)
                                " ("
                                (entry.min_date)
                                ") na miejscu "
                                (entry.initial_position)
                            }
                        }
                    }
                }
            }
        }
    })
}

#[get("/feed.atom?<max_pos>&<exclude_waiting_room>&<limit>")]
async fn feed(
    db: Connection<Db>,
    public_url: &State<PublicUrl<'_>>,
    max_pos: Option<u16>,
    exclude_waiting_room: Option<bool>,
    limit: Option<u32>,
) -> Result<(ContentType, String)> {
    let feed_url = uri!(
        public_url.0.clone(),
        feed(max_pos, exclude_waiting_room, limit)
    );
    let mut buf = r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string();
    quick_xml::se::to_writer(
        &mut buf,
        &atom::Feed::from((
            feed_url,
            downstream::get_results(db, max_pos, exclude_waiting_room, limit).await?,
        )),
    )?;
    Ok((ContentType::XML, buf))
}

#[get("/api/v1?<max_pos>&<exclude_waiting_room>&<limit>")]
async fn api(
    db: Connection<Db>,
    max_pos: Option<u16>,
    exclude_waiting_room: Option<bool>,
    limit: Option<u32>,
) -> Result<Json<Vec<downstream::ApiItem>>> {
    downstream::get_results(db, max_pos, exclude_waiting_room, limit)
        .await
        .map(Json)
}

#[get("/style.css")]
fn style_css() -> (ContentType, &'static str) {
    (ContentType::CSS, include_str!("style.css"))
}
