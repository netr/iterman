use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

trait ListLike {
    type Item;

    fn iter(&mut self) -> Option<Self::Item>;

    // TODO: Add write() method
    // TODO: Add seek() method
}

struct MemoryList<T: Clone> {
    vec: Vec<T>,
    round_robin: bool,
    index: usize,
}

impl<T: Clone> MemoryList<T> {
    pub fn new(vec: Vec<T>, round_robin: bool) -> Self {
        Self {
            vec,
            round_robin,
            index: 0,
        }
    }

    /// Creates a new [MemoryList] with `round_robin` turned on.
    pub fn new_rr(vec: Vec<T>) -> Self {
        Self {
            vec,
            round_robin: true,
            index: 0,
        }
    }
}

impl<T: Clone> ListLike for MemoryList<T> {
    type Item = T;

    fn iter(&mut self) -> Option<Self::Item> {
        if self.round_robin && self.index >= self.vec.len() {
            self.index = 0;
        }

        if self.index < self.vec.len() {
            let val = self.vec[self.index].clone();
            self.index += 1;
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

struct StreamList<T: Read + Seek> {
    buf_reader: BufReader<T>,
    round_robin: bool,
    line_index: usize,
    bytes_offset: usize,
}

impl<T: Read + Seek> StreamList<T> {
    pub fn new(buf_reader: BufReader<T>, round_robin: bool) -> Self {
        Self {
            buf_reader,
            round_robin,
            line_index: 0,
            bytes_offset: 0,
        }
    }

    /// Creates a new [StreamList] with `round_robin` turned on.
    pub fn new_rr(buf_reader: BufReader<T>) -> Self {
        Self {
            buf_reader,
            round_robin: true,
            line_index: 0,
            bytes_offset: 0,
        }
    }

    /// Used internally to manage the line index and byte offset
    fn incr(&mut self, bytes_read: &usize) {
        self.line_index += 1;
        self.bytes_offset += bytes_read;
    }

    /// Used internally to reset the line index and byte offset
    fn reset(&mut self) {
        self.line_index = 0;
        self.bytes_offset = 0;
    }
}

impl<T: Read + Seek> ListLike for StreamList<T> {
    type Item = String;

    fn iter(&mut self) -> Option<Self::Item> {
        let mut string = String::new();

        match self.buf_reader.read_line(&mut string).ok()? {
            0 => {
                if !self.round_robin {
                    return None;
                }

                self.buf_reader.seek(SeekFrom::Start(0)).ok()?;
                self.reset();

                return match self.buf_reader.read_line(&mut string) {
                    Ok(bytes_read) => match bytes_read {
                        0 => None, // Needed to stop empty buffers from returning ""
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

impl<T: Read + Seek> Iterator for StreamList<T>
where
    T: Read + Seek,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        StreamList::iter(self)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn memory_list_reaches_end_correctly_as_i32() {
        let list_iter = MemoryList::new(vec![2, 3, 4], false);
        let collected: Vec<i32> = list_iter.collect();
        assert_eq!(collected, [2, 3, 4]);
    }

    #[test]
    fn memory_list_reaches_end_correctly_as_str() {
        let list_iter = MemoryList::new(vec!["2", "3", "4"], false);
        let collected: Vec<&str> = list_iter.collect();
        assert_eq!(collected, ["2", "3", "4"]);
    }

    #[test]
    fn memory_list_round_robins_correctly() {
        let list_iter = MemoryList::new_rr(vec![2, 3, 4]);
        let collected: Vec<i32> = list_iter.take(6).collect();
        assert_eq!(collected, [2, 3, 4, 2, 3, 4]);
    }

    #[test]
    fn memory_list_should_return_nothing_when_empty() {
        let list_iter = MemoryList::new_rr(vec![]);
        let collected: Vec<i32> = list_iter.take(10).collect();
        assert_eq!(collected, []);
    }

    #[test]
    fn stream_list_reaches_end_correctly() {
        let mock_data = "1\n2\n3\n";
        let cursor = Cursor::new(mock_data);
        let reader = BufReader::new(cursor);
        let list_iter = StreamList::new(reader, false);

        let collected: Vec<String> = list_iter.collect();
        assert_eq!(collected, ["1", "2", "3"]);
    }

    #[test]
    fn stream_list_round_robins_correctly() {
        let mock_data = "1\n2\n3\n";
        let cursor = Cursor::new(mock_data);
        let reader = BufReader::new(cursor);
        let list_iter = StreamList::new_rr(reader);

        let collected: Vec<String> = list_iter.take(6).collect();
        assert_eq!(collected, ["1", "2", "3", "1", "2", "3"]);
    }

    #[test]
    fn stream_list_should_return_nothing_with_an_empty_buffer() {
        let reader = BufReader::new(Cursor::new(""));
        let list_iter = StreamList::new_rr(reader);

        let collected: Vec<String> = list_iter.take(10).collect();
        assert_eq!(collected.len(), 0);
    }
}
