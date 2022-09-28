use bitbit::*;
use std::env;
use std::{collections::HashMap, hash::Hash};

fn main() -> () {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("you need to specify 3 arguments: encode/decode, readpath, writepath")
    }

    match args[1].as_str() {
        "encode" => {
            let text: String = std::fs::read_to_string(&args[2]).unwrap();
            let char_count = count_characters_in_string(&text);
            let tree: Tree = Tree::new_from_char_count(&char_count);
            let mut list: Vec<(char, String)> = Vec::new();
            generate_list_from_tree(&tree, &mut list, "".to_string());
            encode(&text, list.clone(), &args[3]);
        }
        "decode" => {
            let decoded = decode(&args[2]);
            _ = std::fs::write(&args[3], decoded);
        }
        _ => panic!("you need to specify if you want to encode or decode a file (arg1)")
    }
}

fn encode(s: &String, list: Vec<(char, String)>, path: &String) {
    let w = std::fs::File::create(path).unwrap();
    let mut bw = BitWriter::new(w);

    let hm: HashMap<char, String> = list.into_iter().collect();

    // write character-code pairs
    for (c, code) in &hm {
        _ = bw.write_bit(true); // there is another character coming

        let mut c_16: [u16; 2] = [0; 2];
        let c_16_slice = c.encode_utf16(&mut c_16);

        // convert character to utf-16 binary
        let mut c_16_bin: String = "".to_string();
        match c_16_slice.len() {
            1 => {
                _ = bw.write_bit(false); // length of character is 1 x 16
                c_16_bin = format!("{:0>16b}", c_16_slice[0]);
            }
            2 => {
                _ = bw.write_bit(true); // length of character is 2 x 16
                c_16_bin = format!("{:0>16b}{:0>16b}", c_16_slice[0], c_16_slice[1]);
            }
            _ => {}
        }

        // write character
        for b in c_16_bin.chars() {
            match b {
                '0' => _ = bw.write_bit(false),
                '1' => _ = bw.write_bit(true),
                _ => {}
            }
        }

        // write code length (8bit)
        if code.len() > 255 {
            panic!("character code is longer than 255 bits");
        }
        for b in format!("{:0>8b}", code.len()).chars() {
            match b {
                '0' => _ = bw.write_bit(false),
                '1' => _ = bw.write_bit(true),
                _ => {}
            }
        }

        // write code
        for b in code.chars() {
            match b {
                '0' => _ = bw.write_bit(false),
                '1' => _ = bw.write_bit(true),
                _ => {}
            }
        }
    }
    _ = bw.write_bit(false); // end of character-code pairs

    // encode and write text
    for c in s.chars() {
        let code = hm.get(&c).unwrap();
        for b in code.chars() {
            match b {
                '0' => _ = bw.write_bit(false),
                '1' => _ = bw.write_bit(true),
                _ => {}
            }
        }
    }

    // add \0 code at the end
    for b in hm.get(&'\0').unwrap().chars() {
        match b {
            '0' => _ = bw.write_bit(false),
            '1' => _ = bw.write_bit(true),
            _ => {}
        }
    }

    bw.pad_to_byte().unwrap();
}

fn decode(filedir: &str) -> String {
    let r = std::fs::File::open(filedir).unwrap();
    let mut br: BitReader<_, MSB> = BitReader::new(r);

    // construct code-character hashmap from file header
    let mut hm: HashMap<String, char> = HashMap::new();
    loop {
        // zero bit is end of header
        if !br.read_bit().unwrap() {
            break;
        }

        // read character
        let mut c_16_bin: String = "".to_string();
        let c_len: u8 = if br.read_bit().unwrap() { 32 } else { 16 };
        for _ in 0..c_len {
            if br.read_bit().unwrap() {
                c_16_bin.push('1')
            } else {
                c_16_bin.push('0')
            }
        }
        if c_len == 16 {
            c_16_bin.push_str("0000000000000000")
        }

        // convert character from utf-16 binary to char
        let c_16_buff: [u16; 2] = [
            u16::from_str_radix(&c_16_bin[0..16], 2).unwrap(),
            u16::from_str_radix(&c_16_bin[16..32], 2).unwrap(),
        ];
        let c_16_dec = char::decode_utf16(c_16_buff);
        let c: char = c_16_dec.map(|r| r.unwrap()).collect::<Vec<_>>()[0];

        // read code length
        let mut code_len_bin: String = "".to_string();
        for _ in 0..8 {
            if br.read_bit().unwrap() {
                code_len_bin.push('1')
            } else {
                code_len_bin.push('0')
            }
        }
        let code_len = u8::from_str_radix(&code_len_bin, 2).unwrap();

        // read code
        let mut code: String = "".to_string();
        for _ in 0..code_len {
            if br.read_bit().unwrap() {
                code.push('1')
            } else {
                code.push('0')
            }
        }

        // add code-character pair to hashmap
        hm.insert(code, c);
    }

    // decode file
    let mut decoded: String = "".to_string();
    let mut curr: String = "".to_string();
    loop {
        let b = match br.read_bit() {
            Ok(it) => it,
            Err(_err) => break,
        };

        if b {
            curr.push('1')
        } else {
            curr.push('0')
        }

        if hm.get(&curr).is_some() {
            // if character code is correspondent with \0 end decoding
            if *hm.get(&curr).unwrap() == '\0' {
                break;
            }

            decoded.push(*hm.get(&curr).unwrap());
            curr = "".to_string();
        }
    }

    decoded
}

fn count_characters_in_string(s: &String) -> HashMap<char, u32> {
    let mut char_count: HashMap<char, u32> = HashMap::new();

    for c in s.chars() {
        char_count
            .entry(c)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    char_count
}

fn generate_list_from_tree(tree: &Tree, list: &mut Vec<(char, String)>, code: String) {
    let mut current_node: usize = tree.nodes.len() - 1;
    for dir in code.chars() {
        match dir {
            '0' => current_node = tree.nodes[current_node].child1.unwrap(),
            '1' => current_node = tree.nodes[current_node].child2.unwrap(),
            _ => {}
        }
    }

    if tree.nodes[current_node].character.is_some() {
        list.push((tree.nodes[current_node].character.unwrap(), code));
        return;
    }

    generate_list_from_tree(tree, list, code.clone() + "0");
    generate_list_from_tree(tree, list, code.clone() + "1");
}

pub struct Node {
    parent: Option<usize>,
    sibling: Option<usize>,
    child1: Option<usize>,
    child2: Option<usize>,

    weight: u32,
    character: Option<char>,
}

pub struct Tree {
    nodes: Vec<Node>,
}
impl Tree {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn new_from_char_count(char_count: &HashMap<char, u32>) -> Self {
        let mut tree: Tree = Tree::new();
        for (c, count) in char_count {
            tree.add_node(*count, *c);
        }

        tree.add_node(1, '\0');
        tree.connect_all();

        tree
    }

    fn add_node(&mut self, weight: u32, character: char) {
        self.nodes.push(Node {
            parent: None,
            sibling: None,
            child1: None,
            child2: None,
            weight,
            character: Some(character),
        });
    }

    fn connect_nodes(&mut self, node1: usize, node2: usize) {
        let parent_index = self.nodes.len();

        self.nodes.push(Node {
            parent: None,
            sibling: None,
            child1: Some(node1),
            child2: Some(node2),
            weight: self.nodes[node1].weight + self.nodes[node2].weight,
            character: None,
        });

        self.nodes[node1].parent = Some(parent_index);
        self.nodes[node2].parent = Some(parent_index);
    }

    fn find_two_smallest(&self) -> (usize, usize) {
        let mut smallest: u32 = u32::max_value();
        let mut smallest2: u32 = u32::max_value();
        let mut smallest_index: usize = 0;
        let mut smallest2_index: usize = 0;

        for i in 0..self.nodes.len() {
            if self.nodes[i].parent.is_some() {
                continue;
            }

            if self.nodes[i].weight < smallest {
                smallest2 = smallest;
                smallest2_index = smallest_index;
                smallest = self.nodes[i].weight;
                smallest_index = i;
                continue;
            }
            if self.nodes[i].weight < smallest2 {
                smallest2 = self.nodes[i].weight;
                smallest2_index = i;
            }
        }

        (smallest_index, smallest2_index)
    }

    fn connect_all(&mut self) {
        let mut whole_weight: u32 = 0;
        for node in &self.nodes {
            whole_weight += node.weight;
        }

        loop {
            let (smallest1, smallest2) = self.find_two_smallest();
            self.connect_nodes(smallest1, smallest2);

            let new_node_weight = self.nodes[self.nodes.len() - 1].weight;
            if new_node_weight >= whole_weight {
                break;
            }
        }
    }
}
