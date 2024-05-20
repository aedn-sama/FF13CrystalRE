pub mod crystal;
use actix_files::Files;
use actix_web_lab::respond::Html;
use crystal::{Crystarium, FileStructure, Node};
use log::info;
use std::{collections::HashMap, default, fs, io::*, vec};

use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder, Result};
use askama::Template;

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

async fn index(query: web::Query<HashMap<String, String>>) -> Result<impl Responder> {
    let crystarium = read_crystal_wdb("C:\\Users\\adria\\Documents\\crystal_fang.wdb").expect("No Crystarium created");
    let mut nodes: Vec<NodeFragment> = vec![];
    
    let _ = crystarium.nodes
        .iter()
        .for_each(|node| nodes.push(NodeFragment{
                                            char: node.char_name.clone(),
                                            name: node.node_name.clone(), 
                                            cost: node.cp_cost, 
                                            r#type: node.node_type.to_string()
                                        }));


    Ok(Html(Index{nodes: nodes}.render().unwrap()))
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Darf man nicht in Kombi mit wasm machen, WebAssembly mit System native Funktionen.
    let crystarium = read_crystal_wdb("C:\\Users\\adria\\Documents\\crystal_fang.wdb").expect("No Crystarium created");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new( move || {
            App::new()
                .wrap(middleware::Logger::default())
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/assets", "./templates/assets"))
        })
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
    //println!("{:?}", read_crystal_wdb("C:\\Users\\adria\\Documents\\crystal_fang.wdb"));
}

fn read_crystal_wdb(path: &str) -> Result<Crystarium> {
    let file_h = fs::File::open(path)?;

    //Buffered Reader for file
    let mut b_reader = BufReader::new(file_h);

    //File Structure Mapping
    let fstruct = FileStructure::load(&mut b_reader);

    //Using the file structure to get the data for crystal infos.
    let crystarium = Crystarium::create(&mut b_reader, &fstruct);

    Ok(crystarium)
}

#[test]
fn test_read_crystal() {
    read_crystal_wdb("C:\\Users\\adria\\Documents\\crystal_fang.wdb").expect("Expecteded Ok");
}
