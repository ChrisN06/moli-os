//! Normal So'EAU form login and bounded daily telemetry; no browser cookies or challenge solving.
use anyhow::{Context as _, ensure};
use http::{HeaderMap, Method, StatusCode, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fmt::Write as _, time::Duration};

const HOST: &str = "so-eau.toutsurmoneau.fr";
const LOGIN: &str = "/mon-compte-en-ligne/je-me-connecte";
const METERS: &str = "/public-api/cel-consumption/meters-list";
const TELEMETRY: &str = "/public-api/cel-consumption/telemetry";
pub(super) type Result<T> = std::result::Result<T, &'static str>;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Account {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub meter: String,
}
impl Account {
    pub fn valid(&self) -> bool {
        !self.email.is_empty()
            && self.email.len() <= 254
            && self.email.contains('@')
            && !self.password.is_empty()
            && self.password.len() <= 1024
            && (self.meter.is_empty() || meter_valid(&self.meter))
    }
}
fn meter_valid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}
fn encode(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}
fn csrf(page: &str) -> anyhow::Result<String> {
    for marker in [
        r"\\u0022csrfToken\\u0022\\u003A\\u0022",
        r"\u0022csrfToken\u0022\u003A\u0022",
    ] {
        if let Some((_, rest)) = page.split_once(marker) {
            let token = rest.split('\\').next().unwrap_or_default();
            ensure!(
                !token.is_empty()
                    && token.len() <= 512
                    && token
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)),
                "invalid CSRF"
            );
            return Ok(token.to_owned());
        }
    }
    anyhow::bail!("missing CSRF")
}

// Only one fixed HTTPS origin. Redirects are never followed, especially after sending credentials.
struct Client {
    host: String,
    port: u16,
    tls: bool,
    cookies: BTreeMap<String, String>,
}
impl Client {
    fn new() -> Self {
        Self {
            host: HOST.into(),
            port: 443,
            tls: true,
            cookies: BTreeMap::new(),
        }
    }
    async fn request(
        &mut self,
        path: &str,
        form: Option<String>,
    ) -> anyhow::Result<(StatusCode, HeaderMap, moli_net::Body)> {
        let mut builder = http::Request::builder()
            .method(if form.is_some() {
                Method::POST
            } else {
                Method::GET
            })
            .uri(path)
            .header(header::ACCEPT_ENCODING, "identity")
            .header(header::ACCEPT, "application/json,text/html")
            .header(header::USER_AGENT, "Moli OS water integration");
        if !self.cookies.is_empty() {
            builder = builder.header(
                header::COOKIE,
                self.cookies
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("; "),
            );
        }
        let request = if let Some(form) = form {
            builder = builder
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::ORIGIN, format!("https://{}", self.host));
            builder.body(form.into_bytes().into())?
        } else {
            moli_net::empty(builder)?
        };
        let (status, headers, body) = moli_net::web_with_headers(
            &self.host,
            self.port,
            self.tls,
            request,
            Duration::from_secs(12),
            1024 * 1024,
        )
        .await?;
        for cookie in headers.get_all(header::SET_COOKIE) {
            let text = cookie.to_str()?;
            if let Some((name, value)) = text.split(';').next().and_then(|s| s.split_once('=')) {
                ensure!(
                    name.len() <= 128 && value.len() <= 4096 && self.cookies.len() < 32,
                    "cookie size"
                );
                ensure!(
                    name.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
                        && value
                            .bytes()
                            .all(|b| (0x21..=0x7e).contains(&b) && b != b';'),
                    "invalid cookie"
                );
                self.cookies.insert(name.to_owned(), value.to_owned());
            }
        }
        Ok((status, headers, body))
    }
    async fn json(&mut self, path: &str) -> Result<Value> {
        let (status, headers, body) = self
            .request(path, None)
            .await
            .map_err(|_| "water.connection_failed")?;
        if status.is_redirection() || status == StatusCode::UNAUTHORIZED {
            return Err("water.login_failed");
        }
        if status == StatusCode::FORBIDDEN {
            return Err("water.human_required");
        }
        if !status.is_success() {
            return Err("water.connection_failed");
        }
        if !headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|s| s.starts_with("application/json"))
        {
            return Err("water.login_failed");
        }
        let reply: Value = serde_json::from_slice(&body).map_err(|_| "water.data_failed")?;
        reply
            .get("content")
            .filter(|v| !v.is_null())
            .cloned()
            .ok_or("water.data_failed")
    }
    async fn login(&mut self, account: &Account) -> Result<()> {
        let (status, _, page) = self
            .request(LOGIN, None)
            .await
            .map_err(|_| "water.connection_failed")?;
        if status == StatusCode::FORBIDDEN {
            return Err("water.human_required");
        }
        if status != StatusCode::OK {
            return Err("water.connection_failed");
        }
        let token = csrf(std::str::from_utf8(&page).map_err(|_| "water.login_failed")?)
            .map_err(|_| "water.login_failed")?;
        let form = [
            ("_csrf_token", token.as_str()),
            ("tsme_user_login[_username]", account.email.as_str()),
            ("tsme_user_login[_password]", account.password.as_str()),
            (
                "tsme_user_login[_target_path]",
                "/mon-compte-en-ligne/tableau-de-bord",
            ),
        ]
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .join("&");
        let (status, _, _) = self
            .request(LOGIN, Some(form))
            .await
            .map_err(|_| "water.connection_failed")?;
        if status == StatusCode::FORBIDDEN {
            return Err("water.human_required");
        }
        if status != StatusCode::OK && !status.is_redirection() {
            return Err("water.login_failed");
        }
        Ok(())
    }
}
fn select_meter(data: &Value, wanted: &str) -> Result<String> {
    let mut ids = Vec::new();
    for client in data["clientCompteursPro"]
        .as_array()
        .ok_or("water.data_failed")?
    {
        for meter in client["compteursPro"]
            .as_array()
            .ok_or("water.data_failed")?
        {
            let id = match &meter["idPDS"] {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => return Err("water.data_failed"),
            };
            if !meter_valid(&id) {
                return Err("water.data_failed");
            }
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    if wanted.is_empty() && ids.len() == 1 {
        return Ok(ids.remove(0));
    }
    if ids.iter().any(|id| id == wanted) {
        return Ok(wanted.to_owned());
    }
    Err("water.meter_required")
}
fn readings(reply: &Value) -> anyhow::Result<BTreeMap<String, f64>> {
    let mut days = BTreeMap::new();
    for row in reply["measures"].as_array().context("water measures")? {
        let date = row["date"]
            .as_str()
            .context("water date")?
            .split(' ')
            .next()
            .unwrap_or_default();
        ensure!(super::water::date(date), "water date");
        let volume = row["volume"].as_f64().context("water volume")?;
        ensure!(
            volume.is_finite() && (0.0..=1_000_000.0).contains(&volume),
            "water volume"
        );
        if row["index"].is_null() {
            ensure!(volume == 0.0, "missing index");
            continue;
        }
        let index = row["index"].as_f64().context("water index")?;
        ensure!(index.is_finite() && index >= 0.0, "water index");
        if index == 0.0 {
            ensure!(volume == 0.0, "missing index");
            continue;
        }
        ensure!(
            days.insert(date.to_owned(), volume).is_none(),
            "duplicate date"
        );
        ensure!(days.len() <= 366, "too many readings");
    }
    Ok(days)
}
pub(super) struct Readings {
    pub(super) meter: String,
    pub(super) days: BTreeMap<String, f64>,
    pub(super) backfill_error: Option<&'static str>,
}

pub(super) async fn fetch(account: &Account, backfill: bool) -> Result<Readings> {
    tokio::time::timeout(
        Duration::from_secs(90),
        fetch_with(&mut Client::new(), account, backfill),
    )
    .await
    .map_err(|_| "water.connection_failed")?
}
async fn fetch_with(client: &mut Client, account: &Account, backfill: bool) -> Result<Readings> {
    let started = tokio::time::Instant::now();
    client.login(account).await?;
    let meter = select_meter(&client.json(METERS).await?, &account.meter)?;
    let today = jiff::Zoned::now().date();
    let first =
        jiff::civil::Date::new(today.year(), today.month(), 1).map_err(|_| "water.data_failed")?;
    let mut days = BTreeMap::new();
    // Fetch older months once, then keep the rolling refresh at three months.
    let months = if backfill {
        (i64::from(first.year()) - 2024) * 12 + i64::from(first.month()) - 10
    } else {
        3
    };
    let mut backfill_error = None;
    for offset in 0..months.max(3) {
        if offset >= 3 && started.elapsed() >= Duration::from_secs(65) {
            backfill_error = Some("water.connection_failed");
            break;
        }
        let start = first
            .checked_sub(jiff::Span::new().months(offset))
            .map_err(|_| "water.data_failed")?;
        let end = if offset == 0 {
            today
        } else {
            start.last_of_month()
        };
        let path = format!(
            "{TELEMETRY}?mode=daily&id_PDS={}&start_date={start}&end_date={end}",
            encode(&meter)
        );
        let chunk = client.json(&path).await.and_then(|reply| {
            let chunk = readings(&reply).map_err(|_| "water.data_failed")?;
            let start = start.to_string();
            let end = end.to_string();
            if chunk.keys().any(|date| date < &start || date > &end) {
                return Err("water.data_failed");
            }
            Ok(chunk)
        });
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) if offset >= 3 => {
                backfill_error = Some(error);
                break;
            }
            Err(error) => return Err(error),
        };
        days.extend(chunk);
    }
    Ok(Readings {
        meter,
        days,
        backfill_error,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[tokio::test]
    async fn a_human_challenge_stops_before_sending_credentials() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = [0; 4096];
            let n = stream.read(&mut bytes).await.unwrap();
            let request = String::from_utf8_lossy(&bytes[..n]);
            assert!(request.starts_with("GET "));
            assert!(!request.contains("synthetic-password"));
            stream
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        });
        let mut client = Client {
            host: "127.0.0.1".into(),
            port,
            tls: false,
            cookies: BTreeMap::new(),
        };
        let account = Account {
            email: "user@example.org".into(),
            password: "synthetic-password".into(),
            meter: String::new(),
        };
        assert_eq!(client.login(&account).await, Err("water.human_required"));
        server.await.unwrap();
    }
    #[tokio::test]
    async fn normal_login_keeps_session_cookies_and_fetches_daily_json_without_following_redirects()
    {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            for step in 0..7 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0; 4096];
                loop {
                    let n = stream.read(&mut buffer).await.unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&buffer[..n]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
                        let length = headers
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length: "))
                            .map_or(0, |n| n.parse::<usize>().unwrap());
                        if bytes.len() >= end + 4 + length {
                            break;
                        }
                    }
                    assert!(bytes.len() < 16_384);
                }
                let request = String::from_utf8(bytes).unwrap();
                let (code, extra, body) = match step {
                    0 => {
                        assert!(request.starts_with(&format!("GET {LOGIN} ")));
                        (
                            "200 OK",
                            "Set-Cookie: SESSION=synthetic; Path=/; HttpOnly\r\n",
                            r"\\u0022csrfToken\\u0022\\u003A\\u0022synthetic-token\\u0022"
                                .to_owned(),
                        )
                    }
                    1 => {
                        assert!(request.starts_with(&format!("POST {LOGIN} ")));
                        assert!(
                            request
                                .to_ascii_lowercase()
                                .contains("cookie: session=synthetic")
                        );
                        assert!(request.contains("tsme_user_login%5B_password%5D=a%2Bb%26c"));
                        assert!(request.contains("_csrf_token=synthetic-token"));
                        // The client must ignore this destination and verify the API on its fixed origin.
                        (
                            "302 Found",
                            "Location: https://other.example.org/never-follow\r\nSet-Cookie: AUTH=synthetic; Path=/\r\n",
                            String::new(),
                        )
                    }
                    2 => {
                        assert!(request.starts_with(&format!("GET {METERS} ")));
                        assert!(request.to_ascii_lowercase().contains("auth=synthetic"));
                        ("200 OK", "", json!({"content":{"nbMeters":1,"clientCompteursPro":[{"compteursPro":[{"idPDS":"123456"}]}]}}).to_string())
                    }
                    6 => ("503 Service Unavailable", "", String::new()),
                    _ => {
                        assert!(
                            request
                                .starts_with(&format!("GET {TELEMETRY}?mode=daily&id_PDS=123456&"))
                        );
                        let date = request
                            .split("start_date=")
                            .nth(1)
                            .unwrap()
                            .split('&')
                            .next()
                            .unwrap();
                        ("200 OK", "", json!({"content":{"measures":[{"date":date,"index":123.0,"volume":0.125}]}}).to_string())
                    }
                };
                let response = format!(
                    "HTTP/1.1 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let mut client = Client {
            host: "127.0.0.1".into(),
            port,
            tls: false,
            cookies: BTreeMap::new(),
        };
        let account = Account {
            email: "user@example.org".into(),
            password: "a+b&c".into(),
            meter: String::new(),
        };
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            fetch_with(&mut client, &account, true),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(result.meter, "123456");
        assert_eq!(result.days.len(), 3);
        assert_eq!(result.backfill_error, Some("water.connection_failed"));
        server.await.unwrap();
    }
    #[test]
    fn csrf_and_form_values_are_not_confused_with_html_or_form_syntax() {
        assert_eq!(
            csrf(r"prefix \\u0022csrfToken\\u0022\\u003A\\u0022abc.def-123\\u0022 end").unwrap(),
            "abc.def-123"
        );
        assert!(csrf(r"\u0022csrfToken\u0022\u003A\u0022bad&token\u0022").is_err());
        assert_eq!(encode("a+b&é="), "a%2Bb%26%C3%A9%3D");
    }
    #[test]
    fn meter_selection_is_unambiguous_and_belongs_to_the_account() {
        let data =
            json!({"clientCompteursPro":[{"compteursPro":[{"idPDS":"123456"},{"idPDS":987_654}]}]});
        assert!(select_meter(&data, "").is_err());
        assert!(select_meter(&data, "unknown").is_err());
        assert_eq!(select_meter(&data, "987654").unwrap(), "987654");
    }
    #[test]
    fn unknown_days_do_not_overwrite_zero_measurements_and_bad_batches_fail() {
        let good = json!({"measures":[{"date":"2026-11-01 00:00:00","volume":0.0,"index":123.0},{"date":"2026-11-02","volume":0.0,"index":null}]});
        assert_eq!(
            readings(&good).unwrap(),
            BTreeMap::from([("2026-11-01".into(), 0.0)])
        );
        for bad in [
            json!({"date":"2026-02-30","volume":1,"index":123}),
            json!({"date":"2026-11-01","volume":-1,"index":123}),
            json!({"date":"2026-11-01","volume":1,"index":null}),
        ] {
            assert!(readings(&json!({"measures":[bad]})).is_err());
        }
    }
}
