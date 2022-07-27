pub mod parser;
pub mod parser_tests;

#[derive(Default, Debug, PartialEq)]
pub struct Partition {
    // For sorting
    pub(crate) designation: usize,
    pub(crate) start_block: u64,

    // For reconstructing sfdisk dump output
    pub(crate) name: String,
    pub(crate)extras: Vec<String>,
}