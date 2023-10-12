use crate::error::IterManError;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub trait ListLike {
    type Item;

    fn iter(&mut self) -> Option<Self::Item>;
}

/// A [MemoryList] is a [ListLike] that reads from a [Vec].
/// # Examples
/// ```no-run
/// let list = MemoryList::new(vec![2, 3, 4]);
/// assert_eq!(list.collect::<Vec<i32>>(), [2, 3, 4]);
/// ```
pub struct MemoryList<T: Clone> {
    vec: Arc<Mutex<Vec<T>>>,
    round_robin: bool,
    line_index: AtomicUsize,
}

impl<T: Clone> MemoryList<T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            vec: Arc::new(Mutex::new(vec)),
            round_robin: false,
            line_index: AtomicUsize::new(0),
        }
    }

    /// Creates a new [MemoryList] with `round_robin` turned on.
    pub fn new_round_robin(vec: Vec<T>) -> Self {
        Self {
            round_robin: true,
            ..Self::new(vec)
        }
    }

    /// Build a [MemoryList]] and set the initial `line_index` pointer.
    /// # Examples
    /// ```no-run
    /// let mut list = MemoryList::new_round_robin(vec![2, 3, 4]).with_seek_to(2);
    /// ```
    pub fn with_seek_to(mut self, line_index: usize) -> Self {
        self.seek(line_index).unwrap_or_default();
        self
    }

    /// Seek
    /// Should this be public?
    /// TODO: Revisit when persistence is added
    pub fn seek(&mut self, line_index: usize) -> Result<usize, IterManError> {
        if line_index < self.vec.lock().unwrap().len() {
            self.line_index.store(line_index, Ordering::Relaxed);
            return Ok(line_index);
        }

        Err(IterManError::MemoryOutOfBounds {
            line_index,
            max_len: self.vec.lock().unwrap().len(),
        })
    }

    pub fn line_index(&self) -> usize {
        self.line_index.load(Ordering::Relaxed)
    }
}

impl<T: Clone> ListLike for MemoryList<T> {
    type Item = T;

    fn iter(&mut self) -> Option<Self::Item> {
        if self.round_robin && self.line_index() >= self.vec.lock().unwrap().len() {
            self.line_index.store(0, Ordering::Relaxed);
        }

        if self.line_index() < self.vec.lock().unwrap().len() {
            let val = self.vec.lock().unwrap()[self.line_index()].clone();
            self.line_index.fetch_add(1, Ordering::SeqCst);
            Some(val)
        } else {
            None
        }
    }
}

impl<T: Clone> Iterator for MemoryList<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        MemoryList::iter(self)
    }
}

/// A [BufferList] is a [ListLike] that reads from a [BufReader].
/// # Examples
/// ```no-run
/// let reader = BufReader::new(Cursor::new("hello\nworld"));
/// let list = BufferList::new(reader);
/// assert_eq!(list.collect::<Vec<String>>(), ["hello", "world"]);
/// ```
pub struct BufferList<T: Read + Seek> {
    buf_reader: Arc<Mutex<BufReader<T>>>,
    round_robin: bool,
    line_index: AtomicUsize,
    bytes_offset: AtomicUsize,
}

impl<T: Read + Seek> BufferList<T> {
    pub fn new(buf_reader: BufReader<T>) -> Self {
        Self {
            buf_reader: Arc::new(Mutex::new(buf_reader)),
            round_robin: false,
            line_index: AtomicUsize::new(0),
            bytes_offset: AtomicUsize::new(0),
        }
    }

    /// Creates a new [BufferList] with `round_robin` turned on.
    pub fn new_round_robin(buf_reader: BufReader<T>) -> Self {
        Self {
            round_robin: true,
            ..Self::new(buf_reader)
        }
    }

    /// Build a [BufferList]] and set the initial `line_index` and `bytes_offset` pointers.
    /// # Examples
    /// ```no-run
    /// let reader = BufReader::new(Cursor::new("hello\nworld"));
    /// let list = StreamList::new(reader).with_seek_to(1, 6);
    /// ```
    pub fn with_seek_to(mut self, line_index: usize, bytes_offset: usize) -> Self {
        self.seek(line_index, bytes_offset).unwrap_or_default();
        self
    }

    /// Used internally to manage the line index and byte offset
    fn incr(&mut self, bytes_read: &usize) {
        self.line_index.fetch_add(1, Ordering::SeqCst);
        self.bytes_offset.fetch_add(*bytes_read, Ordering::SeqCst);
    }

    /// Reset the line index and byte offset
    pub fn reset(&mut self) {
        self.line_index.store(0, Ordering::Relaxed);
        self.bytes_offset.store(0, Ordering::Relaxed);
    }

    pub fn seek(&mut self, line_index: usize, bytes_offset: usize) -> Result<usize, IterManError> {
        // https://doc.rust-lang.org/stable/std/io/trait.Seek.html#method.stream_len
        let stream_len = match self.buf_reader.lock().unwrap().seek(SeekFrom::End(0)).ok() {
            None => {
                return Err(IterManError::StreamOutOfBounds {
                    line_index,
                    bytes_offset,
                    max_len: 0,
                })
            }
            Some(len) => len,
        };

        if stream_len < bytes_offset as u64 {
            return Err(IterManError::StreamOutOfBounds {
                line_index,
                bytes_offset,
                max_len: stream_len as usize,
            });
        }

        if self
            .buf_reader
            .lock()
            .unwrap()
            .seek(SeekFrom::Start(bytes_offset as u64))
            .ok()
            .is_some()
        {
            self.line_index.store(line_index, Ordering::Relaxed);
            self.bytes_offset.store(bytes_offset, Ordering::Relaxed);
            return Ok(self.bytes_offset());
        }

        Err(IterManError::StreamOutOfBounds {
            line_index,
            bytes_offset,
            max_len: stream_len as usize,
        })
    }

    pub fn line_index(&self) -> usize {
        self.line_index.load(Ordering::Relaxed)
    }

    pub fn bytes_offset(&self) -> usize {
        self.bytes_offset.load(Ordering::Relaxed)
    }
}

impl<T: Read + Seek> ListLike for BufferList<T> {
    type Item = String;

    fn iter(&mut self) -> Option<Self::Item> {
        let mut string = String::new();

        // Scope of immutable borrow is limited here.
        match {
            let mut buf = self.buf_reader.lock().ok()?;
            buf.read_line(&mut string).ok()?
        } {
            0 => {
                if !self.round_robin {
                    return None;
                }

                {
                    let mut buf = self.buf_reader.lock().ok()?;
                    buf.seek(SeekFrom::Start(0)).ok()?;
                }

                self.reset();

                return match {
                    let mut buf = self.buf_reader.lock().ok()?;
                    buf.read_line(&mut string)
                } {
                    Ok(bytes_read) => match bytes_read {
                        0 => None, // Needed to stop empty buffer from returning ""
                        _ => {
                            self.incr(&bytes_read);
                            Some(string.trim().to_string())
                        }
                    },
                    Err(_) => None,
                };
            }
            bytes_read => {
                self.incr(&bytes_read);
                Some(string.trim().to_string())
            }
        }
    }
}

impl<T: Read + Seek> Iterator for BufferList<T>
where
    T: Read + Seek,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        BufferList::iter(self)
    }
}

/// A [MemoryArrayList] is a [ListLike] that reads from a [Vec] of [Vec]s.
pub struct MemoryArrayList<T: Clone> {
    lists: Arc<Mutex<Vec<Vec<T>>>>,
    round_robin: bool,
    cur_list_index: AtomicUsize,
    line_indexes: Arc<Mutex<Vec<usize>>>,
    finished_count: AtomicUsize,
}

impl<T: Clone> MemoryArrayList<T> {
    /// Creates a new [MemoryArrayList] with `round_robin` turned off.
    /// # Examples
    /// ```no-run
    /// let mem_arr = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    /// let list = MemoryArrayList::new(mem_arr);
    /// assert_eq!(
    ///   list.collect::<Vec<i32>>(),
    ///  [1, 4, 7, 2, 5, 8, 3, 6, 9]
    /// );
    /// ```
    pub fn new(mem_arr: Vec<Vec<T>>) -> Self {
        Self {
            lists: Arc::new(Mutex::new(mem_arr.clone())),
            round_robin: false,
            cur_list_index: AtomicUsize::new(0),
            line_indexes: Arc::new(Mutex::new(vec![0; mem_arr.len()])),
            finished_count: AtomicUsize::new(0),
        }
    }

    /// Creates a new [MemoryArrayList] with `round_robin` turned on.
    /// # Examples
    /// ```no-run
    /// let mem_arr = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    /// let list = MemoryArrayList::new_round_robin(mem_arr);
    /// assert_eq!(
    ///    list.take(15).collect::<Vec<i32>>(),
    ///   [1, 4, 7, 2, 5, 8, 3, 6, 9]
    /// );
    /// ```
    pub fn new_round_robin(mem_arr: Vec<Vec<T>>) -> Self {
        Self {
            round_robin: true,
            ..Self::new(mem_arr)
        }
    }
}

impl<T: Clone> Iterator for MemoryArrayList<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        MemoryArrayList::iter(self)
    }
}

impl<T: Clone> ListLike for MemoryArrayList<T> {
    type Item = T;

    fn iter(&mut self) -> Option<Self::Item> {
        let mut cur_list_index = self.cur_list_index.load(Ordering::Relaxed);
        if cur_list_index >= self.lists.lock().unwrap().len() {
            self.cur_list_index.store(0, Ordering::Relaxed);
            cur_list_index = self.cur_list_index.load(Ordering::Relaxed);
        }

        let mut line_indexes = self.line_indexes.lock().unwrap();
        let cur_line_index = line_indexes[cur_list_index];

        let lists = &self.lists.lock().unwrap();
        if cur_line_index < lists[cur_list_index].len() {
            let val = lists[cur_list_index][cur_line_index].clone();

            line_indexes[cur_list_index] += 1;
            if self.round_robin && line_indexes[cur_list_index] >= lists[cur_list_index].len() {
                line_indexes[cur_list_index] = 0;
            }

            self.cur_list_index.fetch_add(1, Ordering::SeqCst);
            return Some(val);
        } else {
            if !self.round_robin {
                self.finished_count.fetch_add(1, Ordering::SeqCst);
                if self.finished_count.load(Ordering::Relaxed) >= lists.len() {
                    return None;
                }
            }
        }

        None
    }
}

pub struct BufferArrayList<T: Read + Seek> {
    buf_reader: Arc<Mutex<Vec<BufferList<T>>>>,
    finished: AtomicUsize,
    round_robin: bool,
    arr_index: AtomicUsize,
    line_indexes: Arc<Mutex<Vec<usize>>>,
    bytes_offset: AtomicUsize,
}

impl<T: Read + Seek> BufferArrayList<T> {
    pub fn new(buf_arr: Vec<BufferList<T>>) -> Self {
        let buf_len = &buf_arr.len();
        Self {
            buf_reader: Arc::new(Mutex::new(buf_arr)),
            round_robin: false,
            finished: AtomicUsize::new(0),
            arr_index: AtomicUsize::new(0),
            line_indexes: Arc::new(Mutex::new(vec![0; *buf_len])),
            bytes_offset: AtomicUsize::new(0),
        }
    }
}

impl<T: Read + Seek> Iterator for BufferArrayList<T>
where
    T: Read + Seek,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let string = String::new();
        Some(string)
    }
}

/// Create a [MemoryList] from a directory by reading each file into memory.
/// # Examples
/// ```no-run
/// let list = mem_list_from_dir("src", false).unwrap();
/// assert_eq!(list.collect::<Vec<String>>().len(), 4);
/// ```
/// # Errors
/// This function will return an error if the path is not a directory.
pub fn mem_list_from_dir(
    path: &str,
    round_robin: bool,
) -> Result<MemoryList<String>, std::io::Error> {
    let mut files = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && !path.is_symlink() {
            let contents = std::fs::read_to_string(path)?;
            files.push(contents);
        }
    }

    if round_robin {
        return Ok(MemoryList::new_round_robin(files));
    }
    Ok(MemoryList::new(files))
}

/// Create a [MemoryList] from a string by splitting it into chunks.
/// # Examples
/// ```no-run
/// let text = "hello world";
/// let list = mem_list_from_chunks(text, 5, true).unwrap();
/// assert_eq!(
///    list.take(6).collect::<Vec<String>>(),
///   ["hello", " worl", "d", "hello", " worl", "d"]
/// );
/// ```
pub fn mem_list_from_chunks(
    text: &str,
    chunk_by: usize,
    round_robin: bool,
) -> Result<MemoryList<String>, std::io::Error> {
    let mut chunks = vec![];
    for chunk in text.as_bytes().chunks(chunk_by) {
        chunks.push(String::from_utf8(chunk.to_vec()).unwrap());
    }

    if round_robin {
        return Ok(MemoryList::new_round_robin(chunks));
    }
    Ok(MemoryList::new(chunks))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    #[ignore]
    fn it_should_create_buffer_array_list() {
        let reader = mock_buffer_reader();
        let buf_reader = BufferList::new(reader);
        let list = BufferArrayList::new(vec![buf_reader]);
        assert_eq!(list.collect::<Vec<String>>(), ["1", "2", "3"]);
    }

    #[test]
    fn it_should_create_memory_array_lists() {
        let mem_arr = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let list = MemoryArrayList::new(mem_arr);
        assert_eq!(
            list.take(15).collect::<Vec<i32>>(),
            [1, 4, 7, 2, 5, 8, 3, 6, 9]
        );
    }

    #[test]
    fn it_should_create_memory_array_lists_with_round_robin() {
        let mem_arr = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let list = MemoryArrayList::new_round_robin(mem_arr);
        assert_eq!(
            list.take(15).collect::<Vec<i32>>(),
            [1, 4, 7, 2, 5, 8, 3, 6, 9, 1, 4, 7, 2, 5, 8]
        );
    }

    #[test]
    #[ignore]
    fn should_from_dir() {
        let dir = mem_list_from_dir("src", false).unwrap();
        assert_eq!(dir.collect::<Vec<String>>().len(), 4);
        // buffer iterator
        let reader = BufReader::new(Cursor::new("1\n2\n3\n"));
        let list = BufferList::new(reader); // will reach EOF and stop
        assert_eq!(list.collect::<Vec<String>>(), ["1", "2", "3"]);
    }

    #[test]
    fn it_should_create_a_mem_list_by_chunks() {
        let text = "hello world";
        let list = mem_list_from_chunks(text, 5, true).unwrap();
        assert_eq!(
            list.take(6).collect::<Vec<String>>(),
            ["hello", " worl", "d", "hello", " worl", "d"]
        );
    }

    #[test]
    fn memory_list_reaches_end_correctly_as_i32() {
        let list = MemoryList::new(vec![2, 3, 4]);
        let collected: Vec<i32> = list.collect();
        assert_eq!(collected, [2, 3, 4]);
    }

    #[test]
    fn memory_list_reaches_end_correctly_as_str() {
        let list = MemoryList::new(vec!["2", "3", "4"]);
        let collected: Vec<&str> = list.collect();
        assert_eq!(collected, ["2", "3", "4"]);
    }

    #[test]
    fn memory_list_round_robins_correctly() {
        let list = MemoryList::new_round_robin(vec![2, 3, 4]);
        let collected: Vec<i32> = list.take(6).collect();
        assert_eq!(collected, [2, 3, 4, 2, 3, 4]);
    }

    #[test]
    fn memory_list_should_return_nothing_when_empty() {
        let list = MemoryList::new_round_robin(vec![]);
        let collected: Vec<i32> = list.take(10).collect();
        assert_eq!(collected, []);
    }

    #[test]
    fn buffer_list_reaches_end_correctly() {
        let reader = mock_buffer_reader();
        let list = BufferList::new(reader);

        let collected: Vec<String> = list.collect();
        assert_eq!(collected, ["1", "2", "3"]);
    }

    #[test]
    fn buffer_list_round_robins_correctly() {
        let reader = mock_buffer_reader();
        let list = BufferList::new_round_robin(reader);

        let collected: Vec<String> = list.take(6).collect();
        assert_eq!(collected, ["1", "2", "3", "1", "2", "3"]);
    }

    #[test]
    fn buffer_list_should_return_nothing_with_an_empty_buffer() {
        let reader = BufReader::new(Cursor::new(""));
        let list = BufferList::new_round_robin(reader);

        let collected: Vec<String> = list.take(10).collect();
        assert_eq!(collected.len(), 0);
    }

    #[test]
    fn memory_list_should_seek() {
        let mut list = MemoryList::new_round_robin(vec![2, 3, 4]);
        list.seek(2).expect("TODO: panic message");
        assert_eq!(list.next(), Some(4));
        assert_eq!(list.line_index(), 3);
    }

    #[test]
    fn memory_list_with_seek_to() {
        let mut list = MemoryList::new_round_robin(vec![2, 3, 4]).with_seek_to(2);
        assert_eq!(list.next(), Some(4));
        assert_eq!(list.line_index(), 3);
    }

    #[test]
    fn memory_list_seek_should_return_false_if_out_of_bounds() {
        let mut list = MemoryList::new(vec![2, 3, 4]);
        let e = list.seek(6).unwrap_err();
        assert_eq!(
            e,
            IterManError::MemoryOutOfBounds {
                line_index: 6,
                max_len: 3,
            }
        );
    }

    #[test]
    fn buffer_list_should_seek() {
        let reader = mock_buffer_reader();
        let mut list = BufferList::new(reader);
        list.seek(2, 4).expect("TODO: panic message");
        assert_eq!(list.next(), Some("3".to_string()));
        assert_eq!(list.line_index(), 3);
        assert_eq!(list.bytes_offset(), 6);
    }

    #[test]
    fn buffer_list_with_seek_to() {
        let reader = mock_buffer_reader();
        let mut list = BufferList::new(reader).with_seek_to(2, 4);
        assert_eq!(list.next(), Some("3".to_string()));
        assert_eq!(list.line_index(), 3);
        assert_eq!(list.bytes_offset(), 6);
    }

    #[test]
    fn buffer_list_seek_should_return_false_if_out_of_bounds() {
        let reader = mock_buffer_reader();
        let mut list = BufferList::new(reader);
        let e = list.seek(7, 50).unwrap_err();
        assert_eq!(
            e,
            IterManError::StreamOutOfBounds {
                line_index: 7,
                bytes_offset: 50,
                max_len: 6,
            }
        );
    }

    fn mock_buffer_reader<'a>() -> BufReader<Cursor<&'a str>> {
        let reader = BufReader::new(Cursor::new("1\n2\n3\n"));
        reader
    }
}
