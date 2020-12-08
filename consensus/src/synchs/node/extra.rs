use types::Certificate;

// We will serialize extra in the extra field for Sync HotStuff, instead of
// crafting another block structure
pub struct Extra {
    pub certificate: Certificate,
}