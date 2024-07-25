use tide::prelude::*;
use tide::Request;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/hello").get(hello);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(default)]
struct Query {
    name: String,
}
impl Default for Query {
    fn default() -> Self {
        Self {
            name: "world".to_owned(),
        }
    }
}

async fn hello(request: Request<()>) -> tide::Result {
    let query: Query = request.query()?;
    let reply = format!("Hello, {}!", query.name);
    Ok(reply.into())
}
