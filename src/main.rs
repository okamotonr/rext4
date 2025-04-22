use std::env;
use std::fs::File;
use std::io::Read;
use rext4::parser::{parse_bitmap, parse_block0, BlockGroupParser, InodeParser};

fn main() {
    let file_path = env::args().nth(1).expect("file path required");
    let mut ext4_file = File::open(file_path).expect("file path does not exists");
    let mut contents = Vec::new();
    ext4_file.read_to_end(&mut contents).expect("failed to read to end");
    let (_, ret) = parse_block0(&contents).expect("failed to parse");
    println!("{:?}", ret.super_block);
    let inode_parser = InodeParser::new(ret.super_block.s_inode_size as u64);
    let bg_parser = BlockGroupParser::new(&ret.super_block);
    for desc in ret.group_descs {
        let (_, bg) = bg_parser.parse(&contents, &desc).expect("failed");
        for (index, is_set) in parse_bitmap(bg.inode_bitmap).enumerate() {
            if is_set {
                let (_, inode) = inode_parser.parse(&bg.inode_table, index).unwrap();
                println!();
                println!("index {} is {:x?}", index, inode);
            }
        }
    }
}

