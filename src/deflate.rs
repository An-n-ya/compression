#![allow(unused)]

use std::{
    collections::LinkedList,
    io::{BufReader, Read},
    time::SystemTime,
};

use crate::bit_io::{BitIO, Code, Numeric, Reader};

const GZIP_ID1: u8 = 0x1f;
const GZIP_ID2: u8 = 0x8b;
const DEFLATE_METHOD: u8 = 8;

// refer to https://www.rfc-editor.org/rfc/rfc1952.pdf S2.3
struct GZipHeader {
    id1: u8,
    id2: u8,
    compression_method: u8,
    flag: u8,
    modification_time: u32,
    extra_flag: u8,
    os: u8,
}

struct GZipFooter {
    crc32: u32,
    input_size: u32,
}

struct GZip {
    header: GZipHeader,
    footer: GZipFooter,
    compressed_block: Option<Vec<Block>>,
    bit_io: BitIO,
}

enum BlockType {
    NoCompression { len: u16 },
    FixedHuffCompression,
    DynamicHuffCompression,
    Error,
}

struct Block {
    is_final: bool,
    _type: BlockType,
    data: BitIO,
}

impl GZip {
    pub fn new(header: GZipHeader, footer: GZipFooter) -> Self {
        Self {
            header,
            footer,
            compressed_block: None,
            bit_io: BitIO::new(LinkedList::new()),
        }
    }

    pub fn deflate(input: &[u8]) -> BitIO {
        let header = GZipHeader {
            id1: GZIP_ID1,
            id2: GZIP_ID2,
            compression_method: DEFLATE_METHOD,
            flag: 0,
            modification_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            extra_flag: 0,
            os: 0x03,
        };
        let footer = GZipFooter {
            crc32: crc32fast::hash(&input),
            input_size: input.len() as u32,
        };
        let mut reader = Reader::new(input);
        let mut blocks = vec![];
        while !reader.is_empty() {
            blocks.push(Self::write_block(&mut reader));
        }

        let mut zip = Self::new(header, footer);

        if blocks.len() != 0 {
            zip.compressed_block = Some(blocks);
        }
        zip.write();
        zip.bit_io
    }

    fn write_block(reader: &mut Reader) -> Block {
        Self::write_no_compression_block(reader)
    }

    fn write_no_compression_block(reader: &mut Reader) -> Block {
        let mut len = 0u16;
        let mut bit_io = BitIO::new(LinkedList::new());
        while !reader.is_empty() {
            if len == u16::MAX {
                // hit max size, jump to next block
                return Block {
                    is_final: false,
                    _type: BlockType::NoCompression { len },
                    data: bit_io,
                };
            }
            let data = reader.read_u8().unwrap();
            len += 1;
            bit_io.write_byte_align(data);
        }
        // reader is empty, so this is the last block
        Block {
            is_final: true,
            _type: BlockType::NoCompression { len },
            data: bit_io,
        }
    }

    fn write(&mut self) {
        self.header.write(&mut self.bit_io);

        if let Some(blocks) = &mut self.compressed_block {
            for block in blocks {
                block.write(&mut self.bit_io);
            }
        }

        self.footer.write(&mut self.bit_io);
    }
}

impl BlockType {
    pub fn write(&self, bit_io: &mut BitIO) {
        match self {
            BlockType::NoCompression { len } => {
                // 00
                bit_io.write_bit_back(false);
                bit_io.write_bit_back(false);
                let n_len = !len;
                bit_io.write_u16_align_little_endian(*len);
                bit_io.write_u16_align_little_endian(n_len);
            }
            BlockType::FixedHuffCompression => {
                // 01
                bit_io.write_bit_back(true);
                bit_io.write_bit_back(false);
            }
            BlockType::DynamicHuffCompression => {
                // 10
                bit_io.write_bit_back(false);
                bit_io.write_bit_back(true);
            }
            BlockType::Error => {
                // 11
                bit_io.write_bit_back(true);
                bit_io.write_bit_back(true);
            }
        }
    }
}

impl Block {
    pub fn write(&mut self, bit_io: &mut BitIO) {
        if self.is_final {
            bit_io.write_bit_back(true);
        } else {
            bit_io.write_bit_back(false);
        }
        self._type.write(bit_io);
        bit_io.append_bit_io(&mut self.data);
    }
}

impl GZipHeader {
    pub fn write(&self, bit_io: &mut BitIO) {
        bit_io.write_byte_align(self.id1);
        bit_io.write_byte_align(self.id2);
        bit_io.write_byte_align(self.compression_method);
        bit_io.write_byte_align(self.flag);
        bit_io.write_u32_align_little_endian(self.modification_time);
        bit_io.write_byte_align(self.extra_flag);
        bit_io.write_byte_align(self.os);
    }
}

impl GZipFooter {
    pub fn write(&self, bit_io: &mut BitIO) {
        bit_io.write_u32_align_little_endian(self.crc32);
        bit_io.write_u32_align_little_endian(self.input_size);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_deflate_no_compression() {
        let n = GZip::deflate(b"h");
        println!("{n:?}");

        let data = n.as_vec();
        fs::write("no_compression.gz", data);
    }
}
