use bitbit::*;
use std::env;
use std::collections::HashMap;

mod huffman_tree;
use crate::huffman_tree::Tree;


fn main() -> () {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("you need to specify 3 arguments: encode/decode, readpath, writepath")
    }

    match args[1].as_str() {
        "encode" => {
            let text: String = std::fs::read_to_string(&args[2]).unwrap();
            let tree: Tree = Tree::new_from_string(&text);
            let mut list: Vec<(char, String)> = Vec::new();
            tree.generate_list(&mut list, "".to_string());
            encode(&text, list.clone(), &args[3]);
        }
        "decode" => {
            let decoded = decode(&args[2]);
            _ = std::fs::write(&args[3], decoded);
        }
        _ => panic!("you need to specify if you want to encode or decode a file (arg1)")
    }
}


//-- encode ---
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
            write_charbit(b, &mut bw);
        }

        // write code length (8bit)
        if code.len() > 255 {
            panic!("character code is longer than 255 bits");
        }
        for b in format!("{:0>8b}", code.len()).chars() {
            write_charbit(b, &mut bw);
        }

        // write code
        for b in code.chars() {
            write_charbit(b, &mut bw);
        }
    }
    _ = bw.write_bit(false); // end of character-code pairs

    // encode and write text
    for c in s.chars() {
        let code = hm.get(&c).unwrap();
        for b in code.chars() {
            write_charbit(b, &mut bw);
        }
    }

    // add \0 code at the end
    for b in hm.get(&'\0').unwrap().chars() {
        write_charbit(b, &mut bw);
    }

    bw.pad_to_byte().unwrap();
}

fn write_charbit(b: char, bw: &mut BitWriter<std::fs::File>) {
    match b {
        '0' => _ = bw.write_bit(false),
        '1' => _ = bw.write_bit(true),
        _ => {}
    }
}

//-- decode ---
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
        read_bits_into_string(&mut c_16_bin, c_len, &mut br);
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
        read_bits_into_string(&mut code_len_bin, 8, &mut br);
        let code_len = u8::from_str_radix(&code_len_bin, 2).unwrap();

        // read code
        let mut code: String = "".to_string();
        read_bits_into_string(&mut code, code_len, &mut br);

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

fn read_bits_into_string(s: &mut String, len: u8, br: &mut BitReader<std::fs::File, MSB>) {
    for _ in 0..len {
        if br.read_bit().unwrap() {
            s.push('1')
        } else {
            s.push('0')
        }
    }
}