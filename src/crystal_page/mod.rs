use crate::view::{ListRoleFragment, NodeFragment, RoleFragment};
use std::collections::BTreeMap;

#[derive(Default, Debug, Clone)]
pub struct CrystalPage {
    pub character: String,
    pub stage: i16,
    pub node_count: usize,
    pub roles: Vec<RoleFragment>,
}



impl CrystalPage {
    pub fn convert(character: &str, node_fragments: &mut Vec<NodeFragment>) -> Vec<Self> {
        //Test with BtreeMap
        let mut btree_entries: BTreeMap<i16, BTreeMap<String, Vec<NodeFragment>>> = BTreeMap::new();

        for node in node_fragments.clone() {
            btree_entries
                .entry(node.stage)
                .or_default()
                .entry(node.role.clone())
                .or_default()
                .push(node.clone());
        }

        //create crystal pages from btree map
        let mut crystal_pages = Vec::<CrystalPage>::new();

        for entry in btree_entries {
            let mut role_fragments: ListRoleFragment = ListRoleFragment(Vec::<RoleFragment>::new());
            let mut crystal_page = CrystalPage::default();
            crystal_page.character = character.to_string();
            crystal_page.stage = entry.0;
            
            for role in entry.1{
                let mut role_fragment: RoleFragment = RoleFragment::default();
                role_fragment.name = role.0;
                role_fragment.nodes = role.1;
                role_fragments.0.push(role_fragment.clone());
            }

            crystal_page.roles = role_fragments.0.clone();
            crystal_pages.push(crystal_page);
        }

        crystal_pages
    }
}


#[test]
fn test_convert() {
    // let node_fragments = [
    //     NodeFragment {
    //         name: "cr_faat01010000".to_string(),
    //         cost: 5,
    //         r#type: "STR".to_string(),
    //         value: 10,
    //         stage: 1,
    //         role: "Commander".to_string(),
    //         image: "templates/assets/Blue Orb.png".to_string(),
    //     },
    //     NodeFragment {
    //         name: "cr_faat02010000".to_string(),
    //         cost: 5,
    //         r#type: "MAG".to_string(),
    //         value: 10,
    //         stage: 2,
    //         role: "Commander".to_string(),
    //         image: "templates/assets/Blue Orb.png".to_string(),
    //     },
    //     NodeFragment {
    //         name: "cr_faat02010000".to_string(),
    //         cost: 5,
    //         r#type: "ATB".to_string(),
    //         value: 10,
    //         stage: 3,
    //         role: "Commander".to_string(),
    //         image: "templates/assets/Blue Orb.png".to_string(),
    //     },
    //     NodeFragment {
    //         name: "cr_faat02010000".to_string(),
    //         cost: 5,
    //         r#type: "ACCESSORY".to_string(),
    //         value: 10,
    //         stage: 1,
    //         role: "Commander".to_string(),
    //         image: "templates/assets/Blue Orb.png".to_string(),
    //     },
    //     NodeFragment {
    //         name: "cr_faat03010000".to_string(),
    //         cost: 5,
    //         r#type: "ATB".to_string(),
    //         value: 10,
    //         stage: 1,
    //         role: "Commander".to_string(),
    //         image: "templates/assets/Blue Orb.png".to_string(),
    //     },
    // ];

    // let mut node_fragments: Vec<NodeFragment> = node_fragments.to_vec();
    // let crystal_page = CrystalPage::convert("Lightning",&mut node_fragments);
    // assert_eq!(crystal_page.len(), 3);
    // assert_eq!(crystal_page[0].stage, 1);
    // assert_eq!(crystal_page[1].stage, 2);
    // assert_eq!(crystal_page[2].stage, 3);
    // assert_eq!(crystal_page[2].roles[0].name, "cr_faat03010000");
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
