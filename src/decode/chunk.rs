use nom::{
    bits::complete::tag as bits_tag,
    bits::complete::take as bits_take,
    branch::alt,
    error::{make_error, ErrorKind},
    number::complete::be_u8,
    sequence::tuple,
    Err as NomErr, IResult,
};

const QOI_RGB_CHUNK_TAG: u8 = 0b11111110;
const QOI_RGBA_CHUNK_TAG: u8 = 0b11111111;
const QOI_OP_INDEX_CHUNK_TAG: usize = 0b00;
const QOI_OP_DIFF_CHUNK_TAG: usize = 0b01;
const QOI_OP_LUMA_CHUNK_TAG: usize = 0b10;
const QOI_OP_RUN_CHUNK_TAG: usize = 0b11;

#[derive(Debug, PartialEq, Eq)]
pub struct QOIRGBChunk {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QOIRGBAChunk {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QOIOpIndexChunk {
    pub index: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QOIOpDiffChunk {
    pub diff_r: u8,
    pub diff_g: u8,
    pub diff_b: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QOIOpLumaChunk {
    pub diff_g: u8,
    pub dr_dg: u8,
    pub db_dg: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QOIOpRunChunk {
    pub run: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum QOIChunk {
    QOIRGBChunk(QOIRGBChunk),
    QOIRGBAChunk(QOIRGBAChunk),
    QOIOpIndexChunk(QOIOpIndexChunk),
    QOIOpDiffChunk(QOIOpDiffChunk),
    QOIOpLumaChunk(QOIOpLumaChunk),
    QOIOpRunChunk(QOIOpRunChunk),
}

pub fn parse_rgb_chunk(input: &[u8]) -> IResult<&[u8], QOIChunk> {
    let (i, tag) = be_u8(input)?;

    if tag != QOI_RGB_CHUNK_TAG {
        return Err(NomErr::Error(make_error(i, ErrorKind::Tag)));
    }

    let (i, (r, g, b)) = tuple((be_u8, be_u8, be_u8))(i)?;

    let chunk = QOIRGBChunk { r, g, b };

    Ok((i, QOIChunk::QOIRGBChunk(chunk)))
}

pub fn parse_rgba_chunk(input: &[u8]) -> IResult<&[u8], QOIChunk> {
    let (i, tag) = be_u8(input)?;

    if tag != QOI_RGBA_CHUNK_TAG {
        return Err(NomErr::Error(make_error(i, ErrorKind::Tag)));
    }

    let (i, (r, g, b, a)) = tuple((be_u8, be_u8, be_u8, be_u8))(i)?;
    let chunk = QOIRGBAChunk { r, g, b, a };

    Ok((i, QOIChunk::QOIRGBAChunk(chunk)))
}

pub fn parse_op_index_chunk(input: &[u8]) -> IResult<&[u8], QOIChunk> {
    // FIXME: error handling
    let result: IResult<(&[u8], usize), usize> =
        bits_tag(QOI_OP_INDEX_CHUNK_TAG, 2usize)((input, 0));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), _) = result.unwrap();

    // FIXME: error handling
    let result: IResult<(&[u8], usize), u8> = bits_take(6usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), index) = result.unwrap();

    assert_eq!(offset, 0);

    let chunk = QOIOpIndexChunk { index };

    Ok((i, QOIChunk::QOIOpIndexChunk(chunk)))
}

pub fn parse_op_diff_chunk(input: &[u8]) -> IResult<&[u8], QOIChunk> {
    const BIAS: u8 = 2;
    let result: IResult<(&[u8], usize), usize> =
        bits_tag(QOI_OP_DIFF_CHUNK_TAG, 2usize)((input, 0));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), _) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(2usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), diff_r) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(2usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), diff_g) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(2usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), diff_b) = result.unwrap();

    assert_eq!(offset, 0);

    let chunk = QOIOpDiffChunk {
        diff_r: diff_r - BIAS,
        diff_g: diff_g - BIAS,
        diff_b: diff_b - BIAS,
    };

    Ok((i, QOIChunk::QOIOpDiffChunk(chunk)))
}

pub fn parse_op_luma_chunk(input: &[u8]) -> IResult<&[u8], QOIChunk> {
    const BIAS: u8 = 8;
    const GREEN_BIAS: u8 = 32;
    let result: IResult<(&[u8], usize), usize> =
        bits_tag(QOI_OP_LUMA_CHUNK_TAG, 2usize)((input, 0));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), _) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(6usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), diff_g) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(4usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), dr_dg) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(4usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), db_dg) = result.unwrap();

    assert_eq!(offset, 0);

    let chunk = QOIOpLumaChunk {
        diff_g: diff_g,
        dr_dg: dr_dg,
        db_dg: db_dg,
    };

    Ok((i, QOIChunk::QOIOpLumaChunk(chunk)))
}

pub fn parse_op_run_chunk(input: &[u8]) -> IResult<&[u8], QOIChunk> {
    const BIAS: u8 = 1; // will add bias 1 instead of subbing bias -1
    let result: IResult<(&[u8], usize), usize> =
        bits_tag(QOI_OP_RUN_CHUNK_TAG, 2usize)((input, 0));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), _) = result.unwrap();

    let result: IResult<(&[u8], usize), u8> = bits_take(6usize)((i, offset));
    if result.is_err() {
        return Err(NomErr::Error(make_error(input, ErrorKind::Tag)));
    }
    let ((i, offset), run) = result.unwrap();

    assert_eq!(offset, 0);

    let chunk = QOIOpRunChunk { run: run + BIAS };

    Ok((i, QOIChunk::QOIOpRunChunk(chunk)))
}

impl QOIChunk {
    pub fn parse(bytes: &[u8]) -> IResult<&[u8], QOIChunk> {
        alt((
            parse_op_index_chunk,
            parse_rgb_chunk,
            parse_rgba_chunk,
            parse_op_diff_chunk,
            parse_op_luma_chunk,
            parse_op_run_chunk,
        ))(bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rgb_chunk() {
        let bytes = [0xFE, 0xA3, 0x89, 0x43];
        let chunk = QOIChunk::QOIRGBChunk(QOIRGBChunk {
            r: 0xA3,
            g: 0x89,
            b: 0x43,
        });

        let result = QOIChunk::parse(bytes.as_ref()).expect("failed to parse rgb chunk");
        let res_chunk = result.1;

        assert_eq!(res_chunk, chunk);
    }

    #[test]
    fn test_rgba_chunk() {
        let bytes = [0xFF, 0x34, 0xFE, 0x9E, 0x90];
        let chunk = QOIChunk::QOIRGBAChunk(QOIRGBAChunk {
            r: 0x34,
            g: 0xFE,
            b: 0x9E,
            a: 0x90,
        });

        let result = QOIChunk::parse(bytes.as_ref()).expect("failed to parse rgb chunk");
        let res_chunk = result.1;

        assert_eq!(res_chunk, chunk);
    }

    #[test]
    fn test_op_index_chunk() {
        let bytes = [0b00000001];
        let chunk = QOIChunk::QOIOpIndexChunk(QOIOpIndexChunk { index: 1 });

        let result = QOIChunk::parse(bytes.as_ref()).expect("failed to parse op index chunk");
        let res_chunk = result.1;

        assert_eq!(res_chunk, chunk);
    }
}
