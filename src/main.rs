pub mod crystal;
use actix_files::Files;
use actix_web_lab::respond::Html;
use crystal::{Crystarium, FileStructure, Node};
use std::{collections::HashMap, fs::{self}, io::*, vec};
use actix_web::{middleware, web::{self, resource}, App, HttpResponse, HttpServer, Responder, Result};
use askama::Template;
use std::vec::Vec;

#[derive(Template)]
#[template(path = "index.html")]
struct Index{
    nodes: Vec<NodeFragment>
}

#[derive(Template, Debug)]
#[template(path = "node.html")]
struct NodeFragment{
    char: String,
    name: String,
    cost: i32,
    r#type: String
}

trait ConvertVecNode {
    fn convert(self) -> Vec<NodeFragment>;
}

impl ConvertVecNode for Vec<Node> {
    fn convert(self) -> Vec<NodeFragment> {
        self.into_iter().map(NodeFragment::from).collect()
    }
}

impl From<Node> for NodeFragment{
    fn from(value: Node) -> Self {
        NodeFragment{ char: value.char_name, name: value.node_name, cost: value.cp_cost, r#type: value.node_type.to_string() }
    }
}

async fn index(_: web::Query<HashMap<String, String>>) -> Result<impl Responder> {
    let crystarium = Crystarium::default();
    let mut nodes: Vec<NodeFragment> = vec![];
    
    log::info!("got Index");

    nodes = crystarium.nodes.convert();

    Ok(Html(Index{nodes: nodes}.render().unwrap()))
}

async fn upload(payload: web::Payload) -> Result<impl Responder>{
    log::info!("got Upload");
    println!("got Upload");
    read_crystal_wdb(Vec::from(payload.to_bytes().await.unwrap())).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://127.0.0.1:8000");

    HttpServer::new( move || {
            App::new()
                .wrap(middleware::Logger::default())
                .service(resource("/upload").route(web::post().to(upload)))
                .service(resource("/").route(web::get().to(index)))
                .service(Files::new("/assets", "./templates/assets"))
        })
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
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

fn read_crystal_wdb(data: Vec<u8> ) -> Result<Crystarium> {
    //Buffered Reader for file
    let mut b_cursor = Cursor::new(data);

    //File Structure Mapping
    let fstruct = FileStructure::load(&mut b_cursor);

    //Using the file structure to get the data for crystal infos.
    let crystarium = Crystarium::create(&mut b_cursor, &fstruct);

    Ok(crystarium)
}



#[test]
fn test_read_crystal() {

}
