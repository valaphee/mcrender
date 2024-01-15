use crate::Renderer;

#[derive(serde::Serialize, serde::Deserialize)]
struct Query {
    frame: Frame,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum Frame {
    Task,
    Challenge,
    Goal,
}

#[actix_web::get("/item/{namespace}/{key}.png")]
async fn get(
    renderer: actix_web::web::Data<Renderer>,
    _path: actix_web::web::Path<(String, String)>,
    _query: actix_web::web::Query<Query>,
) -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok()
        .content_type("image/png")
        .body(vec![])
}
