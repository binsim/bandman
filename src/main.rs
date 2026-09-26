#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use bandman::app::{shell, App};
    use bandman::db::connect_and_migrate;
    use bandman::state::AppState;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::PgPool;

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let pool = connect_and_migrate()
        .await
        .expect("failed to connect to database or run migrations");

    let state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &state,
            routes,
            {
                let pool = pool.clone();
                move || provide_context::<PgPool>(pool.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // Client hydration entry is in lib.rs.
}
