use crate::defs::{BlockContents, Ext4GroupDesc, Ext4Inode, Ext4SuperBlock, RoCompatFeatures};
use nom::multi::count;
use nom::Parser;
use nom_derive::Parse;

pub struct Ext4Fs<'a> {
    super_block: Ext4SuperBlock,
    pub group_descs: Vec<Ext4GroupDesc>,
    pub file: &'a [u8]
}

pub type Err = String;

fn is_power_of(mut n: u64, b: u64) -> bool {
    match b {
        0 => return n == 0,        
        1 => return n == 1,       
        _ => {}
    }

    if n == 0 {
        return false;            
    }

    while n % b == 0 {
        n /= b;
    }
    n == 1
}

fn has_super_block(is_sparse: bool, index: usize) -> bool {
    is_sparse || index == 0 || [3, 5, 7].iter().any(|&x| is_power_of(index as u64, x))
} 

impl<'a> Ext4Fs<'a> {
    pub fn from_file(input: &'a [u8]) -> Result<Self, Err> {
        let (_, super_block) = Ext4SuperBlock::parse(&input[1024..]).unwrap();
        let group_descs = Self::parse_group_descs(&super_block, input);
        Ok(Self { super_block, group_descs, file: input })
    }

    pub fn is_sparse(&self) -> bool {
        self.super_block.s_feature_ro_compat.contains(RoCompatFeatures::SPARSE_SUPER)
    }

    fn parse_group_descs(super_block: &Ext4SuperBlock, input: &[u8]) -> Vec<Ext4GroupDesc> {
        let block_size = super_block.s_log_block_size;
        let block_num = super_block.s_blocks_count_lo;
        let group_num = super_block.s_blocks_count_lo / super_block.s_blocks_per_group;
        assert!((block_size * block_num as u64) == input.len() as u64);

        let (_, group_descs) = count(Ext4GroupDesc::parse, group_num as usize).parse(&input[block_size as usize ..]).unwrap();
        group_descs
    }

    pub fn get_inode(&self, i_no: u64) -> Option<Ext4Inode> {
        let offset_in_block = (i_no - 1) % self.super_block.s_inodes_per_group as u64;
        let block_index = (i_no - 1) / self.super_block.s_inodes_per_group as u64;
        let group_desc = self.group_descs.get(block_index as usize)?;

        (self.get_inode_bit(offset_in_block, group_desc)?).then_some(())?;

        let inodetable_start = (group_desc.bg_inode_table_lo as u64) * self.super_block.s_log_block_size;
        let offset = offset_in_block * self.super_block.s_inode_size as u64;
        Ext4Inode::parse(&self.file[(inodetable_start + offset) as usize..]).map(
            |(_, inode)| inode).ok()
    }

    pub fn get_inode_block_contents(&self, inode: &Ext4Inode) -> Option<BlockContents> {
        inode.get_i_block_contents(&self.file, self.super_block.s_log_block_size as usize)
    }

    fn get_inode_bit(&self, offset_in_block: u64, group_desc: &Ext4GroupDesc) -> Option<bool> {
        let bitmap_start = (group_desc.bg_inode_bitmap_lo as usize) * self.super_block.s_log_block_size as usize;

        let inode_bitgroup_index = offset_in_block / 8;
        let inode_bit_index = offset_in_block % 8;
        let inode_bit_group = &self.file.get(bitmap_start + inode_bitgroup_index as usize)?;
        Some(((*inode_bit_group) >> inode_bit_index) & 0x1 == 0x1)
    }
}

