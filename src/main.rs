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
    http::header, middleware, web::{self, resource, Data}, App, HttpRequest, HttpResponse, HttpServer, Responder, Result
};
use actix_web_lab::respond::Html;

use askama::Template;

use std::{ io::Read, ops::{Deref, DerefMut}, sync::{Arc, Mutex, MutexGuard}, vec::Vec};

use lazy_static::lazy_static;

// Define a static variable using lazy_static
lazy_static! {
    static ref VIEWER_PAGES: Mutex<Arc<Vec<CrystalPage>>> = Mutex::new(Arc::new(Vec::<CrystalPage>::new()));
}

async fn node_viewer(
    req: HttpRequest,
    data: web::Data<Mutex<CrystalData>>,
) -> Result<impl Responder> {
    log::info!("got Node Viewer");

    //first check if static variable has data, if not, get from mutex.
    let mut guard_pages = VIEWER_PAGES.lock().unwrap();
    let mut paged_nodes: Vec<CrystalPage> = guard_pages.as_ref().clone();
    let mut nodes: Vec<NodeFragment> = Vec::default();

    if paged_nodes.is_empty(){
        nodes = data.lock().unwrap().crystal_data.nodes.clone().convert();

        if nodes.is_empty() {
            return Ok(HttpResponse::PermanentRedirect().append_header((header::LOCATION,"/")).finish());
        }

        *guard_pages = Arc::new(CrystalPage::convert(&mut nodes).clone());
        paged_nodes = guard_pages.to_owned().to_vec();
    }
    
    //extract query data - page
    let query = req
        .query_string()
        .to_string()
        .split("page=")
        .last()
        .unwrap()
        .to_string();


    // Can be simplified with clamp
    // let fn_next_page = |page:i32, max_page: i32| if page > max_page { max_page } else { page };
    // let fn_prev_page = |page:i32| if page < 1 { 1 } else { page };
    let page = query.parse::<i32>().unwrap_or(1); //page=i32
    
    let max_page = paged_nodes.iter()
        .max_by_key(|f| f.page)
        .unwrap()
        .page;

    //Set and check next page. If page would be 10, then next_page would be 11 without this bounds_check
    let next_page = (page + 1).clamp(1, max_page);
    let prev_page = (page - 1).clamp(1, max_page);

    let paged_node = paged_nodes
        .iter()
        .find(|r| r.page == page.clamp(1, max_page));
    
    //If node found, then display, else give bad response.
    match paged_node {
        Some(paged_node) => {
            let response = HttpResponse::Ok().body(
                NodeViewer {
                    current_page: paged_node.page,
                    prev_page: prev_page,
                    next_page: next_page,
                    nodes: paged_node.nodes.clone(),
                }
                .render()
                .unwrap(),
            ).into();

            return Ok(response);
        }
        None => {
            return Ok(HttpResponse::Ok().finish());
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

    Ok(HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/node_viewer"))
        .finish())
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
