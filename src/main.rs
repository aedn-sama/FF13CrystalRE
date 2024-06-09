pub mod crystal;
pub mod crystal_page;
pub mod view;

use crystal::{read_crystal_wdb, Crystarium};
use crystal_page::CrystalPage;
// use log::info;
use view::{ConvertVecNode, CrystalData, Index, NodeFragment, NodeViewer, UploadForm};

use actix_files::Files;
use actix_multipart::form::MultipartForm;
use actix_web::{
    guard, http::header, middleware, web::{self, resource, Data}, App, HttpRequest, HttpResponse, HttpServer, Responder, Result
};
use askama::Template;

use std::{
    io::Read, ops::Deref, sync::{Arc, Mutex}, vec::Vec
};

use lazy_static::lazy_static;

// Define a static variable using lazy_static
lazy_static! {
    static ref VIEWER_PAGES: Mutex<Arc<Vec<CrystalPage>>> =
        Mutex::new(Arc::new(Vec::<CrystalPage>::new()));
}

async fn node_viewer(
    req: HttpRequest,
    data: web::Data<Mutex<CrystalData>>,
) -> Result<impl Responder> {
    log::info!("got Node Viewer");

    //first check if static variable has data, if not, get from mutex.
    let mut guard_pages = VIEWER_PAGES.lock().unwrap();
    let mut paged_nodes: Vec<CrystalPage> = guard_pages.as_ref().clone();

    if paged_nodes.is_empty() {
        let guard_crystal_data = data.lock().unwrap();
        let mut nodes: Vec<NodeFragment> = guard_crystal_data.crystal_data.nodes.clone().convert();

        if nodes.is_empty() {
            return Ok(HttpResponse::PermanentRedirect()
                .append_header(("Location", "/"))
                .insert_header((header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"))
                .insert_header((header::PRAGMA, "no-cache"))
                .insert_header((header::EXPIRES, "0"))
                .finish());
        }

        *guard_pages = Arc::new(
            CrystalPage::convert(&guard_crystal_data.crystal_data.character, &mut nodes).clone(),
        );
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

    let page = query.parse::<i16>().unwrap_or(1); //page=i32

    let max_page = paged_nodes.iter().max_by_key(|f| f.stage).unwrap().stage;

    //Set and check next page. If page would be 10, then next_page would be 11 without this bounds_check
    let next_page = (page + 1).clamp(1, max_page);
    let prev_page = (page - 1).clamp(1, max_page);

    let paged_node = paged_nodes
        .iter()
        .find(|r| r.stage == page.clamp(1, max_page));

    //If node found, then display, else give bad response.
    match paged_node {
        Some(paged_node) => {
            let response = HttpResponse::Ok()
                .insert_header((header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"))
                .insert_header((header::PRAGMA, "no-cache"))
                .insert_header((header::EXPIRES, "0"))
                .body(
                    NodeViewer {
                        character: paged_node.character.clone(),
                        current_page: paged_node.stage,
                        prev_page: prev_page,
                        next_page: next_page,
                        roles: paged_node.roles.clone(),
                    }
                    .render()
                    .unwrap(),
                )
                .into();

            return Ok(response);
        }
        None => {
            return Ok(HttpResponse::Ok().finish());
        }
    }
}

async fn index(_req: HttpRequest) -> Result<impl Responder> {
    log::info!("got Index");
    Ok(Into::<HttpResponse>::into(
        HttpResponse::Ok().body(Index.render().unwrap()),
    ))
}

async fn upload(req: HttpRequest, mut form: MultipartForm<UploadForm>) -> Result<impl Responder> {
    log::info!("got Upload");

    //Mutex lock and crystal data prepare for file write
    let mg_crystal_data = req.app_data::<Data<Mutex<CrystalData>>>().unwrap(); //data.lock().unwrap().crystal_data.to_owned();
    let mut crystal_data = mg_crystal_data.lock().unwrap();
    
    //Initialize data at upload
    crystal_data.crystal_data = Crystarium::default();
    let mut guard_pages = VIEWER_PAGES.lock().unwrap();
    let pages_arc = Arc::make_mut(&mut *guard_pages); // Arc mutably dereferenzieren
    pages_arc.clear(); // Den Vektor leeren

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
    // let dbg_file = File::create("debug_contents.txt");
    // dbg_file.unwrap().write_all(format!("{:?}",crystal_data.crystal_data.clone().nodes).as_bytes());

    Ok(HttpResponse::Ok()
        .insert_header(("HX-Redirect", "/node_viewer?page=1"))
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
