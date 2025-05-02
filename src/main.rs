use rext4::{defs::BlockContents, group::Ext4Fs};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let file_path = env::args().nth(1).expect("file path required");
    let mut ext4_file = File::open(file_path).expect("file path does not exists");
    let mut contents = Vec::new();
    ext4_file
        .read_to_end(&mut contents)
        .expect("failed to read to end");

    let ext4_fs = Ext4Fs::from_file(&contents).unwrap();
    traverse_file(&ext4_fs)
}

fn traverse_file(ext4_fs: &Ext4Fs) {
    let root_inode_num = 2;
    let mut queue: VecDeque<(u64, u64, String)> = VecDeque::new();
    queue.push_back((root_inode_num, root_inode_num, "".into()));
    while let Some((parent_inode_num, current_inode_num, parent_path)) = queue.pop_front() {
        println!("{}", parent_path);
        if let Some(inode) = ext4_fs.get_inode(current_inode_num) {
            if let Some(i_block_contents) = ext4_fs.get_inode_block_contents(&inode) {
                match i_block_contents {
                    BlockContents::Data(chain) => {},
                    BlockContents::Dentries(entries) => {
                        let mut num = 0;
                        for d_entry in entries {
                            println!("    d_entry num {}", d_entry.inode);
                            num += 1;
                            if d_entry.inode < 2 {
                                println!("    parent inode is {current_inode_num} {:?}", inode);
                                println!("    ignore this {:?}", d_entry);
                                continue;
                            }
                            if (d_entry.inode as u64 != current_inode_num) && (parent_inode_num != d_entry.inode as u64) {
                                if d_entry.inode == 11 {println!("number eleven")}
                                let file_path = if let Ok(file_name) = d_entry.get_name() {
                                    format!("{}/{}", parent_path, file_name)
                                } else {
                                    println!("cannot get file name {}", d_entry.inode);
                                    format!("{}/<cannot parse>", parent_path)
                                };
                                queue.push_back((
                                        current_inode_num, d_entry.inode as u64, file_path
                                        ))
                            } else {
                                println!("    not ignore d_entry {:?}", d_entry)
                            }
                        }
                        println!("    entry num is {} {}", num, current_inode_num)
                    },
                    BlockContents::InliedData(data) => {
                        if inode.i_mode.ty.is_symlink() {
                            let name = String::from_utf8(data.into()).unwrap_or("failed to get data!".into());
                            println!("symlink to {name}")
                        }
                    }
                }
            } else {
                println!("cannot parse i_block")
            }
        } else {
            println!("cannot find inode {}", current_inode_num);
        }
    }
}
