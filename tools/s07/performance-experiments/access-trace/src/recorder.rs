//! Diagnostic recorder, copied only into an isolated accepted-source build.
//! Recording is source-level observation, never a production timing mode.
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    io::{self, Write},
};

const BLOCK_LIMIT: usize = 1024 * 1024;
const BLOB_CHUNK: usize = 64 * 1024;
const RECORD_HEADER: usize = 52;

thread_local! {
    static ACTIVE: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

struct Recorder {
    output: Box<dyn Write>,
    payload: Vec<u8>,
    block_records: u32,
    blocks: u64,
    records: u64,
    payload_bytes: u64,
    payload_limit: u64,
    chain: Sha256,
    file: u32,
    domain: u32,
}

impl Recorder {
    fn new(mut output: Box<dyn Write>, payload_limit: u64) -> io::Result<Self> {
        output.write_all(b"S07TRC01")?;
        Ok(Self {
            output,
            payload: Vec::with_capacity(BLOCK_LIMIT),
            block_records: 0,
            blocks: 0,
            records: 0,
            payload_bytes: 0,
            payload_limit,
            chain: Sha256::new(),
            file: 0,
            domain: 0,
        })
    }

    fn block(&mut self) -> io::Result<()> {
        if self.payload.is_empty() {
            return Ok(());
        }
        let hash = Sha256::digest(&self.payload);
        self.output.write_all(b"BLK1")?;
        self.output.write_all(&self.blocks.to_le_bytes())?;
        self.output.write_all(&self.block_records.to_le_bytes())?;
        self.output.write_all(
            &u32::try_from(self.payload.len())
                .expect("bounded block")
                .to_le_bytes(),
        )?;
        self.output.write_all(&hash)?;
        self.output.write_all(&self.payload)?;
        self.chain.update(hash);
        self.blocks += 1;
        self.block_records = 0;
        self.payload.clear();
        Ok(())
    }

    fn record(&mut self, op: u16, site: u16, values: [u64; 4], bytes: &[u8]) -> io::Result<()> {
        let length = RECORD_HEADER
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("trace record overflow"))?;
        if length > BLOCK_LIMIT {
            return Err(io::Error::other("trace record exceeds block limit"));
        }
        let next = self
            .payload_bytes
            .checked_add(length as u64)
            .ok_or_else(|| io::Error::other("trace size overflow"))?;
        if next > self.payload_limit {
            return Err(io::Error::other(
                "declared trace payload limit exceeded; capture incomplete",
            ));
        }
        if self.payload.len() + length > BLOCK_LIMIT {
            self.block()?;
        }
        self.payload.extend_from_slice(
            &u32::try_from(length - 4)
                .expect("bounded record")
                .to_le_bytes(),
        );
        self.payload.extend_from_slice(&op.to_le_bytes());
        self.payload.extend_from_slice(&site.to_le_bytes());
        self.payload.extend_from_slice(&self.file.to_le_bytes());
        self.payload.extend_from_slice(&self.domain.to_le_bytes());
        self.payload.extend_from_slice(&0u32.to_le_bytes());
        for value in values {
            self.payload.extend_from_slice(&value.to_le_bytes());
        }
        self.payload.extend_from_slice(bytes);
        self.payload_bytes = next;
        self.records += 1;
        self.block_records += 1;
        Ok(())
    }

    fn finish(mut self) -> io::Result<()> {
        self.block()?;
        self.output.write_all(b"END1")?;
        self.output.write_all(&self.blocks.to_le_bytes())?;
        self.output.write_all(&self.records.to_le_bytes())?;
        self.output.write_all(&self.payload_bytes.to_le_bytes())?;
        self.output.write_all(&self.chain.finalize())?;
        self.output.flush()
    }
}

fn with_active<T>(f: impl FnOnce(&mut Option<Recorder>) -> T) -> T {
    ACTIVE.with(|slot| {
        let mut value = slot
            .try_borrow_mut()
            .expect("access trace observer reentered");
        f(&mut value)
    })
}

pub fn start(output: Box<dyn Write>, payload_limit: u64) -> io::Result<()> {
    with_active(|slot| {
        assert!(slot.is_none(), "access trace already active");
        *slot = Some(Recorder::new(output, payload_limit)?);
        Ok(())
    })
}

pub fn context(file: u32, domain: u32) {
    assert!(domain <= 2, "unknown access trace domain");
    with_active(|slot| {
        let recorder = slot.as_mut().expect("access trace not active");
        recorder.file = file;
        recorder.domain = domain;
    });
}

/// Observer state reads cannot become apparent binder activity. Hooks have
/// operation IDs >=100; their absence outside domain2 is deliberate separation.
pub fn event(op: u16, site: u16, a: u64, b: u64, c: u64, d: u64) {
    with_active(|slot| {
        if let Some(recorder) = slot {
            if op >= 100 && recorder.domain != 2 {
                return;
            }
            recorder
                .record(op, site, [a, b, c, d], &[])
                .expect("access trace write failed; capture incomplete");
        }
    });
}

/// Chunks preserve raw bytes and total length, including empty strings. a/b are
/// the caller's identities; c is the byte offset and d the logical blob length.
pub fn blob(op: u16, site: u16, a: u64, b: u64, bytes: &[u8]) {
    with_active(|slot| {
        if let Some(recorder) = slot {
            assert_eq!(recorder.domain, 1, "state blob outside observer domain");
            let chunks = bytes.len().max(1).div_ceil(BLOB_CHUNK);
            for part in 0..chunks {
                let start = part * BLOB_CHUNK;
                let end = bytes.len().min(start + BLOB_CHUNK);
                recorder
                    .record(
                        op,
                        site,
                        [a, b, start as u64, bytes.len() as u64],
                        &bytes[start..end],
                    )
                    .expect("access trace blob failed; capture incomplete");
            }
        }
    });
}

pub fn finish() -> io::Result<()> {
    with_active(|slot| slot.take().expect("access trace not active").finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[derive(Clone)]
    struct Sink(Arc<Mutex<Vec<u8>>>);
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn framing_hash_and_completion() {
        let sink = Sink(Arc::default());
        let mut recorder = Recorder::new(Box::new(sink.clone()), 104).unwrap();
        recorder.record(1, 0, [0, 1, 2, 3], &[]).unwrap();
        recorder.record(2, 0, [4, 5, 6, 7], &[]).unwrap();
        recorder.finish().unwrap();
        let bytes = sink.0.lock().unwrap();
        assert_eq!(&bytes[..8], b"S07TRC01");
        assert_eq!(&bytes[8..12], b"BLK1");
        assert_eq!(u32::from_le_bytes(bytes[20..24].try_into().unwrap()), 2);
        assert_eq!(&bytes[28..60], Sha256::digest(&bytes[60..164]).as_slice());
        assert_eq!(&bytes[164..168], b"END1");
        assert_eq!(bytes.len(), 224);
    }
    #[test]
    fn overflow_cannot_produce_a_successful_capture() {
        let sink = Sink(Arc::default());
        let mut recorder = Recorder::new(Box::new(sink.clone()), 51).unwrap();
        assert!(recorder
            .record(1, 0, [0; 4], &[])
            .unwrap_err()
            .to_string()
            .contains("limit exceeded"));
        drop(recorder);
        assert_eq!(&*sink.0.lock().unwrap(), b"S07TRC01");
    }
    #[test]
    fn state_observation_suppresses_binder_hooks() {
        let sink = Sink(Arc::default());
        start(Box::new(sink.clone()), 1024).unwrap();
        context(1, 1);
        event(100, 1, 10, 0, 0, 0);
        blob(10, 0, 1, 0, b"");
        context(1, 2);
        event(100, 1, 10, 0, 0, 0);
        finish().unwrap();
        let bytes = sink.0.lock().unwrap();
        assert_eq!(u32::from_le_bytes(bytes[20..24].try_into().unwrap()), 2);
    }
}
