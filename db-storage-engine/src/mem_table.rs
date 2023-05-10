/* 
  MemTable Struct: 
    - MemTable holds a sorted list of the latest written records
    - Writes are duplicated to the WAL for recovery of the MemTable in the event of a restart
    - MemTables have a max capacity and when that is reached, we flush the MemTable to disk as a Table(SSTable)
    - Entries are stored in a Vector instead of a HashMap to support Scans
    - Inside the Vector are MemTableEntrys (see struct below)
*/
pub struct MemTable {
  entries: Vec<MemTableEntry>,
  size: usize,
}
 
/* 
  MemTableEntry Struct: 
    - MemTableEntry holds information about the modified record
    - Key and Value use Vectors to store data
    - Timestamp is the time this write occurred in microseconds and is used to order writes to the same key when cleaning our old data in SSTables
    - Deleted indicates if entry is a Tombstone (which supports fast deletes)
    - Value is optional since Tombstones won't record a value
*/
pub struct MemTableEntry {
  pub key: Vec<u8>,
  pub value: Option<Vec<u8>>,
  pub timestamp: u128,
  pub deleted: bool,
}

/* 
  new() Function: 
    - Simply creates a new empty MemTable
*/
pub fn new() -> MemTable {
  MemTable {
    entries: Vec::new(),
    size: 0,
  }
}

/* 
  get_index() Function: 
    - Performs Binary Search to find a record in the MemTable
    - Binary Search implementation included in the Vec standard library
    - If the record is found `[Result::Ok]` is returned, with the index of record
    - If the record is not found then `[Result::Err]` is returned, with the index to insert the record at
*/
fn get_index(&self, key: &[u8]) -> Result<usize, usize> {
  self
    .entries
    .binary_search_by_key(&key, |e| e.key.as_slice())
}