use actix_web::{middleware, App, HttpServer};
use ya_activity::requestor;
use ya_persistence::executor::DbExecutor;
use ya_persistence::migrations;
use ya_service_api::constants::{ACTIVITY_API, YAGNA_HTTP_ADDR};

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = DbExecutor::new(":memory:")?;
    migrations::run_with_output(&db.conn()?, &mut std::io::stdout())?;

    HttpServer::new(move || {
        let mut activity = actix_web::web::scope(ACTIVITY_API).data(db.clone());
        activity = requestor::control::extend_web_scope(activity);
        activity = requestor::state::extend_web_scope(activity);

        App::new()
            .wrap(middleware::Logger::default())
            .service(activity)
    })
    .bind(*YAGNA_HTTP_ADDR)?
    .run()
    .await?;

    Ok(())
}
