use nom::IResult;
use nom::{le_u64,le_u32,le_u16};
use block::RawBlock;
use options::{parse_options,Options};

pub const TY: u32 = 0x0A0D0D0A;

//    0                   1                   2                   3
//    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +---------------------------------------------------------------+
//  0 |                   Block Type = 0x0A0D0D0A                     |
//    +---------------------------------------------------------------+
//  4 |                      Block Total Length                       |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  8 |                      Byte-Order Magic                         |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 12 |          Major Version        |         Minor Version         |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 16 |                                                               |
//    |                          Section Length                       |
//    |                                                               |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 24 /                                                               /
//    /                      Options (variable)                       /
//    /                                                               /
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                      Block Total Length                       |
//    +---------------------------------------------------------------+

named!(section_header_body<&[u8],SectionHeader>,
       chain!(
           magic: le_u32 ~
           major_version: le_u16 ~
           minor_version: le_u16 ~
           _section_length: le_u64 ~
           options: parse_options?,

           // Can we get the blocks by virtue of knowing how much data we have left here?
           ||{
               let section_length = if _section_length == 0xFFFFFFFFFFFFFFFF {
                   SectionLength::Unspecified
               } else {
                   SectionLength::Bytes(_section_length)
               };

               assert_eq!(magic, 0x1A2B3C4D);
               SectionHeader {
                   ty: TY,
                   block_length: 0,
                   magic: magic,
                   major_version: major_version,
                   minor_version: minor_version,
                   section_length: section_length,
                   options: options,
                   check_length: 0,
           } }
           )
      );

#[derive(PartialEq,Debug)]
pub enum SectionLength {
    Bytes(u64),
    Unspecified,
}

#[derive(Debug)]
pub struct SectionHeader<'a> {
    pub ty: u32,
    pub block_length: u32,
    pub magic: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub section_length: SectionLength,
    pub options: Option<Options<'a>>,
    pub check_length: u32,
}

pub fn parse(blk: RawBlock) -> SectionHeader {
    // TODO(richo) Actually parse out the options afterward
    // I think that we can do this by invoking an options parser, and using the fact that we're
    // dealing with slices by this point to our advantage
    match section_header_body(blk.body) {
        // FIXME(richo) actually do smeometing with the leftover bytes
        IResult::Done(_, mut block) => {
            block.block_length = blk.block_length;
            block.check_length = blk.check_length;
            block
        },
        _ => {
            panic!("Couldn't unpack this section_header");
        }
    }
}

#[cfg(test)]
use block::parse_block;

#[test]
fn test_parse_section_header() {
    let input = b"\n\r\r\n\x1c\x00\x00\x00M<+\x1a\x01\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\x1c\x00\x00\x00";
    match parse_block(input) {
        IResult::Done(left, block) => {
            let section_header = parse(block);

            // Ignored because we do not currently parse the whole block
            assert_eq!(left, b"");
            assert_eq!(section_header.ty, 0x0A0D0D0A);
            assert_eq!(section_header.block_length, 28);
            assert_eq!(section_header.magic, 0x1A2B3C4D);
            assert_eq!(section_header.section_length, SectionLength::Unspecified);
            assert!(section_header.options.is_none());
            assert_eq!(section_header.check_length, 28);
        },
        _ => {
            assert_eq!(1, 2);
        },
    }
}
