#[derive(Debug)]
enum GzipError {
    LogicError(String),
    FormatError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for GzipError {
    fn from(err: std::io::Error) -> GzipError {
        GzipError::IoError(err)
    }
}

type Result<T> = std::result::Result<T, GzipError>;

#[derive(Default, Debug)]
struct GzipHeader {
    cm: u8,
    ftext: bool,
    fhcrc: bool,
    fextra: bool,
    fname: bool,
    fcomment: bool,
    mtime: u32,
    xfl: u8,
    os: u8,
    xlen: u16,
    xextra: Vec<u8>,
    xname: Vec<u8>,
    xcomment: Vec<u8>,
    xhcrc: u16,
}

#[derive(PartialEq, Eq)]
enum GzipStreamState {
    New,
    HeaderRead,
}

struct GzipStream<R: std::io::BufRead> {
    state: GzipStreamState,
    pub header: GzipHeader,
    f: R,
}

fn read_u16_big_endian(buf: &[u8]) -> u16 {
    ((buf[1] as u16) << 8) | buf[0] as u16
}

fn read_u32_big_endian(buf: &[u8]) -> u32 {
    let mut n: u32 = 0;
    for i in 0..4 {
        n <<= 8;
        n |= buf[3-i] as u32;
    }
    n
}

impl<R: std::io::BufRead> GzipStream<R> {
    fn new(f: R) -> GzipStream<R> {
        GzipStream {
            state: GzipStreamState::New,
            header: GzipHeader::default(),
            f: f
        }
    }

    fn read_header(&mut self) -> Result<()> {
        if self.state != GzipStreamState::New {
            return Err(GzipError::LogicError(String::from("Incorrect state")))
        }

        let mut buf = [0; 12];
        self.f.read_exact(&mut buf)?;
        println!("buf {:?}", buf);

        if buf[0] != 0x1f || buf[1] != 0x8b {
            return Err(GzipError::FormatError(
                format!("Unexpected ID1, ID2: got {:x} {:x}", buf[0], buf[1])))
        }

        self.header.cm = buf[2];
        let mut flags = buf[3];
        self.header.ftext = (flags & 1) == 1;
        flags >>= 1;
        self.header.fhcrc = (flags & 1) == 1;
        flags >>= 1;
        self.header.fextra = (flags & 1) == 1;
        flags >>= 1;
        self.header.fname = (flags & 1) == 1;
        flags >>= 1;
        self.header.fcomment = (flags & 1) == 1;

        self.header.mtime = read_u32_big_endian(&buf[4..8]);
        self.header.xfl = buf[8];
        self.header.os = buf[9];
        if self.header.fextra {
            self.header.xlen = read_u16_big_endian(&buf[10..12]);
            self.header.xextra = vec![0; self.header.xlen as usize];
            self.f.read_exact(&mut self.header.xextra)?;
        }
        if self.header.fname {
            self.f.read_until(0u8, &mut self.header.xname)?;
        }
        if self.header.fcomment {
            self.f.read_until(0u8, &mut self.header.xcomment)?;
        }
        if self.header.fhcrc {

        }
        let mut hcrc_buf = [0;2];
        self.f.read_exact(&mut hcrc_buf)?;
        self.header.xhcrc = read_u16_big_endian(&hcrc_buf);

        self.state = GzipStreamState::HeaderRead;
        Ok(())
    }
}

fn main() {
    let stdin  = std::io::stdin();
    let mut g = GzipStream::new(stdin.lock());
    g.read_header().unwrap();
    println!("Header: {:?}", g.header);
}
