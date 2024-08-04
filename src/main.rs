use chrono::{DateTime, Utc};
use chrono_tz::Pacific::Auckland;
use chrono_tz::Tz;
use surf::http::headers::HeaderValues;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use tide::{log, Request};
use tide::{prelude::*, Response};

#[async_std::main]
async fn main() -> tide::Result<()> {
    log::start();
    let mut app = tide::new();

    app.at("/hello").get(hello);
    app.at("/rates").get(rates);
    app.at("/rates/tonight").get(tonights_rates);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);
    app.with(cors);

    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(default)]
struct HelloQuery {
    name: String,
}
impl Default for HelloQuery {
    fn default() -> Self {
        Self {
            name: "world".to_owned(),
        }
    }
}

async fn hello(request: Request<()>) -> tide::Result {
    log_request_origin(&request);

    let query: HelloQuery = request.query()?;
    let reply = format!("Hello, {}!", query.name);
    Ok(reply.into())
}

#[derive(Deserialize, Serialize)]
struct LittleHotelierRates {
    name: String,
    rate_plans: Vec<RatePlan>,
}

#[derive(Deserialize, Serialize)]
struct RatePlan {
    id: u32,
    name: String,
    rate_plan_dates: Vec<RatePlanDate>,
}

#[derive(Deserialize, Serialize)]
struct RatePlanDate {
    id: Option<u32>,
    date: String,
    rate: u16,
    min_stay: u8,
    stop_online_sell: bool,
    close_to_arrival: bool,
    close_to_departure: bool,
    max_stay: Option<u8>,
    available: u8,
}

#[derive(Deserialize, Serialize)]
struct LodgeRate {
    name: String,
    rate: u16,
    num_available: u8,
}

const LITTLE_HOTELIER_BASE_URL: &str =
    "https://apac.littlehotelier.com/api/v1/properties/kakapolodgedirect/rates.json";

#[derive(Deserialize)]
#[serde(default)]
struct RatesQuery {
    start_date: String,
    end_date: String,
}
impl Default for RatesQuery {
    fn default() -> Self {
        Self {
            start_date: "".to_owned(),
            end_date: "".to_owned(),
        }
    }
}
async fn rates(request: Request<()>) -> tide::Result {
    log_request_origin(&request);

    let query: RatesQuery = request.query()?;

    let little_hotelier_url = format!(
        "{}?start_date={}&end_date={}",
        LITTLE_HOTELIER_BASE_URL, query.start_date, query.end_date
    );

    let little_hotelier_response: Vec<LittleHotelierRates> =
        surf::get(little_hotelier_url).recv_json().await?;

    let little_hotelier_rates = little_hotelier_response.first().unwrap();

    let response = Response::builder(200)
        .body(json!(&little_hotelier_rates.rate_plans))
        .build();

    Ok(response)
}

async fn tonights_rates(request: Request<()>) -> tide::Result {
    log_request_origin(&request);

    let today = get_todays_date();
    let todays_date = convert_date_to_rfc3339_string(&today);
    log::info!("today's date: {}", todays_date);

    let little_hotelier_url = format!(
        "{}?start_date={}&end_date={}",
        LITTLE_HOTELIER_BASE_URL, todays_date, todays_date
    );
    log::info!("url to call: {}", little_hotelier_url);

    let little_hotelier_response: Vec<LittleHotelierRates> =
        surf::get(little_hotelier_url).recv_json().await?;

    let little_hotelier_rates = little_hotelier_response.first().unwrap();

    log::info!("got response from Little Hotelier");

    let lodge_rates = map_tonights_rates(little_hotelier_rates);
    let response_body = json!(lodge_rates);

    let response = Response::builder(200).body(response_body).build();
    Ok(response)
}

fn get_todays_date() -> DateTime<Tz> {
    Utc::now().with_timezone(&Auckland)
}

fn convert_date_to_rfc3339_string(date: &DateTime<Tz>) -> String {
    date.to_rfc3339()
        .split('T')
        .map(|string_slice| string_slice.to_owned())
        .collect::<Vec<_>>()
        .first()
        .unwrap_or(&String::from(""))
        .to_owned()
}

fn map_tonights_rates(little_hotelier_rates: &LittleHotelierRates) -> Vec<LodgeRate> {
    let rate_plans = &little_hotelier_rates.rate_plans;

    let rates = rate_plans
        .into_iter()
        .map(|rate_plan| map_rate_plan_to_lodge_rate(rate_plan))
        .collect();

    rates
}

fn map_rate_plan_to_lodge_rate(rate_plan: &RatePlan) -> LodgeRate {
    let rate_plan_date = rate_plan.rate_plan_dates.first().unwrap();

    LodgeRate {
        name: rate_plan.name.to_owned(),
        rate: rate_plan_date.rate,
        num_available: rate_plan_date.available,
    }
}

fn log_request_origin(request: &Request<()>) {
    let default_origin = HeaderValues::from_iter([]);
    let request_origin = request.header("Origin").unwrap_or(&default_origin);
    log::info!("Request origin: {:?}", request_origin);
}
