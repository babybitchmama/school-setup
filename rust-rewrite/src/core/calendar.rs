use crate::config::LessonManagerConfigFile;
use crate::core::{
    change_current_course, get_courses_in_path, get_current_course_info,
    update_polybar_current_course,
};
use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use regex::Regex;
use serde::Deserialize;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const MAX_LENGTH: usize = 140;
const DELAY_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct Event {
    pub summary: String,
    pub location: Option<String>,
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
    pub kind: String,
}

// ---------- Google API response shapes ----------

#[derive(Deserialize)]
struct EventList {
    #[serde(default)]
    items: Vec<RawEvent>,
}

#[derive(Deserialize)]
struct RawEvent {
    summary: Option<String>,
    location: Option<String>,
    start: RawTime,
    end: RawTime,
}

#[derive(Deserialize)]
struct RawTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
}

// ---------- polybar formatting helpers ----------

fn colored_text(text: &str) -> String {
    format!("%{{F#999999}}{}%{{F-}}", text)
}

fn join(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

fn short_title(title: &str, max_len: usize) -> String {
    const ELLIPSIS: &str = " ...";
    if title.chars().count() < max_len {
        return title.to_string();
    }
    let keep = max_len.saturating_sub(ELLIPSIS.len());
    format!("{}{}", title.chars().take(keep).collect::<String>(), ELLIPSIS)
}

fn location(text: &Option<String>) -> String {
    match text {
        Some(t) if !t.is_empty() => format!("{} {}", colored_text("in"), t),
        _ => String::new(),
    }
}

fn summary(text: &str) -> String {
    let re = Regex::new(r"X[0-9A-Za-z]+").unwrap();
    short_title(re.replace_all(text, "").trim(), 45)
}

fn format_time(t: &DateTime<Tz>) -> String {
    t.format("%I:%M %p").to_string()
}

fn format_duration(begin: &DateTime<Tz>, end: &DateTime<Tz>) -> String {
    let secs = (*end - *begin).num_seconds().max(0);
    let minutes = (secs + 59) / 60;

    if minutes == 1 {
        return "1 minute".into();
    }
    if minutes < 60 {
        return format!("{} minutes", minutes);
    }

    let hours = minutes / 60;
    let rest = minutes % 60;

    if hours == 1 && rest == 0 {
        "1 hour".into()
    } else if hours == 1 {
        format!("1 hour {} min", rest)
    } else if rest == 0 || hours > 5 {
        format!("{} hours", hours)
    } else {
        format!("{} hr {:02} min", hours, rest)
    }
}

fn paren_time(t: &DateTime<Tz>) -> String {
    format!("({})", format_time(t))
}

// ---------- the status line ----------

fn text(events: &[Event], now: &DateTime<Tz>) -> String {
    let period = colored_text(".");

    let current = events.iter().find(|e| e.start < *now && *now < e.end);

    let Some(current) = current else {
        let Some(nxt) = events.iter().find(|e| *now <= e.start) else {
            return String::new();
        };
        let loc = location(&nxt.location);
        let start = format_duration(now, &nxt.start);
        return short_title(
            &join(&[
                colored_text(&nxt.kind),
                summary(&nxt.summary),
                colored_text("starts in"),
                if loc.is_empty() { format!("{}{}", start, period) } else { start },
                if loc.is_empty() { String::new() } else { format!("{}{}", loc, period) },
                paren_time(&nxt.start),
            ]),
            MAX_LENGTH,
        );
    };

    let Some(nxt) = events.iter().find(|e| e.start >= current.end) else {
        return short_title(
            &join(&[
                colored_text(&current.kind),
                colored_text("Ends in"),
                format!("{}{}", format_duration(now, &current.end), period),
                paren_time(&current.end),
            ]),
            MAX_LENGTH,
        );
    };

    if current.end == nxt.start {
        return short_title(
            &join(&[
                colored_text(&nxt.kind),
                colored_text("Ends in"),
                format!("{}{}", format_duration(now, &current.end), period),
                colored_text("Next is"),
                summary(&nxt.summary),
                format!("{}{}", location(&nxt.location), period),
                paren_time(&nxt.start),
            ]),
            MAX_LENGTH,
        );
    }

    short_title(
        &join(&[
            colored_text(&nxt.kind),
            colored_text("Ends in"),
            format!("{}{}", format_duration(now, &current.end), period),
            colored_text("Next is"),
            summary(&nxt.summary),
            location(&nxt.location),
            colored_text("after a"),
            format_duration(&current.end, &nxt.start),
            colored_text("break."),
            paren_time(&current.end),
        ]),
        MAX_LENGTH,
    )
}

// ---------- course switching ----------

fn activate_course(config: &LessonManagerConfigFile, event: &Event) {
    for folder in get_courses_in_path(&config.root) {
        let info_path = format!("{}/{}/info.yaml", config.root, folder);
        let info = get_current_course_info(&info_path);

        if info.short == event.summary {
            change_current_course(&config.root, &folder, &config.notes_dir);
            let polybar = shellexpand::tilde(&config.polybar_current_course_file);
            update_polybar_current_course(&polybar, &info.short);
            return;
        }
    }
}

// ---------- network ----------

fn check_internet() {
    loop {
        let reachable = ("1.1.1.1", 53)
            .to_socket_addrs()
            .ok()
            .and_then(|mut a| a.next())
            .map(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok())
            .unwrap_or(false);

        if reachable {
            return;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

async fn fetch_events(
    config: &LessonManagerConfigFile,
    tz: Tz,
    morning: DateTime<Tz>,
    evening: DateTime<Tz>,
) -> Res<Vec<Event>> {
    let calendar_id = config
        .calendar_id
        .as_ref()
        .ok_or("`calendar_id` is not set in config.yaml")?;

    let dir = shellexpand::tilde("~/.config/lesson-manager/credentials/google-calendar");
    let secret = yup_oauth2::read_application_secret(format!("{}/credentials.json", dir)).await?;
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(format!("{}/token.json", dir))
        .build()
        .await?;
    let token = auth.token(&[SCOPE]).await?;
    let access = token.token().ok_or("no access token returned")?;

    println!("Authenticated");
    println!("Searching for events");

    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
        calendar_id.replace('@', "%40").replace('#', "%23")
    );

    let list: EventList = reqwest::Client::new()
        .get(url)
        .bearer_auth(access)
        .query(&[
            ("timeMin", morning.to_rfc3339()),
            ("timeMax", evening.to_rfc3339()),
            ("maxResults", "10".to_string()),
            ("singleEvents", "true".to_string()),
            ("orderBy", "startTime".to_string()),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let re = Regex::new(r"^(Class|Meeting|Tutor|Work): ([ a-zA-Z0-9-]+)$").unwrap();
    let mut events = Vec::new();

    for raw in list.items {
        let (Some(title), Some(start), Some(end)) =
            (raw.summary, raw.start.date_time, raw.end.date_time)
        else {
            continue; // all-day events have no dateTime
        };
        let Some(caps) = re.captures(&title) else { continue };

        let (Ok(start), Ok(end)) = (
            DateTime::parse_from_rfc3339(&start),
            DateTime::parse_from_rfc3339(&end),
        ) else {
            continue;
        };

        events.push(Event {
            kind: caps[1].to_string(),
            summary: caps[2].to_string(),
            location: raw.location,
            start: start.with_timezone(&tz),
            end: end.with_timezone(&tz),
        });
    }

    Ok(events)
}

// ---------- entry point ----------

pub fn main(config: &LessonManagerConfigFile) {
    println!("Waiting for connection");
    check_internet();
    println!("Initializing");

    let tz: Tz = match std::env::var("TZ").ok().and_then(|s| s.parse().ok()) {
        Some(tz) => tz,
        None => {
            eprintln!("Warning: TZ environment variable not set (or not a valid timezone)");
            return;
        }
    };

    let now = Utc::now().with_timezone(&tz);
    let at = |h, m| {
        tz.from_local_datetime(&now.date_naive().and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap()))
            .single()
            .unwrap_or(now)
    };
    let (morning, evening) = (at(6, 0), at(23, 59));

    let events = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(fetch_events(config, tz, morning, evening))
    {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to fetch calendar events: {}", e);
            return;
        }
    };

    println!("Done\n");

    let mut activated = vec![false; events.len()];
    let mut last_print: Option<Instant> = None;

    loop {
        let now = Utc::now().with_timezone(&tz);

        for (i, event) in events.iter().enumerate() {
            if !activated[i] && event.start <= now && now < event.end {
                activated[i] = true;
                activate_course(config, event);
            }
        }

        if last_print.map_or(true, |t| t.elapsed().as_secs() >= DELAY_SECS) {
            println!("{}", text(&events, &now));
            let _ = std::io::stdout().flush(); // polybar reads a pipe, so flush
            last_print = Some(Instant::now());
        }

        if now >= evening {
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
