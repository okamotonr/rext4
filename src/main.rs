use std::env;
use std::fs::File;
use std::io::Read;
use rext4::{defs::BlockContents, group::Ext4Fs, parser::{parse_bitmap, parse_block0, BlockGroupParser, InodeParser}};

fn files_to_blocks(file: &[u8], block_size: u64, n_block: u64, block_index: u64) -> &[u8] {
    let start = (block_size * block_index) as usize;
    let end = start + ((n_block * block_size) as usize);
    &file[start..end]
}

fn main() {
    let file_path = env::args().nth(1).expect("file path required");
    let mut ext4_file = File::open(file_path).expect("file path does not exists");
    let mut contents = Vec::new();
    ext4_file.read_to_end(&mut contents).expect("failed to read to end");
    let (_, ret) = parse_block0(&contents).expect("failed to parse");
    println!("{:?}", ret.super_block);

    let block_size = ret.super_block.s_log_block_size;

    let ext4fs = Ext4Fs::from_file(&contents).unwrap();

    let inode_parser = InodeParser::new(ret.super_block.s_inode_size as u64);
    let bg_parser = BlockGroupParser::new(&ret.super_block);
    for desc in ret.group_descs {
        let (_, bg) = bg_parser.parse(&contents, &desc).expect("failed");
        for (index, is_set) in parse_bitmap(bg.inode_bitmap).enumerate() {
            if is_set {
                let _inode = ext4fs.get_inode((index + 1) as u64).unwrap();
                let _b_contents = ext4fs.get_inode_block_contents(&_inode);
                let (_, inode) = inode_parser.parse(&bg.inode_table, index).unwrap();
                if let Some(block) = _b_contents {
                    match block {
                        BlockContents::Dentries(chain) => {
                            for d in chain.into_iter() {
                                println!("{:?}", d);
                                println!("{:?}", d.get_name());
                            }
                        }
                        _ => {}
                    }

                }
                assert!(_inode == inode);
                println!("file_no {} is {:x?}", index+1, inode);
                if let Some(extents) = inode.get_extents() {
                    for e in extents {
                        println!("{:x?}", e);
                        println!("{:?}", e.ee_start());
                        let ext = files_to_blocks(&contents, block_size, e.ee_len as u64, e.ee_start());
                        if inode.i_mode.ty.is_regular() {
                            if let Ok(string) = String::from_utf8(ext.to_vec()) {
                                print!("{string}")
                            } else {
                                eprintln!("Cannot convert")
                            }
                        }
                        println!("{}", ext.len());
                    }
                }
                if let Some(extent_indecies) = inode.get_extent_indicies() {
                    for e in extent_indecies {
                        println!("{:x?}", e);
                    }
                }
                println!()
            }
        }
    }
}

