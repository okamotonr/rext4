use bitflags::bitflags;
use nom::combinator::map;
use nom::number::complete::{le_u8, le_u16, le_u32};
use nom_derive::{Nom, nom};
use std::{mem::offset_of, ptr::slice_from_raw_parts};

use crate::chain::BufferChainer;

pub const EXT4_LABEL_MAX: usize = 16;
pub const EXT4_S_ERR_END: usize = offset_of!(Ext4SuperBlock, s_mount_opts);
pub const EXT4_S_ERR_START: usize = offset_of!(Ext4SuperBlock, s_error_count);

#[repr(C)]
#[derive(Debug, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct Ext4SuperBlock {
    // 0x00
    pub s_inodes_count: u32,
    pub s_blocks_count_lo: u32,
    pub s_r_blocks_count_lo: u32,
    pub s_free_blocks_count_lo: u32,
    // 0x10
    pub s_free_inodes_count: u32,
    pub s_first_data_block: u32,
    #[nom(Map = "|x: u32| (2 << (x + 9)) as u64", Parse = "le_u32")]
    pub s_log_block_size: u64,
    #[nom(Map = "|x: u32| (2 << (x + 9)) as u64", Parse = "le_u32")]
    pub s_log_cluster_size: u64,
    // 0x20
    pub s_blocks_per_group: u32,
    pub s_clusters_per_group: u32,
    pub s_inodes_per_group: u32,
    pub s_mtime: u32,
    // 0x30
    pub s_wtime: u32,
    pub s_mnt_count: u16,
    pub s_max_mnt_count: u16,
    pub s_magic: u16,
    pub s_state: u16,
    pub s_errors: u16,
    pub s_minor_rev_level: u16,
    // 0x40
    pub s_lastcheck: u32,
    pub s_checkinterval: u32,
    pub s_creator_os: u32,
    pub s_rev_level: u32,
    // 0x50
    pub s_def_resuid: u16,
    pub s_def_resgid: u16,
    // dynamic‑rev only
    pub s_first_ino: u32,
    pub s_inode_size: u16,
    pub s_block_group_nr: u16,
    #[nom(Parse = "map(le_u32, CompatFeatures::from_bits_truncate)")]
    pub s_feature_compat: CompatFeatures,
    // 0x60
    #[nom(Parse = "map(le_u32, IncompatFeatures::from_bits_truncate)")]
    pub s_feature_incompat: IncompatFeatures,
    #[nom(Parse = "map(le_u32, RoCompatFeatures::from_bits_truncate)")]
    pub s_feature_ro_compat: RoCompatFeatures,
    // 0x68
    pub s_uuid: [u8; 16],
    // 0x78
    pub s_volume_name: [u8; EXT4_LABEL_MAX],
    // 0x88
    pub s_last_mounted: [u8; 64],
    // 0xC8
    pub s_algorithm_usage_bitmap: u32,
    pub s_prealloc_blocks: u8,
    pub s_prealloc_dir_blocks: u8,
    pub s_reserved_gdt_blocks: u16,
    // 0xD0
    pub s_journal_uuid: [u8; 16],
    // 0xE0
    pub s_journal_inum: u32,
    pub s_journal_dev: u32,
    pub s_last_orphan: u32,
    pub s_hash_seed: [u32; 4],
    pub s_def_hash_version: u8,
    pub s_jnl_backup_type: u8,
    pub s_desc_size: u16,
    // 0x100
    pub s_default_mount_opts: u32,
    pub s_first_meta_bg: u32,
    pub s_mkfs_time: u32,
    pub s_jnl_blocks: [u32; 17],
    // 0x150
    pub s_blocks_count_hi: u32,
    pub s_r_blocks_count_hi: u32,
    pub s_free_blocks_count_hi: u32,
    pub s_min_extra_isize: u16,
    pub s_want_extra_isize: u16,
    pub s_flags: u32,
    pub s_raid_stride: u16,
    pub s_mmp_update_interval: u16,
    pub s_mmp_block: u64,
    pub s_raid_stripe_width: u32,
    #[nom(Map = "|x: u8| (2 << (x + 9)) as u64", Parse = "le_u8")]
    pub s_log_groups_per_flex: u64,
    pub s_checksum_type: u8,
    pub s_encryption_level: u8,
    pub s_reserved_pad: u8,
    pub s_kbytes_written: u64,
    pub s_snapshot_inum: u32,
    pub s_snapshot_id: u32,
    pub s_snapshot_r_blocks_count: u64,
    pub s_snapshot_list: u32,

    // --- error region starts here ---
    pub s_error_count: u32,
    pub s_first_error_time: u32,
    pub s_first_error_ino: u32,
    pub s_first_error_block: u64,
    pub s_first_error_func: [u8; 32],
    pub s_first_error_line: u32,
    pub s_last_error_time: u32,
    pub s_last_error_ino: u32,
    pub s_last_error_line: u32,
    pub s_last_error_block: u64,
    pub s_last_error_func: [u8; 32],
    // --- error region ends at s_mount_opts ---
    pub s_mount_opts: [u8; 64],

    pub s_usr_quota_inum: u32,
    pub s_grp_quota_inum: u32,
    pub s_overhead_clusters: u32,
    pub s_backup_bgs: [u32; 2],
    pub s_encrypt_algos: [u8; 4],
    pub s_encrypt_pw_salt: [u8; 16],
    pub s_lpf_ino: u32,
    pub s_prj_quota_inum: u32,
    pub s_checksum_seed: u32,
    pub s_wtime_hi: u8,
    pub s_mtime_hi: u8,
    pub s_mkfs_time_hi: u8,
    pub s_lastcheck_hi: u8,
    pub s_first_error_time_hi: u8,
    pub s_last_error_time_hi: u8,
    pub s_first_error_errcode: u8,
    pub s_last_error_errcode: u8,
    pub s_encoding: u16,
    pub s_encoding_flags: u16,
    pub s_orphan_file_inum: u32,
    pub s_reserved: [u32; 94],
    pub s_checksum: u32,
}

const EXT4_NDIR_BLOCKS: usize = 12;
const EXT4_IND_BLOCK: usize = EXT4_NDIR_BLOCKS;
const EXT4_DIND_BLOCK: usize = EXT4_IND_BLOCK + 1;
const EXT4_TIND_BLOCK: usize = EXT4_DIND_BLOCK + 1;
/// Number of block pointers in the inode
pub const EXT4_N_BLOCKS: usize = EXT4_TIND_BLOCK + 1; // adjust if different

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Nom)]
#[nom(LittleEndian)]
pub struct Ext4Inode {
    #[nom(Parse = "map(le_u16, FileMode::from_bits)")]
    pub i_mode: FileMode, // File mode
    pub i_uid: u16,         // Low 16 bits of Owner Uid
    pub i_size_lo: u32,     // Size in bytes (low 32 bits)
    pub i_atime: u32,       // Access time
    pub i_ctime: u32,       // Inode change time
    pub i_mtime: u32,       // Modification time
    pub i_dtime: u32,       // Deletion time
    pub i_gid: u16,         // Low 16 bits of Group Id
    pub i_links_count: u16, // Links count
    pub i_blocks_lo: u32,   // Blocks count (low)
    #[nom(Parse = "map(le_u32, InodeFlags::from_bits_truncate)")]
    pub i_flags: InodeFlags, // File flags
    pub osd1: Osd1Linux1,   // OS-dependent 1
    pub i_block: [u32; EXT4_N_BLOCKS], // Pointers to blocks
    pub i_generation: u32,  // File version (for NFS)
    pub i_file_acl_lo: u32, // File ACL (low 32 bits)
    pub i_size_high: u32,   // Size in bytes (high 32 bits)
    pub i_obso_faddr: u32,  // Obsoleted fragment address
    pub osd2: Osd2Linux2,   // OS-dependent 2
    pub i_extra_isize: u16, // Extra inode size
    pub i_checksum_hi: u16, // CRC32C checksum high part
    pub i_ctime_extra: u32, // Change time (extra nsec, epoch)
    pub i_mtime_extra: u32, // Modification time (extra nsec, epoch)
    pub i_atime_extra: u32, // Access time (extra nsec, epoch)
    pub i_crtime: u32,      // Creation time
    pub i_crtime_extra: u32, // Creation time extra
    pub i_version_hi: u32,  // High 32 bits for 64-bit version
    pub i_projid: u32,      // Project ID
}

fn read_extent_node(input: &[u8]) -> &Ext4ExtentHeader {
    unsafe { &*((input as *const [u8]) as *const u32 as *const Ext4ExtentHeader)}
}

impl Ext4Inode {
    pub fn get_extend_header(&self) -> &Ext4ExtentHeader {
        unsafe { &*((&self.i_block as *const u32) as *const Ext4ExtentHeader) }
    }

    pub fn read_block<'a, 'b, T>(&'a self, input: &'b [u8], block_size: usize, next_fn: fn(&T) -> usize) -> BufferChainer<'b, T> {
        // TODO: use logical block index
        let header = self.get_extend_header();
        println!("heaer is {:?}", header);
        let buffers = if header.eh_depth == 0 {
            let extents = unsafe {
                &*slice_from_raw_parts((&self.i_block as *const u32) as *const Ext4Extent, 5)
            };
            Self::read_extent(input, &extents[1..(1 + header.eh_entries as usize)], block_size)
        } else {
            let indices = unsafe {
                &*slice_from_raw_parts((&self.i_block as *const u32) as *const Ext4ExtentIdx, 5)
            };
            Self::read_e_index(input, &indices[1..(1 + header.eh_entries as usize)], block_size)
        };
        BufferChainer::new(buffers, next_fn)
    }

    fn read_extent<'a, 'b>(input: &'b [u8], extents: &'a [Ext4Extent], block_size: usize) -> Vec<&'b [u8]> {
        let mut ret = Vec::new();
        for extent in extents {
            let start = block_size * (extent.ee_start() as usize);
            let end = start + ((extent.ee_len as usize) * block_size);
            ret.push(&input[start..end])
        }
        ret
    }

    fn read_e_index<'a, 'b>(input: &'b [u8], indices: &'a [Ext4ExtentIdx], block_size: usize) -> Vec<&'b [u8]> {
        let mut ret = Vec::new();
        for index in indices {
            let start = index.ei_leaf() as usize * block_size;
            let header = read_extent_node(&input[start..]);
            println!("{:?}", header);
            if header.eh_entries == 0 {
                break;
            }
            let rest_start = start + size_of::<Ext4ExtentHeader>();
            let rest = &input[rest_start .. rest_start + size_of::<Ext4ExtentHeader>() * header.eh_entries as usize];
            println!("rest len is {}", rest.len());

            let mut res = if header.eh_depth == 0 {
                let extents = unsafe {
                    &*(slice_from_raw_parts(rest.as_ptr() as *const Ext4Extent, header.eh_entries as usize))
                };
                Self::read_extent(input, extents, block_size)
            } else {
                let extents = unsafe {
                    &*(slice_from_raw_parts(rest.as_ptr() as *const Ext4ExtentIdx, header.eh_entries as usize))
                };
                Self::read_e_index(input, extents, block_size)
            };
            ret.append(&mut res);
        }
        ret
    }

    pub fn get_extents(&self) -> Option<&[Ext4Extent]> {
        let header = self.get_extend_header();
        if !header.is_header() {
            return None;
        }
        let base = unsafe {
            &*(slice_from_raw_parts((&self.i_block as *const u32) as *const Ext4Extent, 5))
        };
        if header.eh_depth == 0 {
            Some(&base[1..(1 + header.eh_entries as usize)])
        } else {
            None
        }
    }

    pub fn get_extent_indicies(&self) -> Option<&[Ext4ExtentIdx]> {
        let header = self.get_extend_header();
        if !header.is_header() {
            return None;
        }
        let base = unsafe {
            &*(slice_from_raw_parts((&self.i_block as *const u32) as *const Ext4ExtentIdx, 5))
        };
        if header.eh_depth != 0 {
            Some(&base[1..(1 + header.eh_entries as usize)])
        } else {
            None
        }
    }

    pub fn get_i_block_contents<'a, 'b>(&'a self, input: &'b [u8], block_size: usize) -> Option<BlockContents<'b>> {
        // only support for Dir, Regular and Symlink.
        if self.i_mode.ty.is_symlink() {
            let bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                self.i_block.as_ptr() as *const u8,
                &self.i_block.len() * size_of::<u32>(),
                )
            };
            Some(BlockContents::InliedData(bytes))
        } else if self.i_mode.ty.is_regular() {
            let buffer_chainer = self.read_block(input, block_size, |&_| {1});
            Some(BlockContents::Data(buffer_chainer))
        } else if self.i_mode.ty.is_dir() {
            let buffer_chainer = self.read_block(input, block_size, |&d_entry: &Ext4DirEntry| {
                d_entry.rec_len as usize
            });
            Some(BlockContents::Dentries(buffer_chainer))
        } else {
            None
        }
    }
}

pub enum BlockContents<'a> {
    InliedData(&'a [u8]),
    Dentries(BufferChainer<'a, Ext4DirEntry>),
    Data(BufferChainer<'a, u8>)
}

pub enum IBlockRest<'a> {
    Indices(&'a [Ext4ExtentIdx]),
    Extents(&'a [Ext4Extent])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Fifo,     // 0x1000
    CharDev,  // 0x2000
    Dir,      // 0x4000
    BlockDev, // 0x6000
    Regular,  // 0x8000
    Symlink,  // 0xA000
    Socket,   // 0xC000
    Unknown(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode {
    pub perms: FilePermissions,
    pub ty: FileType,
}

impl FileMode {
    pub fn from_bits(bits: u16) -> Self {
        Self {
            perms: FilePermissions::from_bits_truncate(bits),
            ty: FileType::from_bits(bits),
        }
    }
}

impl FileType {
    fn from_bits(bits: u16) -> Self {
        match bits & 0xF000 {
            0x1000 => FileType::Fifo,
            0x2000 => FileType::CharDev,
            0x4000 => FileType::Dir,
            0x6000 => FileType::BlockDev,
            0x8000 => FileType::Regular,
            0xA000 => FileType::Symlink,
            0xC000 => FileType::Socket,
            other => FileType::Unknown(other),
        }
    }

    pub fn is_regular(&self) -> bool {
        *self == Self::Regular
    }

    pub fn is_symlink(&self) -> bool {
        *self == Self::Symlink
    }

    pub fn is_dir(&self) -> bool {
        *self == Self::Dir
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Nom)]
/// linux-specific fields for osd1
pub struct Osd1Linux1 {
    pub l_i_version: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// hurd-specific fields for osd1
pub struct Osd1Hurd1 {
    pub h_i_translator: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// masix-specific fields for osd1
pub struct Osd1Masix1 {
    pub m_i_reserved1: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Nom)]
/// linux-specific fields for osd2
pub struct Osd2Linux2 {
    pub l_i_blocks_high: u16,
    pub l_i_file_acl_high: u16,
    pub l_i_uid_high: u16,
    pub l_i_gid_high: u16,
    pub l_i_checksum_lo: u16,
    pub l_i_reserved: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// hurd-specific fields for osd2
pub struct Osd2Hurd2 {
    pub h_i_reserved1: u16,
    pub h_i_mode_high: u16,
    pub h_i_uid_high: u16,
    pub h_i_gid_high: u16,
    pub h_i_author: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// masix-specific fields for osd2
pub struct Osd2Masix2 {
    pub h_i_reserved1: u16,
    pub m_i_file_acl_high: u16,
    pub m_i_reserved2: [u32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct Ext4GroupDesc {
    /// Blocks bitmap block (low 32 bits)
    pub bg_block_bitmap_lo: u32,
    /// Inodes bitmap block (low 32 bits)
    pub bg_inode_bitmap_lo: u32,
    /// Inodes table block (low 32 bits)
    pub bg_inode_table_lo: u32,
    /// Free blocks count (low 16 bits)
    pub bg_free_blocks_count_lo: u16,
    /// Free inodes count (low 16 bits)
    pub bg_free_inodes_count_lo: u16,
    /// Directories count (low 16 bits)
    pub bg_used_dirs_count_lo: u16,
    /// EXT4_BG_flags (e.g., INODE_UNINIT)
    #[nom(Parse = "map(le_u16, BgFlags::from_bits_truncate)")]
    pub bg_flags: BgFlags,
    /// Exclude bitmap for snapshots (low 32 bits)
    pub bg_exclude_bitmap_lo: u32,
    /// CRC32C of block bitmap (low 16 bits, little endian)
    pub bg_block_bitmap_csum_lo: u16,
    /// CRC32C of inode bitmap (low 16 bits, little endian)
    pub bg_inode_bitmap_csum_lo: u16,
    /// Unused inodes count (low 16 bits)
    pub bg_itable_unused_lo: u16,
    /// CRC16 of the group descriptor
    pub bg_checksum: u16,

    /// Blocks bitmap block (high 32 bits)
    pub bg_block_bitmap_hi: u32,
    /// Inodes bitmap block (high 32 bits)
    pub bg_inode_bitmap_hi: u32,
    /// Inodes table block (high 32 bits)
    pub bg_inode_table_hi: u32,
    /// Free blocks count (high 16 bits)
    pub bg_free_blocks_count_hi: u16,
    /// Free inodes count (high 16 bits)
    pub bg_free_inodes_count_hi: u16,
    /// Directories count (high 16 bits)
    pub bg_used_dirs_count_hi: u16,
    /// Unused inodes count (high 16 bits)
    pub bg_itable_unused_hi: u16,
    /// Exclude bitmap block (high 32 bits)
    pub bg_exclude_bitmap_hi: u32,
    /// CRC32C of block bitmap (high 16 bits, big endian)
    pub bg_block_bitmap_csum_hi: u16,
    /// CRC32C of inode bitmap (high 16 bits, big endian)
    pub bg_inode_bitmap_csum_hi: u16,

    /// Reserved for future use
    pub bg_reserved: u32,
}

const EXT4_NAME_LEN: usize = 255;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4DirEntry {
    /// Inode number
    pub inode: u32,
    /// Directory entry length (actual size of this entry in the block)
    pub rec_len: u16,
    /// Length of the name in bytes
    pub name_len: u8,

    /// 
    pub file_type: u8,
    /// File name (not NUL‑terminated; valid bytes are up to `name_len`)
    pub name: [u8; EXT4_NAME_LEN],
}

impl Ext4DirEntry {
    pub fn get_name(&self) -> Option<String> {
        let name_end = self.name_len as usize;
        Some(String::from_utf8(self.name[0..name_end].to_vec()).unwrap())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct Ext4ExtentHeader {
    /// magic number (always little-endian in the on-disk format)
    pub eh_magic: u16,
    /// number of valid entries
    pub eh_entries: u16,
    /// capacity (max entries)
    pub eh_max: u16,
    /// depth of the tree (0 = leaf level only)
    pub eh_depth: u16,
    /// generation of the tree
    pub eh_generation: u32,
}

impl Ext4ExtentHeader {
    pub fn is_header(&self) -> bool {
        self.eh_magic == 0xf30a
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct Ext4ExtentIdx {
    /// first logical block this index covers
    pub ei_block: u32,
    /// low 32 bits of the physical block pointer
    pub ei_leaf_lo: u32,
    /// high 16 bits of the physical block pointer
    pub ei_leaf_hi: u16,
    /// unused / padding
    pub ei_unused: u16,
}

impl Ext4ExtentIdx {
    pub fn ei_leaf(&self) -> u64 {
        ((self.ei_leaf_hi as u64) << 32) | self.ei_leaf_lo as u64
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct Ext4Extent {
    /// first logical block this extent covers
    pub ee_block: u32,
    /// number of blocks covered
    pub ee_len: u16,
    /// high-16 bits of the physical block start
    pub ee_start_hi: u16,
    /// low-32 bits of the physical block start
    pub ee_start_lo: u32,
}

impl Ext4Extent {
    pub fn ee_start(&self) -> u64 {
        ((self.ee_start_hi as u64) << 32) | self.ee_start_lo as u64
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Nom)]
#[nom(LittleEndian)]
pub struct Ext4ExtentTail {
    /// CRC32C(uuid + inode # + extent block)
    pub et_checksum: u32,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct CompatFeatures: u32 {
        /// 0x0000_0001: Directory preallocation
        const DIR_PREALLOC         = 0x0000_0001;
        /// 0x0000_0002: “imagic inodes”
        const IMAGIC_INODES        = 0x0000_0002;
        /// 0x0000_0004: Has a journal
        const HAS_JOURNAL          = 0x0000_0004;
        /// 0x0000_0008: Supports extended attributes
        const EXT_ATTR             = 0x0000_0008;
        /// 0x0000_0010: Reserved GDT blocks for expansion
        const RESIZE_INODE         = 0x0000_0010;
        /// 0x0000_0020: Has directory indices
        const DIR_INDEX            = 0x0000_0020;
        /// 0x0000_0040: “Lazy BG”
        const LAZY_BG              = 0x0000_0040;
        /// 0x0000_0080: “Exclude inode”
        const EXCLUDE_INODE        = 0x0000_0080;
        /// 0x0000_0100: “Exclude bitmap”
        const EXCLUDE_BITMAP       = 0x0000_0100;
        /// 0x0000_0200: Sparse Super2
        const SPARSE_SUPER2        = 0x0000_0200;
        /// 0x0000_0400: Fast commits supported
        const FAST_COMMIT          = 0x0000_0400;
        /// 0x0000_1000: Orphan file allocated
        const ORPHAN_PRESENT       = 0x0000_1000;
    }
    #[derive(Default, Debug, Clone, Copy)]
    pub struct IncompatFeatures: u32 {
        /// 0x0000_0001: Compression (INCOMPAT_COMPRESSION)
        const COMPRESSION      = 0x0000_0001;
        /// 0x0000_0002: Directory entries record the file type (INCOMPAT_FILETYPE)
        const FILETYPE         = 0x0000_0002;
        /// 0x0000_0004: Filesystem needs recovery (INCOMPAT_RECOVER)
        const RECOVER          = 0x0000_0004;
        /// 0x0000_0008: Separate journal device (INCOMPAT_JOURNAL_DEV)
        const JOURNAL_DEV      = 0x0000_0008;
        /// 0x0000_0010: Meta block groups (INCOMPAT_META_BG)
        const META_BG          = 0x0000_0010;
        /// 0x0000_0040: Files use extents (INCOMPAT_EXTENTS)
        const EXTENTS          = 0x0000_0040;
        /// 0x0000_0080: 64-bit block numbers (INCOMPAT_64BIT)
        const _64BIT           = 0x0000_0080;
        /// 0x0000_0100: Multiple mount protection (INCOMPAT_MMP)
        const MMP              = 0x0000_0100;
        /// 0x0000_0200: Flexible block groups (INCOMPAT_FLEX_BG)
        const FLEX_BG          = 0x0000_0200;
        /// 0x0000_0400: EA stored in data blocks (INCOMPAT_EA_INODE)
        const EA_INODE         = 0x0000_0400;
        /// 0x0000_1000: Data in directory entry (INCOMPAT_DIRDATA)
        const DIRDATA          = 0x0000_1000;
        /// 0x0000_2000: Metadata checksum seed in superblock (INCOMPAT_CSUM_SEED)
        const CSUM_SEED        = 0x0000_2000;
        /// 0x0000_4000: Large directory / 3-level htree (INCOMPAT_LARGEDIR)
        const LARGEDIR         = 0x0000_4000;
        /// 0x0000_8000: Data in inode (INCOMPAT_INLINE_DATA)
        const INLINE_DATA      = 0x0000_8000;
        /// 0x0001_0000: Encrypted inodes present (INCOMPAT_ENCRYPT)
        const ENCRYPT          = 0x0001_0000;
    }
    #[derive(Default, Debug, Clone, Copy)]
    pub struct RoCompatFeatures: u32 {
        /// 0x0000_0001: Sparse superblocks (RO_COMPAT_SPARSE_SUPER)
        const SPARSE_SUPER      = 0x0000_0001;
        /// 0x0000_0002: Files >2GiB (RO_COMPAT_LARGE_FILE)
        const LARGE_FILE        = 0x0000_0002;
        /// 0x0000_0004: B-tree directories (unused) (RO_COMPAT_BTREE_DIR)
        const BTREE_DIR         = 0x0000_0004;
        /// 0x0000_0008: Huge-file units (RO_COMPAT_HUGE_FILE)
        const HUGE_FILE         = 0x0000_0008;
        /// 0x0000_0010: Group descriptor checksums (RO_COMPAT_GDT_CSUM)
        const GDT_CSUM          = 0x0000_0010;
        /// 0x0000_0020: No subdir link limit (RO_COMPAT_DIR_NLINK)
        const DIR_NLINK         = 0x0000_0020;
        /// 0x0000_0040: Extra inode size (RO_COMPAT_EXTRA_ISIZE)
        const EXTRA_ISIZE       = 0x0000_0040;
        /// 0x0000_0080: Snapshot present (RO_COMPAT_HAS_SNAPSHOT)
        const HAS_SNAPSHOT      = 0x0000_0080;
        /// 0x0000_0100: Quota (RO_COMPAT_QUOTA)
        const QUOTA             = 0x0000_0100;
        /// 0x0000_0200: Bigalloc (RO_COMPAT_BIGALLOC)
        const BIGALLOC          = 0x0000_0200;
        /// 0x0000_0400: Metadata checksumming (RO_COMPAT_METADATA_CSUM)
        const METADATA_CSUM     = 0x0000_0400;
        /// 0x0000_0800: Replicas (unused) (RO_COMPAT_REPLICA)
        const REPLICA           = 0x0000_0800;
        /// 0x0000_1000: Read-only image (RO_COMPAT_READONLY)
        const READONLY          = 0x0000_1000;
        /// 0x0000_2000: Project quotas (RO_COMPAT_PROJECT)
        const PROJECT           = 0x0000_2000;
        /// 0x0000_8000: Verity inodes (RO_COMPAT_VERITY)
        const VERITY            = 0x0000_8000;
        /// 0x0001_0000: Orphan cleanup needed (RO_COMPAT_ORPHAN_PRESENT)
        const ORPHAN_PRESENT    = 0x0001_0000;
    }
    #[derive(Default, Debug, Clone, Copy)]
    pub struct BgFlags: u16 {
        /// 0x0001: inode table and bitmap are not initialized
        const INODE_UNINIT = 0x0001;
        /// 0x0002: block bitmap is not initialized
        const BLOCK_UNINIT = 0x0002;
        /// 0x0004: inode table is zeroed
        const INODE_ZEROED = 0x0004;
    }
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FilePermissions: u16 {
        // --- Others permissions ---
        const S_IXOTH = 0x0001; // execute by others
        const S_IWOTH = 0x0002; // write by others
        const S_IROTH = 0x0004; // read by others

        // --- Group permissions ---
        const S_IXGRP = 0x0008; // execute by group
        const S_IWGRP = 0x0010; // write by group
        const S_IRGRP = 0x0020; // read by group

        // --- Owner permissions ---
        const S_IXUSR = 0x0040; // execute by owner
        const S_IWUSR = 0x0080; // write by owner
        const S_IRUSR = 0x0100; // read by owner

        // --- Special mode bits ---
        const S_ISVTX = 0x0200; // sticky
        const S_ISGID = 0x0400; // set GID
        const S_ISUID = 0x0800; // set UID

        // --- File type bits (mutually exclusive) ---
        const S_IFIFO  = 0x1000; // FIFO (named pipe)
        const S_IFCHR  = 0x2000; // character device
        const S_IFDIR  = 0x4000; // directory
        const S_IFBLK  = 0x6000; // block device
        const S_IFREG  = 0x8000; // regular file
        const S_IFLNK  = 0xA000; // symbolic link
        const S_IFSOCK = 0xC000; // socket
    }

    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct InodeFlags: u32 {
        const SECURE_DELETE        = 0x0000_0001;
        const PRESERVE_UNDELETE    = 0x0000_0002;
        const COMPRESSED           = 0x0000_0004;
        const SYNC                 = 0x0000_0008;
        const IMMUTABLE            = 0x0000_0010;
        const APPEND_ONLY          = 0x0000_0020;
        const NO_DUMP              = 0x0000_0040;
        const NO_ATIME             = 0x0000_0080;
        const DIRTY_COMPRESSED     = 0x0000_0100;
        const COMPRESSED_CLUSTERS  = 0x0000_0200;
        const NO_COMPRESS          = 0x0000_0400;
        const ENCRYPTED            = 0x0000_0800;
        const INDEX                = 0x0000_1000;
        const IMAGIC               = 0x0000_2000;
        const JOURNAL_DATA         = 0x0000_4000;
        const NO_TAIL              = 0x0000_8000;
        const DIRSYNC              = 0x0001_0000;
        const TOPDIR               = 0x0002_0000;
        const HUGE_FILE            = 0x0004_0000;
        const EXTENTS              = 0x0008_0000;
        const VERITY               = 0x0010_0000;
        const EA_INODE             = 0x0020_0000;
        const EOF_BLOCKS           = 0x0040_0000;
        const SNAPFILE             = 0x0100_0000;
        const SNAPFILE_DELETED     = 0x0400_0000;
        const SNAPFILE_SHRUNK      = 0x0800_0000;
        const INLINE_DATA          = 0x1000_0000;
        const PROJINHERIT          = 0x2000_0000;
        const RESERVED             = 0x8000_0000;
    }
}
