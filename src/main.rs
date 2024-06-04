pub mod crystal;
pub mod crystal_page;
pub mod view;

use crystal::read_crystal_wdb;
use crystal_page::CrystalPage;
// use log::info;
use view::{ConvertVecNode, CrystalData, Index, NodeFragment, NodeViewer, UploadForm};

use actix_files::Files;
use actix_multipart::form::MultipartForm;
use actix_web::{
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    middleware,
    web::{self, resource, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_lab::respond::Html;

use askama::Template;

use std::{ io::Read, sync::Mutex, vec::Vec};

async fn node_viewer(
    req: HttpRequest,
    data: web::Data<Mutex<CrystalData>>,
) -> Result<impl Responder> {
    log::info!("got Node Viewer");

    //get and convert crystal data from mutex
    let mut nodes: Vec<NodeFragment> = data.lock().unwrap().crystal_data.nodes.clone().convert();

    if nodes.is_empty() {
        let mut response = HttpResponse::with_body(
            StatusCode::PERMANENT_REDIRECT,
            "No Nodes from file".to_string(),
        );
        let headers = &mut response.head_mut().headers;
        headers.append(header::LOCATION, HeaderValue::from_str("/").unwrap());
        return Ok(response);
    }

    let paged_nodes = CrystalPage::convert(&mut nodes);

    //extract query data - page
    let query = req
        .query_string()
        .to_string()
        .split("page=")
        .last()
        .unwrap()
        .to_string();


    let fn_bounds_page = |page:i32, max_page: i32| if page > max_page { max_page } else { page };
    let page = query.parse::<i32>().unwrap_or(1); //page=i32
    
    let max_page = paged_nodes.iter()
        .max_by_key(|f| f.page)
        .unwrap()
        .page;

    let paged_node = paged_nodes
        .iter()
        .find(|r| r.page == fn_bounds_page(page, max_page));
    
    //If node found, then display, else give bad response.
    match paged_node {
        Some(paged_node) => {
            let response = HttpResponse::new(StatusCode::FOUND).set_body(
                NodeViewer {
                    current_page: paged_node.page,
                    prev_page: (paged_node.page - 1).abs(),
                    next_page: (paged_node.page + 1),
                    nodes: paged_node.nodes.clone(),
                }
                .render()
                .unwrap(),
            );

            // let headers = response.borrow_mut().headers_mut();
            // headers.append(LOCATION, HeaderValue::from_str(link).unwrap());
            return Ok(response);
        }
        None => {
            return Ok(HttpResponse::with_body(
                StatusCode::OK,
                "No more nodes".to_string(),
            ));
        }
    }
}

async fn index(_req: HttpRequest) -> Result<impl Responder> {
    log::info!("got Index");

    Ok(Html(Index.render().unwrap()))
}

async fn upload(req: HttpRequest, mut form: MultipartForm<UploadForm>) -> Result<impl Responder> {
    log::info!("got Upload");

    //Mutex lock and crystal data prepare for file write
    let mg_crystal_data = req.app_data::<Data<Mutex<CrystalData>>>().unwrap(); //data.lock().unwrap().crystal_data.to_owned();
    let mut crystal_data = mg_crystal_data.lock().unwrap();

    //Get first uploaded file.
    let f = form.files.first_mut();

    //if no file given, then respond with status code 415
    if f.is_none() {
        return Ok(HttpResponse::UnsupportedMediaType().finish());
    }

    let f = f.unwrap();

    log::info!("Filename {}", f.file_name.as_ref().unwrap());

    //Declare buffer for file's content
    let mut data: Vec<u8> = Vec::new();

    //Read content to buffer
    f.file.read_to_end(&mut data).unwrap();

    //Parse crystal data
    crystal_data.crystal_data = read_crystal_wdb(data).unwrap().clone();

    // Ok(Redirect::to("/node_viewer").see_other())
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://127.0.0.1:8000");

    let crystal_data = web::Data::new(Mutex::new(CrystalData::default()));

    HttpServer::new(move || {
        App::new()
            .app_data(crystal_data.clone())
            .wrap(middleware::Logger::default())
            .service(resource("/upload").route(web::post().to(upload)))
            .service(resource("/node_viewer").route(web::get().to(node_viewer)))
            .service(resource("/").route(web::get().to(index)))
            .service(Files::new("/assets", "./templates/assets"))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
    .expect("Error Server Listening");

    Ok(())
}
