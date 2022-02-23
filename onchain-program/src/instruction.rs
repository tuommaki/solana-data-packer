use {
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ProgramInstruction {
    /// Create a bucket for data.
    ///
    /// # Account references
    ///   0. `[WRITE]` Uninitialized data bucket account
    ///   1. `[SIGNER]` Account used to derive and control the new data bucket.
    ///   2. `[SIGNER, WRITE]` Account that will fund the new data bucket.
    ///   3. `[]` System program for CPI.
    CreateBucket {
        /// Data to put into the bucket.
        data: Vec<u8>,

        /// Data buckets are always initialized at program-derived
        /// addresses using the funding address, recent blockhash, and
        /// the user-passed `bump_seed`.
        bump_seed: u8,
    },

    /// Append data into bucket.
    AppendIntoBucket {
        /// Data to append into the bucket.
        data: Vec<u8>,
    },
}
