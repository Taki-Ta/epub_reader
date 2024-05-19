use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use epub::doc::{EpubDoc, NavPoint};
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug, Clone)]
struct FileShare {
    doc: Arc<Mutex<EpubDoc<std::io::BufReader<std::fs::File>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let doc = EpubDoc::new("assets/book.epub")?;
    let file_share = FileShare {
        doc: Arc::new(Mutex::new(doc)),
    };
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8081));
    let app = Router::new()
        .route("/", get(handle_index))
        .route("/epub/*path", get(handle_content))
        .layer(Extension(file_share));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    info!("Listening on: {}", listener.local_addr().unwrap());
    match axum::serve(listener, app).await {
        Ok(_) => info!("Server stopped normally"),
        Err(e) => eprintln!("Server stopped with error: {:?}", e),
    }
    Ok(())
}

async fn handle_index(Extension(state): Extension<FileShare>) -> (StatusCode, Response) {
    let doc = state.doc.lock().unwrap();
    let list = get_toc_list(doc.toc.clone());
    let mut html = String::new();
    html.push_str("<html><head><title>Table of Contents</title></head><body>");
    html.push_str("<h1>Table of Contents</h1><ul>");
    for nav in list {
        html.push_str(&format!(
            "<li><a href=\"/epub/{}\">{}</a></li></ul></body></html>",
            nav.content.display(),
            nav.label
        ));
    }
    (StatusCode::OK, Html(html).into_response())
}

async fn handle_content(
    Extension(state): Extension<FileShare>,
    Path(path): Path<String>,
) -> (StatusCode, Response) {
    info!("path: {}", path);
    let mut doc = state.doc.lock().unwrap();
    let path = std::path::Path::new(&path);
    let content = doc.get_resource_by_path(path).unwrap();
    let content = String::from_utf8_lossy(content.as_slice()).to_string();
    (StatusCode::OK, Html(content).into_response())
}

//may return a list of &NavPoint
#[allow(dead_code)]
fn get_toc_list(toc: Vec<NavPoint>) -> Vec<NavPoint> {
    let mut toc_list = Vec::new();
    for nav in toc {
        if !nav.children.is_empty() {
            toc_list.push(nav.clone());
            let children = get_toc_list(nav.children);
            for child in children {
                toc_list.push(child);
            }
        } else {
            toc_list.push(nav.clone());
        }
    }
    //order by playOrder
    toc_list.sort_by(|a, b| a.play_order.cmp(&b.play_order));
    toc_list
}
