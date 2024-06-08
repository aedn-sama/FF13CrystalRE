use byteorder::{BigEndian, ByteOrder};
use std::{ fmt, fs, io::{self, BufRead, BufReader, Cursor, Read, Seek}, vec };

// WDB
//     • int: CP cost
//     • int: String offset - Ability Id
//     • short: Node value
//     • byte: node type
//         ◦ 1 = HP
//         ◦ 2 = Str
//         ◦ 3 = Mag
//         ◦ 4 = Accessory
//         ◦ 5 = ATB
//         ◦ 6 = Ability
//         ◦ 7 = Role
//     • byte / 16: Stage
//     • byte % 16: Role
// 00 00 2E E0, 00 00 01 D9, 00 C3, 01, (91)->0101 1011
#[derive(Debug, Default)]
pub struct Entry {
    name: String,
    offset: i32,
    length: i32,
}

#[derive(Debug, Default)]
pub struct FileStructure {
    magic: String,
    count: i32,
    entries: Vec<Entry>,
    stringlist: Vec<String>,
    stringtypelist: Vec<u8>,
    typelist: Vec<u8>,
    version: i32,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum NodeType {
    #[default]
    HP,
    STR,
    MAG,
    ACCESSORY,
    ATB,
    ABILITY,
    ROLE,
    INVALID,
}

#[derive(Debug, Default, Clone)]
pub struct Node {
    pub char_name: String,
    pub node_name: String,
    pub cp_cost: i32,
    pub ability: String,
    pub node_value: i16,
    pub node_type: NodeType,
    pub stage: u8,
    pub role: u8,
}

#[derive(Default, Debug, Clone)]
pub struct Crystarium {
    pub nodes: Vec<Node>,
}

pub trait ReadUtilities {
    fn load_part<T: BufRead + Seek>(reader: &mut T, size: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size);

        // Get a reader for the next `size` amount of bytes
        let mut part_reader = reader.take(size as u64);

        // Read the part into the buffer
        part_reader.read_to_end(&mut buf).unwrap();

        // Return the buffer
        buf
    }

    fn load_string<T: BufRead + Seek>(reader: &mut T, size: usize) -> String {
        //Load a specific size of bytes into String::from_utf8 to extract the string from the bytes
        String::from_utf8(Self::load_part(reader, size))
            .unwrap()
            .to_string()
    }

    fn load_string_eof<T: BufRead + Seek>(reader: &mut T) -> String {
        //Read until null terminator comes.
        let mut buf_vec: Vec<u8> = Default::default();
        let _ = reader.read_until(b'\0', &mut buf_vec).unwrap();

        String::from_utf8(buf_vec).expect("Byte to string convert failed")
    }
}

impl ReadUtilities for Entry {}
impl Entry {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_entries<T: BufRead + Seek>(reader: &mut T, count: i32) -> Vec<Entry> {
        const ENTRYSIZE: usize = 32;
        const STRINGSIZE: usize = 16;

        let mut entries: Vec<Entry> = Vec::new();

        //Skip 8 bytes
        Self::load_part(reader, 8);

        for _ in 0..count {
            let name = Self::load_string_eof(reader);
            let mut padding_size = STRINGSIZE - name.len();
            let mut padding: Vec<u8>;
            if name.starts_with('!') {
                padding = vec![0; padding_size];
                reader.read(&mut padding).expect("Skip Padding failed");
            }

            let offset = BigEndian::read_i32(&Self::load_part(reader, 4));
            let length = BigEndian::read_i32(&Self::load_part(reader, 4));

            {
                // Jeder Entry ist 32 Stellen lang, also sicherstellen, das genug padding geskippt wird.
                padding_size = ENTRYSIZE - STRINGSIZE - 4 - 4;
                padding = vec![0; padding_size];
                reader.read(&mut padding).expect("Skip Padding failed");
            }

            entries.push(Entry {
                name: name,
                offset: offset,
                length: length,
            })
        }

        entries
    }
}

//Empty implementation for including functions.
impl ReadUtilities for FileStructure {}

impl FileStructure {
    pub fn load_version<T: BufRead + Seek>(reader: &mut T, entries: &Vec<Entry>) -> i32 {
        let _ = entries
            .iter()
            .find(|entry| entry.name == "!!version\0")
            .expect("version not found");

        let version = BigEndian::read_i32(&Self::load_part(reader, 4));
        version
    }

    pub fn load_typelist<T: BufRead + Seek>(reader: &mut T, entries: &Vec<Entry>) -> Vec<u8> {
        let entry = entries
            .iter()
            .find(|entry| entry.name == "!!typelist\0")
            .expect("typelist not found");
        let mut typelist = vec![0; entry.length as usize];

        reader.read_exact(&mut typelist).unwrap();

        typelist
    }

    pub fn load_stringtypelist<T: BufRead + Seek>(reader: &mut T, entries: &Vec<Entry>) -> Vec<u8> {
        let entry = entries
            .iter()
            .find(|entry| entry.name == "!!strtypelist\0")
            .expect("strtypelist not found");
        let mut stringtypelist = vec![0; entry.length as usize];

        reader.read_exact(&mut stringtypelist).unwrap();

        stringtypelist
    }

    pub fn load_stringlist<T: BufRead + Seek>(reader: &mut T, entries: &Vec<Entry>) -> Vec<String> {
        let entry = entries
            .iter()
            .find(|entry| entry.name == "!!string\0")
            .expect("string not found");
        let mut stringlist: Vec<String> = Default::default();
        let mut position = reader.stream_position().unwrap() as i32;

        if position == entry.offset {
            while position < entry.offset + entry.length {
                let name = Self::load_string_eof(reader);
                stringlist.push(name.clone());
                position += name.len() as i32;
            }
        } else {
            panic!("position/offset mismatch");
        }

        stringlist
    }

    pub fn load<T: BufRead + Seek>(reader: &mut T) -> FileStructure {
        let mut fstruct = FileStructure::default();
        fstruct.magic = Self::load_string(reader, 4);
        fstruct.count = BigEndian::read_i32(&Self::load_part(reader, 4));
        fstruct.entries = Entry::load_entries(reader, fstruct.count);
        fstruct.stringlist = Self::load_stringlist(reader, &mut fstruct.entries);
        fstruct.stringtypelist = Self::load_stringtypelist(reader, &mut fstruct.entries);
        fstruct.typelist = Self::load_typelist(reader, &mut fstruct.entries);
        fstruct.version = Self::load_version(reader, &mut fstruct.entries);

        fstruct
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            NodeType::HP => write!(f, "HP"),
            NodeType::STR => write!(f, "STR"),
            NodeType::MAG => write!(f, "MAG"),
            NodeType::ACCESSORY => write!(f, "ACCESSORY"),
            NodeType::ATB => write!(f, "ATB"),
            NodeType::ABILITY => write!(f, "ABILITY"),
            NodeType::ROLE => write!(f, "ROLE"),
            NodeType::INVALID => write!(f, "INVALID"),
        }
    }

}

impl NodeType{
    pub fn to_imagesrc(&self) -> &str{
        match self {
            NodeType::HP => "assets/Green Orb.png",
            NodeType::STR => "assets/Red Orb.png",
            NodeType::MAG => "assets/Purple Orb.png",
            NodeType::ACCESSORY => "assets/Orange Orb.png",
            NodeType::ATB => "assets/White Crystal.png",
            NodeType::ABILITY => "assets/Yellow Orb.png",
            NodeType::ROLE => "assets/White Crystal.png",
            NodeType::INVALID => "",
        }
    }
}

impl Node {
    pub fn new(
        char_name: String,
        node_name: String,
        cp_cost: i32,
        ability: String,
        node_value: i16,
        node_type: NodeType,
        stage: u8,
        role: u8,
    ) -> Self {
        return Node {
            char_name: char_name,
            node_name: node_name,
            cp_cost: cp_cost,
            ability: ability,
            node_value: node_value,
            node_type: node_type,
            stage: stage,
            role: role,
        };
    }
}

//Empty implementation for including functions.
impl ReadUtilities for Crystarium{}

impl Crystarium {
    pub fn create<T: BufRead + Seek + Sized>(reader: &mut T, fstruct: &FileStructure) -> Crystarium {
        let position = reader.stream_position().unwrap();
        let mut crystarium = Crystarium::default();
        
        //Going forward to first entry position.
        let entry = fstruct.entries.iter().filter(|e: &&Entry| !e.name.starts_with('!')).nth(0).unwrap();
        if position < entry.offset as u64{ 
            let advance_bytes = entry.offset as u64 - position;
            io::copy(&mut reader.by_ref().take(advance_bytes), &mut io::sink()).unwrap();
        }

        for entry in fstruct.entries.iter().filter(|e: &&Entry| !e.name.starts_with('!')){

            let cp_cost = BigEndian::read_i32(&Self::load_part(reader, 4));
            let _ = BigEndian::read_i32(&Self::load_part(reader, 4));
            let node_value: i16 = BigEndian::read_i16(&Self::load_part(reader, 2));
            let mut node_type_vec: Vec<u8> = Vec::new();
            io::copy(&mut reader.by_ref().take(1), &mut node_type_vec).unwrap();
            let node_type:NodeType = match node_type_vec[0]{
                1 => { NodeType::HP },
                2 => { NodeType::STR },
                3 => { NodeType::MAG },
                4 => { NodeType::ACCESSORY },
                5 => { NodeType::ATB },
                6 => { NodeType::ABILITY },
                7 => { NodeType::ROLE },
                _ => { NodeType::INVALID },   
            };

            let char_name = match &entry.name[..5] {
                "cr_fa" => "Fang",
                "cr_hp" => "Hope",
                "cr_lt" => "Lightning",
                "cr_sz" => "Sazh",
                "cr_sn" => "Snow",
                "cr_va" => "Vanille",
                _ => "None",
            };
            
            let mut buf_stagerole: Vec<u8> = Vec::new();
            io::copy(&mut reader.by_ref().take(1), &mut buf_stagerole).unwrap();
            let stage = buf_stagerole[0]/16;
            let role = buf_stagerole[0]%16;
 
            crystarium.nodes.push(Node::new(
                char_name.to_string(),
                entry.name.clone(),
                cp_cost,
                "ability".to_string(),
                node_value,
                //ability,
                node_type,
                stage,
                role
            ));
        }    

        crystarium
    }
}

pub fn read_crystal_wdb_with_file(path: &str) -> Result<Crystarium, &'static str> {
    let file_h = fs::File::open(path).unwrap();

    //Buffered Reader for file
    let mut b_reader = BufReader::new(file_h);

    //File Structure Mapping
    let fstruct = FileStructure::load(&mut b_reader);

    //Using the file structure to get the data for crystal infos.
    let crystarium = Crystarium::create(&mut b_reader, &fstruct);

    Ok(crystarium)
}

pub fn read_crystal_wdb(data: Vec<u8>) -> Result<Crystarium, &'static str> {
    //Buffered Reader for file
    let mut b_cursor = Cursor::new(data);

    //File Structure Mapping
    let fstruct = FileStructure::load(&mut b_cursor);

    //Using the file structure to get the data for crystal infos.
    let crystarium = Crystarium::create(&mut b_cursor, &fstruct);

    Ok(crystarium)
}
