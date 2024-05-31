pub mod crystal;

use actix_files::Files;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{
    middleware, web::{self, resource, Data, Redirect}, App, HttpRequest, HttpServer, Responder, Result
};
use actix_web_lab::respond::Html;
use askama::Template;
use crystal::{Crystarium, FileStructure, Node};
use std::{sync::Mutex, vec::Vec};
use std::{
    collections::HashMap,
    fs::{self},
    io::*
};

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

#[derive(Template)]
#[template(path = "node_edit.html")]
struct NodeViewer {
    nodes: Vec<NodeFragment>,
}

#[derive(Template, Debug)]
#[template(path = "node.html")]
struct NodeFragment {
    char: String,
    name: String,
    cost: i32,
    r#type: String,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[derive(Clone, Debug, Default)]
struct CrystalData {
    crystal_data: Crystarium,
}

trait ConvertVecNode {
    fn convert(self) -> Vec<NodeFragment>;
}

impl ConvertVecNode for Vec<Node> {
    fn convert(self) -> Vec<NodeFragment> {
        self.into_iter().map(NodeFragment::from).collect()
    }
}

impl From<Node> for NodeFragment {
    fn from(value: Node) -> Self {
        NodeFragment {
            char: value.char_name,
            name: value.node_name,
            cost: value.cp_cost,
            r#type: value.node_type.to_string(),
        }
    }
}

async fn node_viewer(data: web::Data<Mutex<CrystalData>>) -> Result<impl Responder> {
    let nodes:Vec<NodeFragment> = data.lock().unwrap().crystal_data.nodes.clone().convert();
    log::info!("got Node Viewer");
    log::info!("{:?}", nodes);
    let html = Html(NodeViewer{ nodes: nodes }.render().unwrap()); 
    Ok(html)
}

async fn index(_: web::Query<HashMap<String, String>>) -> Result<impl Responder> {
    log::info!("got Index");
    Ok(Html(Index.render().unwrap()))
}

async fn upload(req: HttpRequest ,mut form: MultipartForm<UploadForm>) -> Result<impl Responder> {
    log::info!("got Upload");
    
    //Mutex lock and crystal data prepare for file write
    let mg_crystal_data = req.app_data::<Data<Mutex<CrystalData>>>().unwrap(); //data.lock().unwrap().crystal_data.to_owned();
    let mut crystal_data = mg_crystal_data.lock().unwrap();

    //Get first uploaded file.
    let f = form.files.first_mut().unwrap();

    log::info!("Filename {}", f.file_name.as_ref().unwrap());

    //Declare buffer for file's content
    let mut data: Vec<u8> = Vec::new();

    //Read content to buffer
    f.file.read_to_end(&mut data).unwrap();

    //Parse crystal data
    crystal_data.crystal_data = read_crystal_wdb(data).unwrap().clone();

    Ok(Redirect::to("/node_viewer").see_other())
    // Ok(HttpResponse::Ok())

    // Ok(HttpResponse::Found()
    //     .append_header(("Location", "/node_viewer"))
    //     .body(
    //         NodeViewer{ 
    //             nodes: crystal_data.crystal_data.nodes.clone().convert() }
    //         .render()
    //         .unwrap()
    //     )
    // )
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
    .await.expect("Error Server Listening");

    if 1 == 2{
        read_crystal_wdb_with_file("test").unwrap();
    }

    Ok(())
}
fn read_crystal_wdb_with_file(path: &str) -> Result<Crystarium> {
    let file_h = fs::File::open(path)?;

    //Buffered Reader for file
    let mut b_reader = BufReader::new(file_h);

    //File Structure Mapping
    let fstruct = FileStructure::load(&mut b_reader);

    //Using the file structure to get the data for crystal infos.
    let crystarium = Crystarium::create(&mut b_reader, &fstruct);

    Ok(crystarium)
}

fn read_crystal_wdb(data: Vec<u8>) -> Result<Crystarium> {
    //Buffered Reader for file
    let mut b_cursor = Cursor::new(data);

    //File Structure Mapping
    let fstruct = FileStructure::load(&mut b_cursor);

    //Using the file structure to get the data for crystal infos.
    let crystarium = Crystarium::create(&mut b_cursor, &fstruct);

    Ok(crystarium)
}

#[test]
fn test_read_crystal() {}
