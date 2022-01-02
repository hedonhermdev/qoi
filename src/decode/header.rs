use nom::{
    bytes::complete::tag,
    error::context,
    number::complete::{be_u32, be_u8},
    sequence::tuple,
    IResult,
};

#[derive(PartialEq, Eq, Debug)]
pub struct QOIHeader {
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}

impl QOIHeader {
    pub fn parse(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag("qoif")(bytes)?;
        let (i, (width, height, channels, colorspace)) = context(
            "failed to parse header",
            tuple((be_u32, be_u32, be_u8, be_u8)),
        )(i)?;

        let header = QOIHeader {
            width,
            height,
            channels,
            colorspace,
        };

        Ok((i, header))
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn colorspace(&self) -> u8 {
        self.colorspace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_valid_header() {
        let bytes: [u8; 14] = [
            0x71, 0x6F, 0x69, 0x66, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x04, 0x00,
        ];
        let expected = QOIHeader {
            width: 0x200,
            height: 0x200,
            channels: 0x04,
            colorspace: 0x00,
        };

        let actual = QOIHeader::parse(bytes.as_ref());

        assert_eq!(actual, Ok(([].as_ref(), expected)));
    }

    #[test]
    pub fn test_invalid_header() {
        let bytes: [u8; 14] = [
            0x73, 0x6F, 0x69, 0x66, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x04, 0x00,
        ];

        let actual = QOIHeader::parse(bytes.as_ref());

        assert!(actual.is_err());
    }
}
