use crate::decode::chunk::*;
use crate::decode::header::*;

use nom::bytes::complete::tag;
use nom::multi::many_till;
use nom::sequence::tuple;
use nom::IResult;
use rgb::RGBA;

const INDEX_ARRAY_LENGTH: usize = 64;

#[allow(dead_code)]
#[derive(Debug)]
pub struct QOI {
    pub header: QOIHeader,
    pub chunks: Vec<QOIChunk>,
}

pub fn end_marker(input: &[u8]) -> IResult<&[u8], ()> {
    let (i, _) = tag([0, 0, 0, 0, 0, 0, 0, 1])(input)?;

    Ok((i, ()))
}

impl QOI {
    pub fn parse(bytes: &[u8]) -> IResult<&[u8], Self> {
        // let (i, (header, chunks)) = tuple((dbg_dmp(QOIHeader::parse, "header"), many1(dbg_dmp(QOIChunk::parse, "chunk"))))(bytes)?;
        let (i, (header, (chunks, _))) =
            tuple((QOIHeader::parse, many_till(QOIChunk::parse, end_marker)))(bytes)?;
        let qoi = QOI { header, chunks };

        Ok((i, qoi))
    }

    pub fn into_pixels(&self) -> Vec<RGBA<u8>> {
        let capacity = self.header.height() * self.header.width();
        let mut pixels = Vec::with_capacity(capacity as usize);

        let mut running_hash = [None; INDEX_ARRAY_LENGTH];

        self.chunks.iter().fold(
            (0, RGBA::new(0, 0, 0, 255)),
            |(mut index, prev_pixel): (usize, _), chunk| {
                match chunk {
                    QOIChunk::QOIRGBChunk(chunk) => {
                        pixels.push(RGBA::new(chunk.r, chunk.g, chunk.b, prev_pixel.a));
                    }
                    QOIChunk::QOIRGBAChunk(chunk) => {
                        pixels.push(RGBA::new(chunk.r, chunk.g, chunk.b, chunk.a));
                    }
                    QOIChunk::QOIOpIndexChunk(chunk) => {
                        let pixel = running_hash[chunk.index as usize]
                            .expect("did not found hash at expected position");
                        pixels.push(pixel);
                    }
                    QOIChunk::QOIOpDiffChunk(chunk) => {
                        const BIAS: u8 = 2;
                        let (r, g, b) = (
                            prev_pixel.r.wrapping_add(chunk.diff_r),
                            prev_pixel.g.wrapping_add(chunk.diff_g),
                            prev_pixel.b.wrapping_add(chunk.diff_b),
                        );
                        pixels.push(RGBA::new(r, g, b, prev_pixel.a));
                    }
                    QOIChunk::QOIOpLumaChunk(chunk) => {
                        const BIAS: u8 = 8;
                        const GREEN_BIAS: u8 = 32;
                        let vg = chunk.diff_g;
                        let r = prev_pixel.r.wrapping_add(vg).wrapping_add(chunk.dr_dg).wrapping_sub(BIAS);
                        let g = prev_pixel.g.wrapping_add(vg).wrapping_sub(GREEN_BIAS);
                        let b = prev_pixel.b.wrapping_add(vg).wrapping_add(chunk.db_dg).wrapping_sub(BIAS);

                        pixels.push(RGBA::new(r, g, b, prev_pixel.a));
                    }
                    QOIChunk::QOIOpRunChunk(chunk) => {
                        const BIAS: usize = 1;
                        let length: usize = (chunk.run as usize).wrapping_add(BIAS);
                        for _ in 0..(length) {
                            pixels.push(prev_pixel);
                        }

                        index = index + (chunk.run as usize) - 1;
                    }
                }

                let p: [u8; 4] = pixels[index].into();
                let (r, g, b, a) = (p[0] as usize, p[1] as usize, p[2] as usize, p[3] as usize);
                let hash_position= (r * 3 + g * 5 + b * 7 + a * 11) % 64;

                running_hash[hash_position as usize] = Some(pixels[index]);

                (index + 1, pixels[index])
            },
        );

        pixels
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;

    #[test]
    pub fn test_end_marker() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 1];
        let result = end_marker(bytes.as_ref());

        assert!(result.is_ok());
    }

    #[test]
    pub fn test_header_and_end() {
        let bytes = [
            0x71, 0x6F, 0x69, 0x66, // qoif
            0x00, 0x00, 0x00, 0x02, // height
            0x00, 0x00, 0x00, 0x02, // width
            0x04, // channels
            0x00, // colorspace
            0xFE, 0xFF, 0xFF, 0xFF, // (255, 255, 255) rgb chunk
            0xFE, 0xFF, 0xFF, 0xFF, // (255, 255, 255) rgb chunk
            0xFE, 0xFF, 0xFF, 0xFF, // (255, 255, 255) rgb chunk
            0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // end marker
        ];

        let res = QOI::parse(bytes.as_ref());

        println!("{:?}", &res);

        assert!(res.is_ok());
    }
}
