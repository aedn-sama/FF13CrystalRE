use crate::crystal::*;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;


#[derive(Template)]
#[template(path = "node_edit.html")]
pub struct NodeViewer {
    pub current_page: i32,
    pub next_page: i32,
    pub prev_page: i32,
    pub nodes: Vec<NodeFragment>,
}

#[derive(Clone, Debug)]
pub struct NodeFragment {
    pub char: String,
    pub name: String,
    pub cost: i32,
    pub image: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(rename = "file")]
    pub files: Vec<TempFile>,
}

#[derive(Clone, Debug, Default)]
pub struct CrystalData {
    pub crystal_data: Crystarium,
}

pub trait ConvertVecNode {
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
            image: value.node_type.to_imagesrc().to_string()
        }
    }
}