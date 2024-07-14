use std::{cmp::min, io::{BufReader, Read, Result}};

pub struct ResetableReader<T: Read> {
    buf_reader: BufReader<T>,
    buf: Vec<u8>,
    fill_len: usize,
    read_len: usize,
}

impl<T: Read> ResetableReader<T> {
    pub fn new(inner: T) -> Self {
        return Self {
            buf_reader: BufReader::new(inner),
            buf: Vec::new(),
            fill_len: 0,
            read_len: 0,
        };
    }

    pub fn reset_cursor(&mut self) {
        self.read_len = 0;
    }

    pub fn drop_read_bytes(&mut self) {
        self.buf.drain(0..self.read_len);
        self.fill_len -= self.read_len;
        self.read_len = 0;
    }
}

impl<T: Read> Read for ResetableReader<T> {

    fn read(&mut self, read_buf: &mut [u8]) -> Result<usize> {

        let unread_bytes_in_buf = self.fill_len - self.read_len;

        let bytes_needed_from_stream;
        if read_buf.len() >= unread_bytes_in_buf {
            bytes_needed_from_stream = read_buf.len() - unread_bytes_in_buf;
        } else {
            bytes_needed_from_stream = 0;
        }

        if bytes_needed_from_stream + self.fill_len > self.buf.len() {
            self.buf.resize(self.fill_len + bytes_needed_from_stream, 0);
        }

        let slice = self.buf.as_mut_slice();

        if bytes_needed_from_stream > 0 {

            //Need to read bytes from the reader
            let end = bytes_needed_from_stream + self.fill_len;
            let slice_to_read_into = &mut slice[self.fill_len..end];

            let result = self.buf_reader.read(slice_to_read_into);

            match result {
                Ok(read_len) => self.fill_len += read_len,
                Err(_) => return result,
            }

        }

        //Now, the bytes have already been buffered
        let bytes_available = self.fill_len - self.read_len;
        let len_to_read = min(bytes_available, read_buf.len());

        let slice_to_read_into = &mut read_buf[0..len_to_read];

        slice_to_read_into.copy_from_slice(&mut slice[self.read_len..(self.read_len + len_to_read)]);

        self.read_len += len_to_read;

        let result = Ok(len_to_read);

        return result;
    }
}

#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_no_bytes_needed() {

        let source: [u8; 4] = [45, 78, 34, 99];
        let cursor: Cursor<[u8; 4]> = Cursor::new(source);
        let mut resetable_reader = ResetableReader::new(cursor);

        let mut buf: [u8; 2] = [0, 0];
        
        let result = resetable_reader.read(&mut buf);
        assert!(matches!(result, Ok(2)));
        assert_eq!([45, 78], buf);

        let result = resetable_reader.read(&mut buf);
        assert!(matches!(result, Ok(2)));
        assert_eq!([34, 99], buf);

        resetable_reader.reset_cursor();

        let result = resetable_reader.read(&mut buf);
        assert!(matches!(result, Ok(2)));
        assert_eq!([45, 78], buf);

        resetable_reader.drop_read_bytes();

        let result = resetable_reader.read(&mut buf);
        assert!(matches!(result, Ok(2)));
        assert_eq!([34, 99], buf);

        buf = [0, 0];
        let result = resetable_reader.read(&mut buf);
        assert!(matches!(result, Ok(0)));
        assert_eq!([0, 0], buf);

        resetable_reader.reset_cursor();

        let result = resetable_reader.read(&mut buf);
        assert!(matches!(result, Ok(2)));
        assert_eq!([34, 99], buf);

    }
}
