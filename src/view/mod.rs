use crate::crystal::*;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;


#[derive(Template)]
#[template(path = "node_edit.html")]
pub struct NodeViewer {
    pub nodes: Vec<NodeFragment>,
}

#[derive(Clone, Debug)]
pub struct NodeFragment {
    pub char: String,
    pub name: String,
    pub cost: i32,
    pub r#type: String,
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
            r#type: value.node_type.to_string(),
        }
    }
}