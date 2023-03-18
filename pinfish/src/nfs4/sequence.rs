use std::sync::Mutex;
use tokio::sync::{Semaphore, SemaphorePermit};

/// Sequence and slot number for NFS4 SEQUENCE operation
#[derive(Debug)]
pub struct SequenceInfo {
    pub slot: u32,
    pub sequence: u32,
}

/// Holds a slot and sequence number and releases them
/// when dropped.
pub struct ClientSequence<'a> {
    pub info: SequenceInfo,
    owner: &'a ClientSequencer,
    _permit: SemaphorePermit<'a>,
}

impl<'a> core::ops::Deref for ClientSequence<'a> {
    type Target = SequenceInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl<'a> core::ops::Drop for ClientSequence<'a> {
    fn drop(&mut self) {
        self.owner.free_slot(self.info.slot)
    }
}

struct ClientSequencerInner {
    pub busy: Vec<u64>,
    pub sequences: Vec<u32>,
}

impl ClientSequencerInner {
    pub fn allocate_slot(&mut self) -> usize {
        for i in 0..self.busy.len() {
            if self.busy[i] != u64::MAX {
                let first = self.busy[i].trailing_ones() as usize;
                let bit = 1 << first;
                let result = i * 64 + first;

                assert!(result < self.sequences.len());

                self.busy[i] |= bit;

                return result;
            }
        }

        // Should be unreachable as semaphore ensures a slot is available
        panic!("no free slots");
    }

    pub fn free_slot(&mut self, slot: usize) {
        let index = slot / 64;
        let shift = slot % 64;
        let bit = (1 << shift) as u64;

        assert_eq!(self.busy[index] & bit, bit);

        self.busy[index] &= !bit;
    }

    pub fn get_max(&self) -> usize {
        self.sequences.len() - 1
    }
    /*
        pub fn highest_used(&self) -> usize {
            for i in (0..self.busy.len()).rev() {
                if self.busy[i] != 0 {
                    let first = self.busy[i].leading_zeros() as usize;
                    let result = i * 64 + first;

                    assert!(result < self.sequences.len());

                    return result;
                }
            }

            // Should be unreachable as semaphore ensures a slot is available
            panic!("no free slots");
        }
    */
}

/// Manages slots and sequence numbers for NFS client
pub struct ClientSequencer {
    /// Sempahor used for waiting for a free slot
    sem: Semaphore,

    inner: Mutex<ClientSequencerInner>,
}

impl ClientSequencer {
    /// Creates a new sequencer with `size` slots
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        ClientSequencer {
            sem: Semaphore::new(size),
            inner: Mutex::new(ClientSequencerInner {
                busy: vec![0; (size + 63) / 64],
                sequences: vec![0; size],
            }),
        }
    }

    fn free_slot(&self, slot: u32) {
        let index = slot as usize;
        let mut inner = self.inner.lock().unwrap();
        inner.free_slot(index);
    }

    /// Asynchronously allocates a `ClientSequence` for NFS SEQUENCE
    /// op used as part of COMPOUND call.
    pub async fn get_seq(&self) -> ClientSequence<'_> {
        // Acquire a permit, this must succeed because the semaphore is not
        // closed until dropped
        let permit = self
            .sem
            .acquire()
            .await
            .expect("unexpected semaphore error");

        let (slot, sequence) = {
            let mut inner = self.inner.lock().unwrap();
            let index = inner.allocate_slot();
            inner.sequences[index] += 1;
            (index as u32, inner.sequences[index])
        };

        ClientSequence {
            info: SequenceInfo { slot, sequence },
            owner: &self,
            _permit: permit,
        }
    }

    pub fn get_max(&self) -> u32 {
        self.inner.lock().unwrap().get_max() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_test() {
        let sequencer = ClientSequencer::new(100);
        let seq0 = sequencer.get_seq().await;
        assert_eq!(seq0.slot, 0);
        assert_eq!(seq0.sequence, 1);

        let seq1 = sequencer.get_seq().await;
        assert_eq!(seq1.slot, 1);
        assert_eq!(seq1.sequence, 1);

        drop(seq0);

        let seq2 = sequencer.get_seq().await;
        assert_eq!(seq2.slot, 0);
        assert_eq!(seq2.sequence, 2);

        let seq3 = sequencer.get_seq().await;
        assert_eq!(seq3.slot, 2);
        assert_eq!(seq3.sequence, 1);
    }
}
