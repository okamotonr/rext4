use crate::defs::{Ext4GroupDesc, Ext4Inode, Ext4SuperBlock};
use nom::bytes::streaming::take;
use nom::multi::count;
use nom::{IResult, Parser};
use nom_derive::Parse;

pub fn consume_block(input: &[u8], block_size: u64, current_bytes: u64) -> IResult<&[u8], &[u8]> {
    let rem = (block_size - (current_bytes % block_size)) % block_size;
    take(rem)(input)
}

pub fn parse_block0(input: &[u8]) -> IResult<&[u8], BlockGroupWithSuperBlock> {
    let (input, _) = take(1024_u32)(input)?;
    let (input, super_block) = Ext4SuperBlock::parse(input)?;
    let (input, _) = consume_block(input, super_block.s_log_block_size, 2048)?;

    let group_num = super_block.s_blocks_count_lo / super_block.s_blocks_per_group;
    let (input, group_descs) = count(Ext4GroupDesc::parse, group_num as usize).parse(input)?;
    let descs_bytes = (group_num * super_block.s_desc_size as u32) as u64;
    let (input, _) = consume_block(input, super_block.s_log_block_size, descs_bytes)?;
    let (input, r_gdt_blocks) =
        take(super_block.s_log_block_size * super_block.s_reserved_gdt_blocks as u64)(input)?;

    let ret = BlockGroupWithSuperBlock {
        super_block,
        group_descs,
        reserved_gdt_blocks: r_gdt_blocks,
    };
    Ok((input, ret))
}
//
// pub fn parse_block_group(input: &[u8], block_size: u64, ) -> IResult<&[u8], BlockGroup> {
//
// }


#[derive(Debug)]
pub struct BlockGroupWithSuperBlock<'a> {
    pub super_block: Ext4SuperBlock,
    pub group_descs: Vec<Ext4GroupDesc>,
    pub reserved_gdt_blocks: &'a [u8],
}

#[derive(Debug)]
pub struct BlockGroup<'a> {
    pub data_block_bitmap: &'a [u8],
    pub inode_bitmap: &'a [u8],
    pub inode_table: &'a [u8],
}

pub struct BlockGroupParser {
    block_size: u64,
    inode_bitmap_size: u64,
    inode_table_size: u64,
}

impl BlockGroupParser {
    pub fn new(super_block: &Ext4SuperBlock) -> Self {
        let block_size = super_block.s_log_block_size;
        let inode_table_size =
            super_block.s_inode_size as u64 * super_block.s_inodes_per_group as u64;
        let inode_bitmap_size = super_block.s_inodes_per_group as u64 / 8;
        Self {
            block_size,
            inode_table_size,
            inode_bitmap_size
        }
    }

    pub fn parse<'a, 'b>(
        &self,
        input: &'a [u8],
        group_desc: &'b Ext4GroupDesc,
    ) -> IResult<&'a [u8], BlockGroup<'a>> {
        let (input2, _) = take(group_desc.bg_block_bitmap_lo as u64 * self.block_size)(input)?;
        let (_, data_block_bitmap) = take(self.block_size)(input2)?;

        let (input2, _) = take(group_desc.bg_inode_bitmap_lo as u64 * self.block_size)(input)?;
        let (_, inode_bitmap) = take(self.inode_bitmap_size)(input2)?;

        let (input2, _) = take(group_desc.bg_inode_table_lo as u64 * self.block_size)(input)?;
        let (ret, inode_table) = take(self.inode_table_size)(input2)?;

        Ok((
            ret,
            BlockGroup {
                data_block_bitmap,
                inode_bitmap,
                inode_table,
            },
        ))
    }
}

pub fn parse_bitmap(bitmap: &[u8]) -> impl Iterator<Item = bool> {
    bitmap.iter().map(|&x| {x_to_bools(x)}).flatten()
}

fn x_to_bools(x: u8) -> [bool; 8] {
    let ret = [
        0x1 & x == 0x1,
        0x2 & x == 0x2,
        0x4 & x == 0x4,
        0x8 & x == 0x8,
        0x10 & x == 0x10,
        0x20 & x == 0x20,
        0x40 & x == 0x40,
        0x80 & x == 0x80,
    ];
    ret
}

pub struct InodeParser {
    inode_size: u64,
    inode_struct_size: u64,
}

impl InodeParser {
    pub fn new(inode_size: u64) -> Self {
        let inode_struct_size = size_of::<Ext4Inode>() as u64;
        assert!(inode_struct_size <= inode_size);
        Self { inode_size, inode_struct_size }
    }

    pub fn parse<'a, 'b>(&'a self, input: &'b [u8], start_at: usize) -> IResult<&'b [u8], Ext4Inode> {
        let input = &input[(start_at * self.inode_size as usize)..];
        let (input, inode) = Ext4Inode::parse(input)?;
        let (input, _) = take(self.inode_size - self.inode_struct_size)(input)?;
        Ok((input, inode))
    }
}
