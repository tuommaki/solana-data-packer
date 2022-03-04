use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ProgramInstruction {
    /// Create a bucket for data.
    ///
    /// # Account references
    ///   0. `[SIGNER, WRITE]` Account used to derive and control the new data bucket.
    ///   1. `[SIGNER, WRITE]` Account that will fund the new data bucket.
    ///   2. `[WRITE]` Uninitialized data bucket account
    ///   3. `[]` System program for CPI.
    CreateBucket {
        /// Seed data (the first bytes) to put into the bucket.
        data: Vec<u8>,

        /// Total size of data bucket.
        size: usize,

        /// Data buckets are always initialized at program-derived
        /// addresses using the funding address, recent blockhash, and
        /// the user-passed `bump_seed`.
        bump_seed: u8,
    },

    /// Put data into bucket.
    ///
    /// # Account references
    ///   0. `[SIGNER, WRITE]` Account used to control the new data bucket.
    ///   1. `[SIGNER, WRITE]` Account that will fund the extension of data bucket.
    ///   2. `[WRITE]` Data bucket account.
    ///   3. `[]` System program for CPI.
    PutIntoBucket {
        /// Data to put into the bucket.
        data: Vec<u8>,

        /// Offset
        offset: usize,
    },
}
