//! `lmtht` crate represents Logarithmic Multi-Tier Hash Tree -- an implementation of a list structure
//! with Hash Tree (Merkle Tree) that stores a complete history of additive changes in that tree
//! structure, with efficient append characteristics for practical storage device. This allows
//! data to be appended and, like a typical hash tree, can be used to verify data corruption or
//! tampering with very small amounts of data.
//!
//! See also [my personal research page for more detail](https://hazm.at/mox/algorithm/structural-algorithm/logarithmic-multi-tier-hash-tree/index.html).
//!
//! # Examples
//!
//! ```rust
//! use lmtht::{MemStorage, LMTHT, Value, Node};
//! let mut db = LMTHT::new(MemStorage::new()).unwrap();
//!
//! // Returns None for non-existent indices.
//! let mut query = db.query().unwrap();
//! assert_eq!(None, query.get(1).unwrap());
//!
//! // The first value is considered to index 1, and they are simply incremented thereafter.
//! let first = "first".as_bytes();
//! let root = db.append(first).unwrap();
//! let mut query = db.query().unwrap();
//! assert_eq!(1, root.i);
//! assert_eq!(first, query.get(root.i).unwrap().unwrap());
//!
//! // Similar to the typical hash tree, you can refer to a verifiable value using root hash.
//! let second = "second".as_bytes();
//! let third = "third".as_bytes();
//! db.append(second).unwrap();
//! let root = db.append(third).unwrap();
//! let mut query = db.query().unwrap();
//! let values = query.get_values_with_hashes(2, 0).unwrap().unwrap();
//! assert_eq!(1, values.values.len());
//! assert_eq!(Value::new(2, second.to_vec()), values.values[0]);
//! assert_eq!(Node::new(3, 2, root.hash), values.root());
//!
//! // By specifying `j` greater than 0, you can refer to contiguous values that belongs to
//! // the binary subtree. The following refers to the values belonging to intermediate nodes bââ.
//! let values = query.get_values_with_hashes(2, 1).unwrap().unwrap();
//! assert_eq!(2, values.values.len());
//! assert_eq!(Value::new(1, first.to_vec()), values.values[0]);
//! assert_eq!(Value::new(2, second.to_vec()), values.values[1]);
//! assert_eq!(Node::new(3, 2, root.hash), values.root());
//! ```
//!
use std::cmp::min;
use std::fmt::{Debug, Display, Formatter};
use std::fs::*;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, LockResult, RwLock};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use highway::{HighwayBuilder, Key};

use crate::checksum::{HashRead, HashWrite};
use crate::error::Detail;
use crate::error::Detail::*;
use crate::model::{range, NthGenHashTree};

pub(crate) mod checksum;
pub mod error;
pub mod inspect;
pub mod model;

#[cfg(test)]
pub mod test;

/// lmtht ã¯ã¬ã¼ãã§ä½¿ç¨ããæ¨æº Resultã[`error::Detail`] ãåç§ã
pub type Result<T> = std::result::Result<T, error::Detail>;

/// ããã·ã¥æ¨ãä¿å­ããæ½è±¡åãããã¹ãã¬ã¼ã¸ã§ããread ç¨ã¾ãã¯ read + write ç¨ã®ã«ã¼ã½ã«åç§ãå®è£ãããã¨ã§
/// ä»»æã®ããã¤ã¹ã«ç´ååãããã¨ãã§ãã¾ãã
pub trait Storage {
  /// ãã®ã¹ãã¬ã¼ã¸ã«å¯¾ãã read ã¾ãã¯ read + write ç¨ã®ã«ã¼ã½ã«ãä½æãã¾ãã
  fn open(&self, writable: bool) -> Result<Box<dyn Cursor>>;
}

/// ã­ã¼ã«ã«ãã¡ã¤ã«ã·ã¹ãã ã®ãã¹ãã¹ãã¬ã¼ã¸ã¨ãã¦ä½¿ç¨ããå®è£ã§ãã
impl<P: AsRef<Path>> Storage for P {
  fn open(&self, writable: bool) -> Result<Box<dyn Cursor>> {
    let file = OpenOptions::new().read(true).write(writable).create(writable).open(self);
    match file {
      Ok(file) => Ok(Box::new(file)),
      Err(err) => Err(Detail::FailedToOpenLocalFile {
        file: self.as_ref().to_str().map(|s| s.to_string()).unwrap_or(self.as_ref().to_string_lossy().to_string()),
        message: err.to_string(),
      }),
    }
  }
}

/// ã¡ã¢ãªä¸ã®é åãã¹ãã¬ã¼ã¸ã¨ãã¦ä½¿ç¨ããå®è£ã§ãã`drop()` ãããæç¹ã§è¨é²ãã¦ããåå®¹ãæ¶æ»ãããããã¹ãã
/// èª¿æ»ã§ã®ä½¿ç¨ãæ³å®ãã¦ãã¾ãã
pub struct MemStorage {
  buffer: Arc<RwLock<Vec<u8>>>,
}

impl MemStorage {
  /// æ®çºæ§ã¡ã¢ãªãä½¿ç¨ããã¹ãã¬ã¼ã¸ãæ§ç¯ãã¾ãã
  pub fn new() -> MemStorage {
    Self::with(Arc::new(RwLock::new(Vec::<u8>::with_capacity(4 * 1024))))
  }

  /// æå®ãããã¢ãããã¯åç§ã«ã¦ã³ã/RWã­ãã¯ä»ãã®å¯å¤ãããã¡ãä½¿ç¨ããã¹ãã¬ã¼ã¸ãæ§ç¯ãã¾ããããã¯èª¿æ»ã®ç®çã§
  /// å¤é¨ããã¹ãã¬ã¼ã¸ã®åå®¹ãåç§ãããã¨ãæ³å®ãã¦ãã¾ãã
  pub fn with(buffer: Arc<RwLock<Vec<u8>>>) -> MemStorage {
    MemStorage { buffer }
  }
}

impl Storage for MemStorage {
  fn open(&self, writable: bool) -> Result<Box<dyn Cursor>> {
    Ok(Box::new(MemCursor { writable, position: 0, buffer: self.buffer.clone() }))
  }
}

struct MemCursor {
  writable: bool,
  position: usize,
  buffer: Arc<RwLock<Vec<u8>>>,
}

impl Cursor for MemCursor {}

impl io::Seek for MemCursor {
  fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
    self.position = match pos {
      io::SeekFrom::Start(position) => position as usize,
      io::SeekFrom::End(position) => {
        let mut buffer = lock2io(self.buffer.write())?;
        let new_position = (buffer.len() as i64 + position) as usize;
        while buffer.len() < new_position {
          buffer.push(0u8);
        }
        new_position
      }
      io::SeekFrom::Current(position) => (self.position as i64 + position) as usize,
    };
    Ok(self.position as u64)
  }
}

impl io::Read for MemCursor {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let buffer = lock2io(self.buffer.read())?;
    let length = min(buf.len(), buffer.len() - self.position);
    (&mut buf[..]).write_all(&buffer[self.position..self.position + length])?;
    self.position += length;
    Ok(length)
  }
}

impl io::Write for MemCursor {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    if !self.writable {
      return Err(io::Error::from(io::ErrorKind::PermissionDenied));
    }
    let mut buffer = lock2io(self.buffer.write())?;
    let length = buffer.write(buf)?;
    self.position += length;
    Ok(length)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

/// `LockResult` ã `io::Result` ã«å¤æãã¾ãã
#[inline]
fn lock2io<T>(result: LockResult<T>) -> io::Result<T> {
  result.map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
}

/// ã¹ãã¬ã¼ã¸ãããã¼ã¿ã®å¥åºåãè¡ãããã®ã«ã¼ã½ã«ã§ãã
pub trait Cursor: io::Seek + io::Read + io::Write {}

impl Cursor for File {}

/// LMTHT ãã¤ã³ããã¯ã¹ i ã¨ãã¦ä½¿ç¨ããæ´æ°ã®åã§ãã`u64` ãè¡¨ãã¦ãã¾ãã
///
/// 64-bit ãã¢ããªã±ã¼ã·ã§ã³ã¸ã®é©ç¨ã«å¤§ããããå ´å `small_index` feature ãæå®ãããã¨ã§ `u32` ã«å¤æ´ãã
/// ãã¨ãã§ãã¾ãã
///
pub type Index = model::Index;

/// [`Index`] åã®ãããå¹ãè¡¨ãå®æ°ã§ãã64 ãè¡¨ãã¦ãã¾ãã
///
/// ã³ã³ãã¤ã«æã« `small_index` feature ãæå®ãããã¨ã§ãã®å®æ°ã¯ 32 ã¨ãªãã¾ãã
///
pub const INDEX_SIZE: u8 = model::INDEX_SIZE;

/// ããã·ã¥æ¨ãæ§æãããã¼ããè¡¨ãã¾ãã
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Node {
  /// ãã®ãã¼ãã®ã¤ã³ããã¯ã¹ã
  pub i: Index,
  /// ãã®ãã¼ãã®é«ãã
  pub j: u8,
  /// ãã®ãã¼ãã®ããã·ã¥å¤ããã®å¤ã¯ [`Hash::hash()`] ã«ãã£ã¦ç®åºããã¦ãã¾ãã
  pub hash: Hash,
}

impl Node {
  pub fn new(i: Index, j: u8, hash: Hash) -> Node {
    Node { i, j, hash }
  }
  fn for_node(node: &MetaInfo) -> Node {
    Self::new(node.address.i, node.address.j, node.hash.clone())
  }

  /// ãã®ãã¼ããå·¦æã`right` ãã¼ããå³æã¨ããè¦ªãã¼ããç®åºãã¾ãã
  pub fn parent(&self, right: &Node) -> Node {
    debug_assert!(self.i < right.i);
    debug_assert!(self.j >= right.j);
    let i = right.i;
    let j = self.j + 1;
    let hash = self.hash.combine(&right.hash);
    Node::new(i, j, hash)
  }
}

impl Display for Node {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("{},{}:{}", self.i, self.j, hex(&self.hash.value)))
  }
}

/// ããã·ã¥æ¨ã«ä¿å­ããã¦ããå¤ãåç§ãã¾ãã
#[derive(PartialEq, Eq, Debug)]
pub struct Value {
  /// ãã®å¤ã®ã¤ã³ããã¯ã¹ã
  pub i: Index,
  /// ãã®å¤ã®ãã¤ããªå¤ã
  pub value: Vec<u8>,
}

impl Value {
  pub fn new(i: Index, value: Vec<u8>) -> Value {
    Value { i, value }
  }
  /// ãã®å¤ã®ããã·ã¥å¤ãç®åºãã¾ãã
  pub fn hash(&self) -> Hash {
    Hash::hash(&self.value)
  }
  pub fn to_node(&self) -> Node {
    Node::new(self.i, 0u8, self.hash())
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("{}:{}", self.i, hex(&self.value)))
  }
}

/// ããã·ã¥æ¨ããåå¾ãããçµè·¯ã®åå²åã®ããã·ã¥å¤ãå«ãå¤ã®ã»ããã§ããå¤ã®ããã·ã¥å¤ã¨åå²ãã¼ãã®ããã·ã¥å¤ãã
/// ã«ã¼ãããã·ã¥ãç®åºããã¯ã©ã¤ã¢ã³ããæã¤ã«ã¼ãããã·ã¥ã¨æ¯è¼ãããã¨ã§ãåå¾ããå¤ãæ¹å¤ããã¦ããªããã¨ãæ¤è¨¼
/// ãããã¨ãã§ãã¾ãã
#[derive(Debug)]
pub struct ValuesWithBranches {
  pub values: Vec<Value>,
  pub branches: Vec<Node>,
}

impl ValuesWithBranches {
  pub fn new(values: Vec<Value>, branches: Vec<Node>) -> ValuesWithBranches {
    // values ã¯é£ç¶ãã¦ããªããã°ãªããªã
    #[cfg(debug_assertions)]
    for i in 0..values.len() - 1 {
      debug_assert_eq!(values[i].i + 1, values[i + 1].i);
    }
    ValuesWithBranches { values, branches }
  }

  /// ãã®çµæããå¾ãããã«ã¼ããã¼ããã«ã¼ãããã·ã¥ä»ãã§ç®åºãã¾ãã
  pub fn root(&self) -> Node {
    // ãã¹ã¦ã®å¤ãããã·ã¥å¤ã«å¤æãã
    let mut hashes = self.values.iter().map(|value| value.to_node()).collect::<Vec<Node>>();

    // å¤ããç®åºããããã·ã¥å¤ãæãããã
    while hashes.len() > 1 {
      // hashes ã®è¦ç´ ã 2 ã¤ä¸çµã§æãããã (è¦ç´ æ°ãå¥æ°ã®å ´åã¯æãå³ããã¼ããä¸éæ§ã®ä¸­éãã¼ã)
      for k in 0..hashes.len() / 2 {
        let left = &hashes[k * 2];
        let right = &hashes[k * 2 + 1];
        hashes[k] = left.parent(&right);
      }
      // æãããã¾ãã¦ããªãä¸éæ§ã®ä¸­éãã¼ãã¯æ¬¡ã«æã¡è¶ã
      let fraction = if hashes.len() % 2 != 0 {
        let len = hashes.len();
        hashes[len / 2] = hashes.pop().unwrap();
        1
      } else {
        0
      };
      hashes.truncate(hashes.len() / 2 + fraction);
    }

    // çµè·¯ããåå²ãããã¼ãã®ããã·ã¥å¤ã¨çµ±åãã«ã¼ããã¼ããç®åºãã
    let mut folding = hashes.remove(0);
    for k in 0..self.branches.len() {
      let branch = &self.branches[self.branches.len() - k - 1];
      let (left, right) = if folding.i < branch.i { (&folding, branch) } else { (branch, &folding) };
      folding = left.parent(&right);
    }
    folding
  }
}

// --------------------------------------------------------------------------

/// [`Hash::hash()`] ã«ãã£ã¦å¾ãããããã·ã¥å¤ã®ãã¤ããµã¤ãºãè¡¨ãå®æ°ã§ããããã©ã«ãã® `feature = "sha256"`
/// ãã«ãã§ã¯ 32 ãè¡¨ãã¾ãã
pub const HASH_SIZE: usize = {
  #[cfg(feature = "highwayhash64")]
  {
    8
  }
  #[cfg(any(feature = "sha224", feature = "sha512_224"))]
  {
    28
  }
  #[cfg(any(feature = "sha256", feature = "sha512_256"))]
  {
    32
  }
  #[cfg(feature = "sha512")]
  {
    64
  }
};

/// ããã·ã¥æ¨ãä½¿ç¨ããããã·ã¥å¤ã§ãã
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Hash {
  pub value: [u8; HASH_SIZE],
}

impl Hash {
  pub fn new(hash: [u8; HASH_SIZE]) -> Hash {
    Hash { value: hash }
  }

  /// æå®ãããå¤ãããã·ã¥åãã¾ãã
  pub fn hash(value: &[u8]) -> Hash {
    #[cfg(feature = "highwayhash64")]
    {
      use highway::HighwayHash;
      let mut builder = HighwayBuilder::default();
      builder.write_all(value).unwrap();
      Hash::new(builder.finalize64().to_le_bytes())
    }
    #[cfg(not(feature = "highwayhash64"))]
    {
      use sha2::Digest;
      #[cfg(feature = "sha224")]
      use sha2::Sha224 as Sha2;
      #[cfg(any(feature = "sha256"))]
      use sha2::Sha256 as Sha2;
      #[cfg(feature = "sha512")]
      use sha2::Sha512 as Sha2;
      #[cfg(feature = "sha512/224")]
      use sha2::Sha512Trunc224 as Sha2;
      #[cfg(feature = "sha512/256")]
      use sha2::Sha512Trunc256 as Sha2;
      let output = Sha2::digest(value);
      debug_assert_eq!(HASH_SIZE, output.len());
      let mut hash = [0u8; HASH_SIZE];
      (&mut hash[..]).write_all(&output).unwrap();
      Hash::new(hash)
    }
  }

  /// æå®ãããããã·ã¥å¤ã¨é£çµããããã·ã¥å¤ `hash(self.hash || other.hash)` ãç®åºãã¾ãã
  pub fn combine(&self, other: &Hash) -> Hash {
    let mut value = [0u8; HASH_SIZE * 2];
    value[..HASH_SIZE].copy_from_slice(&self.value);
    value[HASH_SIZE..].copy_from_slice(&other.value);
    Hash::hash(&value)
  }

  pub fn to_str(&self) -> String {
    hex(&self.value)
  }
}

/// ãã¼ã b_{i,j} ãå«ãã¨ã³ããªãã¹ãã¬ã¼ã¸ä¸ã®ã©ãã«ä½ç½®ããããè¡¨ãã¾ãã
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct Address {
  /// ããã·ã¥æ¨ã®ãªã¹ãæ§é ä¸ã§ã®ä½ç½®ã1 ããéå§ã [`Index`] ã®æå¤§å¤ã¾ã§ã®å¤ãåãã¾ãã
  pub i: Index,
  /// ãã®ãã¼ãã®é«ã (æãé ãèãã¼ãã¾ã§ã®è·é¢)ã0 ã®å ´åããã¼ããèãã¼ãã§ãããã¨ãç¤ºãã¦ãã¾ããæå¤§å¤ã¯
  /// [`INDEX_SIZE`] ã§ãã
  pub j: u8,
  /// ãã®ãã¼ããæ ¼ç´ããã¦ããã¨ã³ããªã®ã¹ãã¬ã¼ã¸åé ­ããã®ä½ç½®ã§ãã
  pub position: u64,
}

impl Address {
  pub fn new(i: Index, j: u8, position: u64) -> Address {
    Address { i, j, position }
  }
}

/// ããã·ã¥å¤ãå«ãããã¼ã b_{i,j} ã®å±æ§æå ±ãè¡¨ãã¾ãã
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct MetaInfo {
  pub address: Address,
  pub hash: Hash,
}

impl MetaInfo {
  pub fn new(address: Address, hash: Hash) -> MetaInfo {
    MetaInfo { address, hash }
  }
}

impl Display for MetaInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Node({},{}@{}){}", self.address.i, self.address.j, self.address.position, self.hash.to_str()))
  }
}

/// å·¦å³ã®æãæã¤ä¸­éãã¼ããè¡¨ãã¾ãã
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct INode {
  pub meta: MetaInfo,
  /// å·¦æã®ãã¼ã
  pub left: Address,
  /// å³æã®ãã¼ã
  pub right: Address,
}

impl INode {
  pub fn new(meta: MetaInfo, left: Address, right: Address) -> INode {
    INode { meta, left, right }
  }
}

/// å¤ãæã¤èãã¼ããè¡¨ãã¾ãã
#[derive(PartialEq, Eq, Debug)]
struct ENode {
  pub meta: MetaInfo,
  pub payload: Vec<u8>,
}

#[derive(Eq, PartialEq, Debug)]
enum RootRef<'a> {
  None,
  INode(&'a INode),
  ENode(&'a ENode),
}

#[derive(PartialEq, Eq, Debug)]
struct Entry {
  enode: ENode,
  inodes: Vec<INode>,
}

// --------------------------------------------------------------------------

/// HighwayHash ã§ãã§ãã¯ãµã ç¨ã®ããã·ã¥å¤ãçæããããã®ã­ã¼ (256-bit åºå®å¤)ã
const CHECKSUM_HW64_KEY: [u64; 4] = [0xFA5015F2E22BCFC6u64, 0xCE5A4ED9A4025C80, 0x16B9731717F6315E, 0x0F34D06AE93BD8E9];

/// ãã¤ã­ã¼ã (å¤) ã®æå¤§ãã¤ããµã¤ãºãè¡¨ãå®æ°ã§ãã2GB (2,147,483,647 bytes) ãè¡¨ãã¾ãã
///
/// ãã¬ã¤ã©ã¼ã® offset å¤ã u32 ã«ããããã«ã¯ã¨ã³ããªã®ç´ååè¡¨ç¾ãæå¤§ã§ã `u32::MAX` ã¨ããå¿è¦ãããã¾ãã
/// ãããã£ã¦ä»»æå¸³ã®ãã¤ã­ã¼ãã¯ 2GB ã¾ã§ã¨ãã¾ãããã®å®æ°ã¯ããããã¹ã¯ã¨ãã¦ãä½¿ç¨ãããã 1-bit ã®é£ç¶ã§
/// æ§æããã¦ããå¿è¦ãããã¾ãã
///
pub const MAX_PAYLOAD_SIZE: usize = 0x7FFFFFFF;

/// LMTHT ãã¡ã¤ã«ã®åé ­ã«è¨é²ããã 3 ãã¤ãã®è­å¥å­ãè¡¨ãå®æ°ã§ããå¤ã¯ Unicode ã§ã®deciduous tree ð² (U+1F332)
/// ã«ç±æ¥ãã¾ãã
pub const STORAGE_IDENTIFIER: [u8; 3] = [0x01u8, 0xF3, 0x33];

/// è­å¥å­ã«ç¶ãã¦éç½®ãããããã®å®è£ã«ãããã¹ãã¬ã¼ã¸ãã©ã¼ãããã®ãã¼ã¸ã§ã³ã§ããç¾å¨ã¯ 1 ãä½¿ç¨ãã¾ãã
pub const STORAGE_VERSION: u8 = 1;

/// ä½¿ç¨ãããã¨ãã¦ããã¹ãã¬ã¼ã¸ã¨äºææ§ãããããç¢ºèªãã¾ãã
fn is_version_compatible(version: u8) -> bool {
  version <= STORAGE_VERSION
}

#[derive(PartialEq, Eq, Debug)]
struct CacheInner {
  last_entry: Entry,
  model: NthGenHashTree,
}

#[derive(PartialEq, Eq, Debug)]
struct Cache(Option<CacheInner>);

impl Cache {
  fn new(last_entry: Entry, model: NthGenHashTree) -> Self {
    debug_assert_eq!(model.n(), last_entry.enode.meta.address.i);
    Cache(Some(CacheInner { last_entry, model }))
  }
  fn from_entry(last_entry: Option<Entry>) -> Self {
    let inner = if let Some(last_entry) = last_entry {
      let n = last_entry.enode.meta.address.i;
      let model = NthGenHashTree::new(n);
      Some(CacheInner { last_entry, model })
    } else {
      None
    };
    Cache(inner)
  }

  fn last_entry(&self) -> Option<&Entry> {
    if let Some(CacheInner { last_entry, .. }) = &self.0 {
      Some(last_entry)
    } else {
      None
    }
  }

  fn root(&self) -> Option<Node> {
    self
      .last_entry()
      .map(|e| e.inodes.last().map(|i| &i.meta).unwrap_or(&e.enode.meta))
      .map(|root| Node::new(root.address.i, root.address.j, root.hash))
  }

  fn root_ref<'a>(&self) -> RootRef {
    self
      .last_entry()
      .map(|e| e.inodes.last().map(|i| RootRef::INode(i)).unwrap_or(RootRef::ENode(&e.enode)))
      .unwrap_or(RootRef::None)
  }

  fn n(&self) -> Index {
    self.last_entry().map(|e| e.enode.meta.address.i).unwrap_or(0)
  }
}

/// ã¹ãã¬ã¼ã¸ä¸ã«ç´ååããã Logarithmic Multi-Tier Hash Tree ãè¡¨ãæ¨æ§é ã«å¯¾ããæä½ãå®è£ãã¾ãã
pub struct LMTHT<S: Storage> {
  storage: Box<S>,
  latest_cache: Arc<Cache>,
}

impl<S: Storage> LMTHT<S> {
  /// æå®ããã [`Storage`] ã«ç´ååãããããã·ã¥æ¨ãä¿å­ãã LMTHT ãæ§ç¯ãã¾ãã
  ///
  /// ã¹ãã¬ã¼ã¸ã« [`std::path::Path`] ã [`std::path::PathBuf`] ã®ãããªãã¹ãæå®ããããã¨ãã®ãã¡ã¤ã«ã«
  /// ç´ååãããããã·ã¥æ¨ãä¿å­ãã¾ãããã¹ããæ¤è¨¼ç®çã§ã¯ã¡ã¢ãªä¸ã«ããã·ã¥æ¨ãç´ååãã [`MemStorage`] ã
  /// ä½¿ç¨ãããã¨ãã§ãã¾ããã¹ãã¬ã¼ã¸ã¯æ½è±¡åããã¦ããããç¬èªã® [`Storage`] å®è£ãä½¿ç¨ãããã¨ãã§ãã¾ãã
  ///
  /// # Examples
  ///
  /// ä»¥ä¸ã¯ã·ã¹ãã ã®ãã³ãã©ãªãã£ã¬ã¯ããªä¸ã® `mbht-example.db` ã«ããã·ã¥æ¨ãç´ååããä¾ã§ãã
  ///
  /// ```rust
  /// use lmtht::{LMTHT,Storage,Result};
  /// use std::env::temp_dir;
  /// use std::fs::remove_file;
  /// use std::path::PathBuf;
  ///
  /// fn append_and_get(file: &PathBuf) -> Result<()>{
  ///   let mut db = LMTHT::new(file)?;
  ///   let root = db.append(&vec![0u8, 1, 2, 3])?;
  ///   assert_eq!(Some(vec![0u8, 1, 2, 3]), db.query()?.get(root.i)?);
  ///   Ok(())
  /// }
  ///
  /// let mut path = temp_dir();
  /// path.push("lmtht-example.db");
  /// append_and_get(&path).expect("test failed");
  /// remove_file(path.as_path()).unwrap();
  /// ```
  pub fn new(storage: S) -> Result<LMTHT<S>> {
    let gen_cache = Arc::new(Cache::from_entry(None));
    let mut db = LMTHT { storage: Box::new(storage), latest_cache: gen_cache };
    db.init()?;
    Ok(db)
  }

  /// ç¾å¨ã®æ¨æ§é ã®ã«ã¼ããã¼ããåç§ãã¾ãã
  pub fn root(&self) -> Option<Node> {
    self.latest_cache.root()
  }

  /// æ¨æ§é ã®ç¾å¨ã®ä¸ä»£ (ãªã¹ãã¨ãã¦ä½åã®è¦ç´ ãä¿æãã¦ããã) ãè¿ãã¾ãã
  pub fn n(&self) -> Index {
    self.latest_cache.n()
  }

  /// ãã® LMTHT ã®ç¾å¨ã®é«ããåç§ãã¾ãããã¼ããä¸ã¤ãå«ã¾ãã¦ããªãå ´åã¯ 0 ãè¿ãã¾ãã
  pub fn height(&self) -> u8 {
    self.root().map(|root| root.j).unwrap_or(0)
  }

  /// ãã® LMTHT ã®ã«ã¼ãããã·ã¥ãåç§ãã¾ããä¸ã¤ã®ãã¼ããå«ã¾ãã¦ããªãå ´åã¯ `None` ãè¿ãã¾ãã
  pub fn root_hash(&self) -> Option<Hash> {
    self.root().map(|root| root.hash)
  }

  pub fn storage(&self) -> &S {
    self.storage.as_ref()
  }

  fn init(&mut self) -> Result<()> {
    let mut cursor = self.storage.open(true)?;
    let length = cursor.seek(io::SeekFrom::End(0))?;
    match length {
      0 => {
        // ãã¸ãã¯ãã³ãã¼ã®æ¸ãè¾¼ã¿
        cursor.write_all(&STORAGE_IDENTIFIER)?;
        cursor.write_u8(STORAGE_VERSION)?;
      }
      1..=3 => return Err(FileIsNotContentsOfLMTHTree { message: "bad magic number" }),
      _ => {
        // ãã¸ãã¯ãã³ãã¼ã®ç¢ºèª
        let mut buffer = [0u8; 4];
        cursor.seek(io::SeekFrom::Start(0))?;
        cursor.read_exact(&mut buffer)?;
        if buffer[..3] != STORAGE_IDENTIFIER[..] {
          return Err(FileIsNotContentsOfLMTHTree { message: "bad magic number" });
        } else if !is_version_compatible(buffer[3]) {
          return Err(IncompatibleVersion(buffer[3] >> 4, buffer[3] & 0x0F));
        }
      }
    }

    let length = cursor.seek(io::SeekFrom::End(0))?;
    let tail = if length == 4 {
      None
    } else {
      // æ«å°¾ã®ã¨ã³ããªãèª­ã¿è¾¼ã¿
      back_to_safety(cursor.as_mut(), 4 + 8, "The first entry is corrupted.")?;
      let offset = cursor.read_u32::<LittleEndian>()?;
      back_to_safety(cursor.as_mut(), offset + 4, "The last entry is corrupted.")?;
      let entry = read_entry(&mut cursor, 0)?;
      if cursor.stream_position()? != length {
        // å£ããã¹ãã¬ã¼ã¸ããèª­ã¿è¾¼ãã  offset ãããã¾ãã¾ã©ããã®æ­£ããã¨ã³ããªå¢çãæãã¦ããå ´åãæ­£ãã
        // èª­ã¿è¾¼ãããçµæã¨ãªãä½ç½®ã¯æ«å°¾ã¨ä¸è´ããªãã
        let msg = "The last entry is corrupted.".to_string();
        return Err(DamagedStorage(msg));
      }
      Some(entry)
    };

    // ã­ã£ãã·ã¥ãæ´æ°
    let new_cache = Cache::from_entry(tail);
    self.latest_cache = Arc::new(new_cache);

    Ok(())
  }

  /// æå®ãããå¤ããã® LMTHT ã«è¿½å ãã¾ãã
  ///
  /// # Returns
  /// ãã®æä½ã«ãã£ã¦æ´æ°ãããã«ã¼ããã¼ããè¿ãã¾ãããã®ã«ã¼ããã¼ãã¯æ°ããæ¨æ§é ã®ã«ã¼ãããã·ã¥ã§ãã
  /// `hash` ã«å ãã¦ãããã·ã¥æ¨ã«å«ã¾ããè¦ç´ æ° `i`ãããã·ã¥æ¨ã®é«ã `j` ãæã¡ã¾ãã
  ///
  pub fn append(&mut self, value: &[u8]) -> Result<Node> {
    if value.len() > MAX_PAYLOAD_SIZE {
      return Err(TooLargePayload { size: value.len() });
    }

    let mut cursor = self.storage.open(true)?;

    // èãã¼ãã®æ§ç¯
    let position = cursor.seek(SeekFrom::End(0))?;
    let i = self.latest_cache.root().map(|node| node.i + 1).unwrap_or(1);
    let hash = Hash::hash(value);
    let enode = ENode { meta: MetaInfo::new(Address::new(i, 0, position), hash), payload: Vec::from(value) };

    // ä¸­éãã¼ãã®æ§ç¯
    let mut inodes = Vec::<INode>::with_capacity(INDEX_SIZE as usize);
    let mut right_hash = enode.meta.hash;
    let gen = NthGenHashTree::new(i);
    let mut right_to_left_inodes = gen.inodes();
    right_to_left_inodes.reverse();
    for n in right_to_left_inodes.iter() {
      debug_assert_eq!(i, n.node.i);
      debug_assert_eq!(n.node.i, n.right.i);
      debug_assert!(n.node.j >= n.right.j + 1);
      debug_assert!(n.left.j >= n.right.j);
      if let Some(left) = Query::get_node(&self.latest_cache, &mut cursor, n.left.i, n.left.j)? {
        let right = Address::new(n.right.i, n.right.j, position);
        let hash = left.hash.combine(&right_hash);
        let node = MetaInfo::new(Address::new(n.node.i, n.node.j, position), hash);
        let inode = INode::new(node, left.address, right);
        inodes.push(inode);
        right_hash = hash;
      } else {
        // åé¨ã®æ¨æ§é ã¨ã¹ãã¬ã¼ã¸ä¸ã®ãã¼ã¿ãçç¾ãã¦ãã
        return inconsistency(format!("cannot find the node b_{{{},{}}}", n.left.i, n.left.j));
      }
    }

    // è¿å¤ã®ããã®é«ãã¨ã«ã¼ãããã·ã¥ãåå¾
    let (j, root_hash) =
      if let Some(inode) = inodes.last() { (inode.meta.address.j, inode.meta.hash) } else { (0u8, enode.meta.hash) };

    // ã¨ã³ããªãæ¸ãè¾¼ãã§ç¶æãæ´æ°
    cursor.seek(SeekFrom::End(0))?;
    let entry = Entry { enode, inodes };
    write_entry(&mut cursor, &entry)?;

    // ã­ã£ãã·ã¥ãæ´æ°
    self.latest_cache = Arc::new(Cache::new(entry, gen));

    Ok(Node::new(i, j, root_hash))
  }

  pub fn query(&self) -> Result<Query> {
    let cursor = self.storage.open(false)?;
    let gen = self.latest_cache.clone();
    Ok(Query { cursor, gen })
  }
}

pub struct Query {
  cursor: Box<dyn Cursor>,
  gen: Arc<Cache>,
}

impl Query {
  /// ãã®ã¯ã¨ãªã¼ãå¯¾è±¡ã¨ãã¦ããæ¨æ§é ã®ä¸ä»£ãåç§ãã¾ãã
  pub fn n(&self) -> Index {
    self.gen.n()
  }

  /// ç¯å²å¤ã®ã¤ã³ããã¯ã¹ (0 ãå«ã) ãæå®ããå ´åã¯ `None` ãè¿ãã¾ãã
  pub fn get(&mut self, i: Index) -> Result<Option<Vec<u8>>> {
    if let Some(node) = Self::get_node(self.gen.as_ref(), &mut self.cursor, i, 0)? {
      self.cursor.seek(io::SeekFrom::Start(node.address.position))?;
      let entry = read_entry_without_check(&mut self.cursor, node.address.position, node.address.i)?;
      let Entry { enode: ENode { payload, .. }, .. } = entry;
      Ok(Some(payload))
    } else {
      Ok(None)
    }
  }

  /// èãã¼ã b_i ã®å¤ãä¸­éãã¼ãã®ããã·ã¥å¤ä»ãã§åå¾ãã¾ãã
  #[inline]
  pub fn get_with_hashes(&mut self, i: Index) -> Result<Option<ValuesWithBranches>> {
    self.get_values_with_hashes(i, 0)
  }

  /// æå®ããããã¼ã b_{i,j} ãã«ã¼ãã¨ããé¨åæ¨ã«å«ã¾ãã¦ãããã¹ã¦ã®å¤ (èãã¼ã) ãä¸­éãã¼ãã®ããã·ã¥å¤
  /// ä»ãã§åå¾ãã¾ãããã®çµæããç®åºãããã«ã¼ãããã·ã¥ãä½¿ç¨ãã¦ãå¤ã®ãã¼ã¿ãç ´æãæ¹ããããã¦ããªããã¨ã
  /// æ¤è¨¼ãããã¨ãã§ãã¾ãã
  ///
  /// # Returns
  /// è¿å¤ã«ã¯ç¯å²ã«å«ã¾ãã 1 åä»¥ä¸ã®å¤ã¨ãb_{i,j} ã¸ã®çµè·¯ããåå²ãããã¼ããå«ã¾ãã¦ãã¾ããããã§å¾ããã
  /// å¤ã®ç¯å²ã¯ [model::range(i,j)](range) ãä½¿ã£ã¦ç®åºãããã¨ãã§ãã¾ããb_{i,j} ãã«ã¼ãã¨ãã
  /// [é¨åæ¨ãå®å¨äºåæ¨](model::is_pbst) ã®å ´åãè¿å¤ã®æ°ã¯ `1 << j` åã«ãªãã¾ããå®å¨äºåæ¨ã§ãªãå ´åã¯
  /// `1 << j` ããå°ãªãåæ°ã¨ãªãã¾ãã
  ///
  /// `i` ã« 0 ãå«ãç¯å²å¤ã®ã¤ã³ããã¯ã¹ãæå®ããå ´åã¯ `None` ãè¿ãã¾ãã
  ///
  /// # Example
  /// ```rust
  /// use lmtht::{LMTHT, MemStorage, Hash};
  /// use lmtht::model::{range, is_pbst};
  ///
  /// let mut db = LMTHT::new(MemStorage::new()).unwrap();
  /// let mut latest_root_hash = Hash::hash(&vec![]);
  /// for i in 0u32..100 {
  ///   let current_root = db.append(&i.to_le_bytes()).unwrap();
  ///   latest_root_hash = current_root.hash;
  /// }
  /// let mut query = db.query().unwrap();
  /// let values = query.get_values_with_hashes(40, 3).unwrap().unwrap();
  /// assert!(is_pbst(40, 3));
  /// assert_eq!(1 << 3, values.values.len());
  /// assert_eq!(*range(40, 3).start(), values.values[0].i);
  /// assert_eq!(*range(40, 3).end(), values.values[(1 << 3) - 1].i);
  /// assert_eq!(latest_root_hash, values.root().hash);
  /// ```
  ///
  pub fn get_values_with_hashes(&mut self, i: Index, j: u8) -> Result<Option<ValuesWithBranches>> {
    let (last_entry, model) = if let Some(CacheInner { last_entry, model }) = &self.gen.0 {
      if i == 0 || i > model.n() {
        return Ok(None);
      }
      (last_entry, model)
    } else {
      return Ok(None);
    };
    let root = match self.gen.root_ref() {
      RootRef::INode(inode) => *inode,
      RootRef::ENode(enode) => {
        self.cursor.seek(SeekFrom::Start(enode.meta.address.position))?;
        let Entry { enode: ENode { payload, .. }, .. } =
          read_entry_without_check(&mut self.cursor, enode.meta.address.position, i)?;
        return Ok(Some(ValuesWithBranches { values: vec![Value { i, value: payload }], branches: vec![] }));
      }
      RootRef::None => return Ok(None),
    };
    let path = match model.path_to(i, j) {
      Some(path) => path,
      None => return Ok(None),
    };
    debug_assert_eq!(model.root().i, root.meta.address.i);
    debug_assert_eq!(model.root().j, root.meta.address.j);

    // ç®çã®ãã¼ãã¾ã§çµè·¯ãç§»åããªããåå²ã®ããã·ã¥å¤ãåå¾ãã
    let mut prev = root;
    let mut inodes = last_entry.inodes.clone();
    let mut branches = Vec::<Node>::with_capacity(INDEX_SIZE as usize);
    for step in path.steps.iter().map(|s| s.step) {
      // å·¦æå´ã®ã¨ã³ããªã® INode ãèª­ã¿è¾¼ã¿ (å³æå´ã®ãã¼ãã¯ inodes ã«å«ã¾ãã¦ãã)
      self.cursor.seek(SeekFrom::Start(prev.left.position))?;
      let left_inodes = read_inodes(&mut self.cursor, prev.left.position)?;

      // å·¦å³ã©ã¡ãã®æãæ¬¡ã®ãã¼ãã§ã©ã¡ããåå²ã®ãã¼ãããå¤æ­
      let (next, next_inodes, branch, branch_inodes) = if prev.left.i == step.i && prev.left.j == step.j {
        (&prev.left, left_inodes, &prev.right, inodes)
      } else {
        debug_assert!(prev.right.i == step.i && prev.right.j == step.j);
        (&prev.right, inodes, &prev.left, left_inodes)
      };

      // åå²ãããã¼ãã®ããã·ã¥å¤ä»ãã®æå ±ãä¿å­
      if branch.j > 0 {
        // INode ã¨ãã¦åå²ãããã¼ããåç§ãã¦ä¿å­
        if let Some(inode) = branch_inodes.iter().find(|n| n.meta.address.j == branch.j) {
          debug_assert!(inode.meta.address == *branch);
          branches.push(Node::for_node(&inode.meta));
        } else {
          return inconsistency(format!(
            "in searching for b_{{{},{}}} in T_{}, branch inode b_{{{}, {}}} isn't included in {:?}",
            i,
            j,
            self.n(),
            branch.i,
            branch.j,
            branch_inodes
          ));
        }
      } else {
        // ENode ã¨ãã¦åå²ãããã¼ããèª­ã¿è¾¼ãã§ä¿å­
        self.cursor.seek(SeekFrom::Start(branch.position))?;
        let entry = read_entry_without_check(&mut self.cursor, branch.position, branch.i)?;
        branches.push(Node::for_node(&entry.enode.meta));
      }

      if next.j == 0 {
        debug_assert_eq!((i, j), (next.i, next.j), "branch={:?}", branch);
        self.cursor.seek(SeekFrom::Start(next.position))?;
        let Entry { enode: ENode { payload, .. }, .. } =
          read_entry_without_check(&mut self.cursor, next.position, next.i)?;
        let values = vec![Value { i: next.i, value: payload }];
        return Ok(Some(ValuesWithBranches::new(values, branches)));
      }

      // æ¬¡ã®ãã¼ãã«ç§»å
      if let Some(inode) = next_inodes.iter().find(|node| node.meta.address == *next) {
        prev = *inode;
        inodes = next_inodes;
      } else {
        return inconsistency(format!(
          "in searching for ({},{}), the inode ({}, {}) on the route isn't included in {:?}",
          i, j, next.i, next.j, next_inodes
        ));
      }
    }

    // ç®çã®ãã¼ãã«å«ã¾ãã¦ããå¤ãåå¾ãã
    let values = self.get_values_belonging_to(&prev)?;
    Ok(Some(ValuesWithBranches::new(values, branches)))
  }

  fn get_node(gen: &Cache, cursor: &mut Box<dyn Cursor>, i: Index, j: u8) -> Result<Option<MetaInfo>> {
    if let Some((position, _)) = Self::get_entry_position(gen, cursor, i, false)? {
      cursor.seek(io::SeekFrom::Start(position))?;
      if j == 0 {
        let entry = read_entry_without_check(cursor, position, i)?;
        Ok(Some(entry.enode.meta))
      } else {
        let inodes = read_inodes(cursor, position)?;
        Ok(inodes.iter().find(|inode| inode.meta.address.j == j).map(|inode| inode.meta))
      }
    } else {
      Ok(None)
    }
  }

  /// æå®ããã `inode` ãã«ã¼ãã¨ããé¨åæ¨ã«å«ã¾ãã¦ãããã¹ã¦ã®å¤ãåç§ãã¾ããèª­ã¿åºãç¨ã®ã«ã¼ã½ã«ã¯ `inode`
  /// ã®ä½ç½®ãæãã¦ããå¿è¦ã¯ããã¾ããã
  fn get_values_belonging_to(&mut self, inode: &INode) -> Result<Vec<Value>> {
    // inode ãå·¦ææ¹åã«èã«å°éããã¾ã§ç§»å
    let mut mover = *inode;
    while mover.left.j > 0 {
      self.cursor.seek(SeekFrom::Start(mover.left.position))?;
      let inodes = read_inodes(&mut self.cursor, mover.left.position)?;
      mover = match inodes.iter().find(|node| node.meta.address.j == mover.left.j) {
        Some(inode) => *inode,
        None => panic!(),
      };
    }

    let range = range(inode.meta.address.i, inode.meta.address.j);
    let (i0, i1) = (*range.start(), *range.end());
    let mut values = Vec::<Value>::with_capacity((i1 - i0) as usize);
    let mut i = mover.left.i;
    self.cursor.seek(SeekFrom::Start(mover.left.position))?;
    while i <= i1 {
      let Entry { enode: ENode { meta: node, payload }, .. } = read_entry_without_check_to_end(&mut self.cursor, i)?;
      debug_assert!(node.address.i == i);
      values.push(Value { i, value: payload });
      i += 1;
    }
    Ok(values)
  }

  /// `i` çªç®ã®ã¨ã³ããªã®ä½ç½®ãåç§ãã¾ãããã®æ¤ç´¢ã¯ç¾å¨ã®ã«ã¼ããã¼ããåºæºã«ããæ¢ç´¢ãè¡ãã¾ãã
  fn get_entry_position(
    gen: &Cache,
    cursor: &mut Box<dyn Cursor>,
    i: Index,
    with_branch: bool,
  ) -> Result<Option<(Index, Vec<MetaInfo>)>> {
    match &gen.root_ref() {
      RootRef::INode(root) => {
        let root = (*root).clone();
        search_entry_position(cursor, &root, i, with_branch)
      }
      RootRef::ENode(root) if root.meta.address.i == i => Ok(Some((root.meta.address.position, vec![]))),
      _ => Ok(None),
    }
  }
}

/// æå®ãããã«ã¼ã½ã«ã®ç¾å¨ã®ä½ç½®ããã¨ã³ããªãèª­ã¿è¾¼ã¿ã¾ãã
/// æ­£å¸¸çµäºæã®ã«ã¼ã½ã«ã¯æ¬¡ã®ã¨ã³ããªãæãã¦ãã¾ãã
fn read_entry<C>(r: &mut C, i_expected: Index) -> Result<Entry>
where
  C: io::Read + io::Seek,
{
  let position = r.stream_position()?;
  let mut hasher = HighwayBuilder::new(Key(CHECKSUM_HW64_KEY));
  let mut r = HashRead::new(r, &mut hasher);
  let entry = read_entry_without_check(&mut r, position, i_expected)?;

  // ãªãã»ããã®æ¤è¨¼
  let offset = r.length();
  let trailer_offset = r.read_u32::<LittleEndian>()?;
  if offset != trailer_offset as u64 {
    return Err(IncorrectEntryHeadOffset { expected: trailer_offset, actual: offset });
  }

  // ãã§ãã¯ãµã ã®æ¤è¨¼
  let checksum = r.finish();
  let trailer_checksum = r.read_u64::<LittleEndian>()?;
  if checksum != trailer_checksum {
    let length = offset as u32 + 4 + 8;
    return Err(ChecksumVerificationFailed { at: position, length, expected: trailer_checksum, actual: checksum });
  }

  Ok(entry)
}

/// æå®ãããã«ã¼ã½ã«ã®ç¾å¨ã®ä½ç½®ãã checksum ã«ããæ¤è¨¼ãªãã§ã¨ã³ããªãèª­ã¿è¾¼ã¿ã¾ããæ­£å¸¸çµäºæã®ã«ã¼ã½ã«ã®ä½ç½®ã¯
/// æ¬¡ã®ã¨ã³ããªã®æ¦éãæãã¦ãã¾ãã
fn read_entry_without_check_to_end<C>(r: &mut C, i_expected: Index) -> Result<Entry>
where
  C: io::Read + io::Seek,
{
  let position = r.stream_position()?;
  let entry = read_entry_without_check(r, position, i_expected)?;
  r.seek(SeekFrom::Current(4 /* offset */ + 8 /* checksum */))?;
  Ok(entry)
}

/// æå®ãããã«ã¼ã½ã«ã®ç¾å¨ã®ä½ç½®ããã¨ã³ããªãèª­ã¿è¾¼ã¿ã¾ãããã¬ã¤ã©ã¼ã® offset ã¨ checksum ã¯èª­ã¿è¾¼ã¾ããªã
/// ãããæ­£å¸¸çµäºæã®ã«ã¼ã½ã«ã¯ offset ã®ä½ç½®ãæãã¦ãã¾ãã
fn read_entry_without_check(r: &mut dyn io::Read, position: u64, i_expected: Index) -> Result<Entry> {
  let mut hash = [0u8; HASH_SIZE];

  // ä¸­éãã¼ãã®èª­ã¿è¾¼ã¿
  let inodes = read_inodes(r, position)?;
  let i = inodes.first().map(|inode| inode.meta.address.i).unwrap_or(1);
  if i != i_expected && i_expected != 0 {
    return Err(Detail::IncorrectNodeBoundary { at: position });
  }

  // èãã¼ãã®èª­ã¿è¾¼ã¿
  let payload_size = r.read_u32::<LittleEndian>()? & MAX_PAYLOAD_SIZE as u32;
  let mut payload = Vec::<u8>::with_capacity(payload_size as usize);
  unsafe { payload.set_len(payload_size as usize) };
  r.read_exact(&mut payload)?;
  r.read_exact(&mut hash)?;
  let enode = ENode { meta: MetaInfo::new(Address::new(i, 0, position), Hash::new(hash)), payload };

  Ok(Entry { enode, inodes })
}

/// æå®ãããã«ã¼ã½ã«ã®ç¾å¨ã®ä½ç½®ãã¨ã³ããªã®åé ­ã¨ãã¦ãã¹ã¦ã® `INode` ãèª­ã¿è¾¼ã¿ã¾ããæ­£å¸¸çµäºããå ´åãã«ã¼ã½ã«
/// ä½ç½®ã¯æå¾ã® `INode` ãèª­ã¿è¾¼ãã ç´å¾ãæãã¦ãã¾ãã
fn read_inodes(r: &mut dyn io::Read, position: u64) -> Result<Vec<INode>> {
  let mut hash = [0u8; HASH_SIZE];
  let i = r.read_u64::<LittleEndian>()?;
  let inode_count = r.read_u8()?;
  let mut right_j = 0u8;
  let mut inodes = Vec::<INode>::with_capacity(inode_count as usize);
  for _ in 0..inode_count as usize {
    let j = (r.read_u8()? & (INDEX_SIZE - 1)) + 1; // ä¸ä½ 6-bit ã®ã¿ãä½¿ç¨
    let left_position = r.read_u64::<LittleEndian>()?;
    let left_i = r.read_u64::<LittleEndian>()?;
    let left_j = r.read_u8()?;
    r.read_exact(&mut hash)?;
    inodes.push(INode {
      meta: MetaInfo::new(Address::new(i, j, position), Hash::new(hash)),
      left: Address::new(left_i, left_j, left_position),
      right: Address::new(i, right_j, position),
    });
    right_j = j;
  }
  Ok(inodes)
}

/// æå®ãããã«ã¼ã½ã«ã«ã¨ã³ããªãæ¸ãè¾¼ã¿ã¾ãã
/// ãã®ã¨ã³ããªã«å¯¾ãã¦æ¸ãè¾¼ã¿ãè¡ãããé·ããè¿ãã¾ãã
fn write_entry(w: &mut dyn Write, e: &Entry) -> Result<usize> {
  debug_assert!(e.enode.payload.len() <= MAX_PAYLOAD_SIZE);
  debug_assert!(e.inodes.len() <= 0xFF);

  let mut hasher = HighwayBuilder::new(Key(CHECKSUM_HW64_KEY));
  let mut w = HashWrite::new(w, &mut hasher);

  // ä¸­éãã¼ãã®æ¸ãè¾¼ã¿
  w.write_u64::<LittleEndian>(e.enode.meta.address.i)?;
  w.write_u8(e.inodes.len() as u8)?;
  for i in &e.inodes {
    debug_assert_eq!((i.meta.address.j - 1) & (INDEX_SIZE - 1), i.meta.address.j - 1);
    w.write_u8((i.meta.address.j - 1) & (INDEX_SIZE - 1))?; // ä¸ä½ 6-bit ã®ã¿ä¿å­
    w.write_u64::<LittleEndian>(i.left.position)?;
    w.write_u64::<LittleEndian>(i.left.i)?;
    w.write_u8(i.left.j)?;
    w.write_all(&i.meta.hash.value)?;
  }

  // èãã¼ãã®æ¸ãè¾¼ã¿
  w.write_u32::<LittleEndian>(e.enode.payload.len() as u32)?;
  w.write_all(&e.enode.payload)?;
  w.write_all(&e.enode.meta.hash.value)?;

  // ã¨ã³ããªåé ­ã¾ã§ã®ãªãã»ãããæ¸ãè¾¼ã¿
  w.write_u32::<LittleEndian>(w.length() as u32)?;

  // ãã§ãã¯ãµã ã®æ¸ãè¾¼ã¿
  w.write_u64::<LittleEndian>(w.finish())?;

  Ok(w.length() as usize)
}

/// `root` ã«æå®ãããä¸­éãã¼ããé¨åæ¨æ§é ã®ã«ã¼ãã¨ãã¦ b_{i,*} ã«è©²å½ããèãã¼ãã¨ä¸­éãã¼ããå«ãã§ãã
/// ã¨ã³ããªã®ã¹ãã¬ã¼ã¸åã§ã®ä½ç½®ãåå¾ãã¾ããè©²å½ããã¨ã³ããªãå­å¨ããªãå ´åã¯ `None` ãè¿ãã¾ãã
///
/// `with_branch` ã« true ãæå®ããå ´åãè¿å¤ã«ã¯ `root` ããæ¤ç´¢å¯¾è±¡ã®ãã¼ãã«è³ãã¾ã§ã®åå²åã®ããã·ã¥å¤ã
/// æã¤ãã¼ããå«ã¾ãã¾ããããã¯ããã·ã¥ããªã¼ããããã·ã¥ä»ãã§å¤ãåç§ããããã®åä½ã§ããfalse ãæå®ããå ´åã¯
/// é·ã 0 ã® `Vec` ãè¿ãã¾ãã
///
fn search_entry_position<C>(
  r: &mut C,
  root: &INode,
  i: Index,
  with_branch: bool,
) -> Result<Option<(u64, Vec<MetaInfo>)>>
where
  C: io::Read + io::Seek,
{
  if root.meta.address.i == i {
    // æå®ãããã«ã¼ããã¼ããæ¤ç´¢å¯¾è±¡ã®ãã¼ãã®å ´å
    return Ok(Some((root.meta.address.position, vec![])));
  } else if i == 0 || i > root.meta.address.i {
    // ã¤ã³ããã¯ã¹ 0 ã®ç¹æ®å¤ãæã¤ãã¼ãã¯æç¤ºçã«å­å¨ããªã
    return Ok(None);
  }

  let mut branches = Vec::<MetaInfo>::with_capacity(INDEX_SIZE as usize);
  let mut mover = root.clone();
  for _ in 0..INDEX_SIZE {
    // æ¬¡ã®ãã¼ãã®ã¢ãã¬ã¹ãåç§
    let next = if i <= mover.left.i {
      read_branch(r, &mover.right, with_branch, &mut branches)?;
      mover.left
    } else if i <= mover.meta.address.i {
      read_branch(r, &mover.left, with_branch, &mut branches)?;
      mover.right
    } else {
      // æå¹ç¯å²å¤
      return Ok(None);
    };

    // æ¬¡ã®ãã¼ãã®ã¢ãã¬ã¹ãæ¤ç´¢å¯¾è±¡ãªããã®ã¨ã³ããªã®ä½ç½®ãè¿ã
    if next.i == i {
      return Ok(Some((next.position, branches)));
    }

    // æ«ç«¯ã«å°éãã¦ããå ´åã¯çºè¦ã§ããªãã£ããã¨ãæå³ãã
    if next.j == 0 {
      return Ok(None);
    }

    // b_{i,*} ã®ä¸­éãã¼ããã­ã¼ããã¦æ¬¡ã®ä¸­éãã¼ããåå¾
    mover = read_inode(r, &next)?;
  }

  fn read_inode<C>(r: &mut C, addr: &Address) -> Result<INode>
  where
    C: io::Read + io::Seek,
  {
    debug_assert_ne!(0, addr.j);
    r.seek(io::SeekFrom::Start(addr.position))?;
    let inodes = read_inodes(r, addr.position)?;
    let inode = inodes.iter().find(|inode| inode.meta.address.j == addr.j);
    if let Some(inode) = inode {
      Ok(inode.clone())
    } else {
      // åé¨ã®æ¨æ§é ã¨ã¹ãã¬ã¼ã¸ä¸ã®ãã¼ã¿ãçç¾ãã¦ãã
      inconsistency(format!("entry i={} in storage doesn't contain an inode at specified level j={}", addr.i, addr.j))
    }
  }

  fn read_branch<C>(r: &mut C, addr: &Address, with_branch: bool, branches: &mut Vec<MetaInfo>) -> Result<()>
  where
    C: io::Read + io::Seek,
  {
    if with_branch {
      let branch = if addr.j == 0 {
        r.seek(io::SeekFrom::Start(addr.position))?;
        let entry = read_entry_without_check(r, addr.position, addr.i)?;
        entry.enode.meta
      } else {
        read_inode(r, &addr)?.meta
      };
      branches.push(branch);
    }
    Ok(())
  }

  // ã¹ãã¬ã¼ã¸ä¸ã®ãã¼ã¿ã®ãã¤ã³ã¿ãå¾ªç°åç§ãèµ·ããã¦ãã
  inconsistency(format!(
    "The maximum hop count was exceeded before reaching node b_{} from node b_{{{},{}}}.\
     The data on the storage probably have circular references.",
    i, root.meta.address.i, root.meta.address.j
  ))
}

/// æå®ãããã«ã¼ã½ã«ãç¾å¨ã®ä½ç½®ãã `distance` ãã¤ãåæ¹ã«ç§»åãã¾ããç§»ååãã«ã¼ã½ã«ã®åé ­ãè¶ããå ´åã¯
/// `if_err` ãã¡ãã»ã¼ã¸ã¨ããã¨ã©ã¼ãçºçãã¾ãã
#[inline]
fn back_to_safety(cursor: &mut dyn Cursor, distance: u32, if_err: &'static str) -> Result<u64> {
  let from = cursor.stream_position()?;
  let to = from - distance as u64;
  if to < STORAGE_IDENTIFIER.len() as u64 + 1 {
    Err(DamagedStorage(format!("{} (cannot move position from {} to {})", if_err, from, to)))
  } else {
    Ok(cursor.seek(io::SeekFrom::Current(-(distance as i64)))?)
  }
}

/// panic_over_inconsistency ãå®ç¾©ããã¦ããå ´åã¯ panic ãã¦åé¨çç¾ãæ¤åºããå ´æãç¥ãããã
fn inconsistency<T>(msg: String) -> Result<T> {
  #[cfg(feature = "panic_over_inconsistency")]
  {
    panic!("{}", msg)
  }
  #[cfg(not(feature = "panic_over_inconsistency"))]
  {
    Err(InternalStateInconsistency { message: msg })
  }
}

#[inline]
fn hex(value: &[u8]) -> String {
  value.iter().map(|c| format!("{:02X}", c)).collect()
}
