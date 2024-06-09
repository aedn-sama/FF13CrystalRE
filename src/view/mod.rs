use std::collections::HashMap;

use crate::crystal::*;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template, Default)]
#[template(path = "node_edit.html")]
pub struct NodeViewer {
    pub character: String,
    pub current_page: i16,
    pub next_page: i16,
    pub prev_page: i16,
    pub roles: Vec<RoleFragment>,
}

#[derive(Clone, Debug, Default)]
pub struct RoleFragment {
    pub name: String,
    pub nodes: Vec<NodeFragment>,
}

#[derive(Clone, Debug)]
pub struct NodeFragment {
    pub name: String,
    pub cost: i32,
    pub value: i16,
    pub role: String,
    pub stage: i16,
    pub r#type: String,
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
            name: value.node_name,
            cost: value.cp_cost,
            value: value.node_value,
            stage: value.stage.into(),
            role: NodeRole::from(value.role).to_string(),
            r#type: value.node_type.to_string(),
            image: value.node_type.to_imagesrc().to_string(),
        }
    }
}

#[derive(Default)]
pub struct ListRoleFragment(pub Vec<RoleFragment>);

impl From<Vec<NodeFragment>> for ListRoleFragment {
    fn from(node_fragments: Vec<NodeFragment>) -> ListRoleFragment {
        let mut hm_fragments: HashMap<String, Vec<NodeFragment>> = HashMap::new();
        let mut nodes = Vec::<NodeFragment>::new();

        for node in node_fragments {
            //do we already have entries with the specific page?
            if hm_fragments.contains_key(&node.role) {
                let hm_fragments_nodes = hm_fragments.get_mut(&node.role).unwrap();
                hm_fragments_nodes.push(node.clone());
            } else {
                //insert page with first node
                nodes.push(node.clone());
                hm_fragments.insert(node.role, nodes.clone());
            }
        }

         //sort the entries by page num
         let mut sorted_rnodes: Vec<_> = hm_fragments.into_iter().collect();
         sorted_rnodes.sort_by_key(|k| k.0.clone());
 
         //create crystal pages from hash map
         let mut role_fragments = Vec::<RoleFragment>::new();
 
         for rnode in sorted_rnodes {
             let mut role_fragment = RoleFragment::default();
             role_fragment.name = rnode.0;
             role_fragment.nodes = rnode.1;
             role_fragments.push(role_fragment);
         }
        // let role_fragment: Vec<RoleFragment> = Vec::new();

        // for fragment in node_fragments {
        //     role_fragment.push(RoleFragment{})
        // }
        ListRoleFragment(role_fragments)
    }
}
