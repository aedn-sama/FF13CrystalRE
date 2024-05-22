pub mod crystal;
use actix_files::Files;
use actix_web_lab::respond::Html;
use crystal::{Crystarium, FileStructure, Node};
use log::info;
use std::{collections::HashMap, default, fs, io::*, vec};

use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder, Result};
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

// impl From<Vec<NodeTest>> for Vec<NodeFragment>{
//     fn from(value: Vec<NodeTest>) -> Self {
//         NodeFragment{ char: value.char_name, name: value.node_name, cost: value.cp_cost, r#type: value.node_type.to_string() }
//         vec![]
//     }
// }

async fn index(query: web::Query<HashMap<String, String>>) -> Result<impl Responder> {
    let crystarium = Crystarium::default();
    let mut nodes: Vec<NodeFragment> = vec![];
    
    nodes = crystarium.nodes.convert();

    // let _ = crystarium.nodes
    //     .iter()
    //     .for_each(|node| nodes.push(NodeFragment::from(node.clone())
    //                                     // NodeFragment{
    //                                         // char: node.char_name.clone(),
    //                                         // name: node.node_name.clone(), 
    //                                         // cost: node.cp_cost, 
    //                                         // r#type: node.node_type.to_string()}
    //                                     ));


    Ok(Html(Index{nodes: nodes}.render().unwrap()))
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {

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
    if read_crystal_wdb("C:\\Users\\adria\\Documents\\crystal_fang.wdb").is_ok(){

    }
}
