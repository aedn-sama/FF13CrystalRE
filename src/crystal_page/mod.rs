use crate::view::NodeFragment;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct CrystalPage{
    pub page: i32,
    pub node_count: usize,
    pub nodes: Vec<NodeFragment> 
}


impl CrystalPage{
    pub fn convert(node_fragments :&mut Vec<NodeFragment>) -> Vec<Self>{
        //hashmap of node fragnebts per page
        let mut hm_pnode_fragments = HashMap::<i32, Vec<NodeFragment>>::new();

        for node in node_fragments {
            //get crystal name level as page
            let page = node.name[7..9].parse::<i32>().unwrap();
            let mut nodes = Vec::<NodeFragment>::new();
            
            //do we already have entries with the specific page?
            if hm_pnode_fragments.contains_key(&page){
                //get mutable crystal page from hashed table and append node
                let pnode_fragments = hm_pnode_fragments.get_mut(&page).unwrap();
                pnode_fragments.push(node.clone());
            
            } else {
                //insert page with first node
                nodes.push(node.clone());
                hm_pnode_fragments.insert(page,  nodes);
            }
        }
        

        //sort the entries by page num
        let mut sorted_pnodes: Vec<_> = hm_pnode_fragments.into_iter().collect();
        sorted_pnodes.sort_by_key(|k| k.0.abs());

        //create crystal pages from hash map
        let mut crystal_pages = Vec::<CrystalPage>::new();
        
        for pnode in sorted_pnodes {
            let mut crystal_page = CrystalPage::default();
            crystal_page.page = pnode.0;
            crystal_page.node_count = pnode.1.len();
            crystal_page.nodes = pnode.1;
            crystal_pages.push(crystal_page);
        }
        
        crystal_pages
    }
}

#[test]
fn test_convert(){
    let node_fragments = [
        NodeFragment{name: "cr_faat01010000".to_string(),char: "Fang".to_string(),cost: 5, r#type:"ATB".to_string()},
        NodeFragment{name: "cr_faat02010000".to_string(),char: "Aang".to_string(),cost: 5, r#type:"ATB".to_string()},
        NodeFragment{name: "cr_faat02010000".to_string(),char: "Bang".to_string(),cost: 5, r#type:"ATB".to_string()},
        NodeFragment{name: "cr_faat02010000".to_string(),char: "Cang".to_string(),cost: 5, r#type:"ATB".to_string()},
        NodeFragment{name: "cr_faat03010000".to_string(),char: "Roku".to_string(),cost: 5, r#type:"ATB".to_string()},
    ];

    let mut node_fragments :Vec<NodeFragment> = node_fragments.to_vec();
    let crystal_page = CrystalPage::convert(&mut node_fragments);
    assert_eq!(crystal_page.len(), 3);
    assert_eq!(crystal_page[0].page, 1);
    assert_eq!(crystal_page[1].page, 2);
    assert_eq!(crystal_page[2].page, 3);
    assert_eq!(crystal_page[2].nodes[0].name, "cr_faat03010000");

}
// cr_faat01010000
// 01010000
// 01020000
// 01030000
// 01040000
// 01050000
// 01910000
// 02010000
// 02020000
// 02030000