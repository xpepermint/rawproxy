use async_std::prelude::*;
use async_std::io::{Read, Write};
use crate::{ErrorKind};

/// Extracts a specific header from the provided protocol headers.
pub fn read_header(lines: &Vec<String>, name: &str) -> Option<String> {
    let needle = format!("{}: ", name);
    let line = lines.into_iter().find(|line| line.starts_with(&needle) );
    match line {
        None => None,
        Some(line) => {
            let mut splitter = line.splitn(2, ' ');
            splitter.next().unwrap();
            let value = splitter.next().unwrap();
            Some(value.to_string())
        },
    }
}

/// Modifies the provided protocol headers by inserting/replaces a header.
pub fn write_header(lines: &mut Vec<String>, name: &str, value: &str) {
    let needle = format!("{}: ", name);
    let mut found = false;
    for line in lines.iter_mut() {
        if line.starts_with(&needle) {
            *line = format!("{}: {}", name, value).to_string();
            found = true;
            break;
        }
    }
    if !found {
        lines.push(
            format!("{}: {}", name, value),
        );
    }
}

/// Parses string into number.
pub fn string_to_usize(txt: &str) -> Option<usize> {
    match txt.parse::<usize>() {
        Ok(length) => Some(length),
        Err(_) => None,
    }
}

/// Returns true if the provided bytes array holds a specific sequance of bytes.
pub fn vec_has_sequence(bytes: &[u8], needle: &[u8]) -> bool {
    let mut foun = 0;
    let nsize = needle.len();
    for byte in bytes.into_iter() {
        if foun == nsize {
            return true;
        } else if *byte == needle[foun] {
            foun += 1;
        } else {
            foun = 0;
        }
    }
    false
}

/// Parses HTTP protocol headers into `lines`. What's left in the stream represents request body.
/// Limit in number of bytes for the protocol headers can be applied.
pub async fn read_protocol<I>(input: &mut I, lines: &mut Vec<String>, limit: Option<usize>) -> Result<(), ErrorKind>
    where
    I: Read + Unpin,
{
    let mut buffer: Vec<u8> = Vec::new();
    let mut stage = 0; // 1 = first \r, 2 = first \n, 3 = second \r, 4 = second \n
    let mut count = 0;

    loop {
        let mut byte = [0u8];
        let size = match input.read(&mut byte).await {
            Ok(size) => size,
            Err(_) => return Err(ErrorKind::ReadFailed),
        };
        let byte = byte[0];
        count += 1;

        if size == 0 {
            break;
        } else if limit.is_some() && Some(count) > limit {
            return Err(ErrorKind::SizeLimitExceeded(limit.unwrap()));
        } else if byte == 0x0D { // char \r
            if stage == 0 || stage == 2 {
                stage += 1;
            } else {
                return Err(ErrorKind::InvalidData);
            }
        } else if byte == 0x0A { // char \n
            if stage == 1 || stage == 3 {
                let line = match String::from_utf8(buffer.to_vec()) {
                    Ok(line) => line,
                    Err(_) => return Err(ErrorKind::InvalidData),
                };
                lines.push(line);
                buffer.clear();
                stage += 1;
            } else {
                return Err(ErrorKind::InvalidData);
            }
        } else { // arbitrary char
            buffer.push(byte);
            stage = 0;
        }

        if stage == 4 { // next byte belongs to body
            break;
        }
    }

    lines.push("".to_string()); // EOF
    Ok(())
}

/// Streams body data from input to output.
pub async fn forward_body<I, O>(mut input: &mut I, mut output: &mut O, lines: &Vec<String>, limit: Option<usize>) -> Result<(), ErrorKind>
    where
    I: Write + Read + Unpin,
    O: Write + Read + Unpin,
{
    let encoding = match read_header(&lines, "Transfer-Encoding") {
        Some(encoding) => encoding,
        None => String::from("identity"),
    };

    if encoding.contains("chunked") {
        forward_chunked_body(&mut input, &mut output, limit).await
    } else {
        let length = match read_header(&lines, "Content-Length") {
            Some(encoding) => match string_to_usize(&encoding) {
                Some(encoding) => encoding,
                None => return Err(ErrorKind::InvalidHeader(String::from("Content-Length"))),
            },
            None => 0,
        };
        if length == 0 {
            return Ok(());
        } else if limit.is_some() && length > limit.unwrap() {
            return Err(ErrorKind::SizeLimitExceeded(limit.unwrap()));
        }
        forward_sized_body(&mut input, &mut output, length).await
    }
}

/// Streams chunk body data from input to output. Body length is unknown but
/// we can provide size limit.
/// 
/// The method searches for `0\r\n\r\n` which indicates the end of an input
/// stream. If the limit is set and the body exceeds the allowed size then the
/// forwarding will be stopped with and error.
pub async fn forward_chunked_body<I, O>(input: &mut I, output: &mut O, limit: Option<usize>) -> Result<(), ErrorKind>
    where
    I: Write + Read + Unpin,
    O: Write + Read + Unpin,
{
    let mut buffer: Vec<u8> = Vec::new();
    let mut count = 0;
    loop {
        let mut bytes = [0u8; 1024];
        let size = match input.read(&mut bytes).await {
            Ok(size) => size,
            Err(_) => return Err(ErrorKind::ReadFailed),
        };
        let mut bytes = &mut bytes[0..size].to_vec();
        count += size;

        if limit.is_some() && count >= limit.unwrap() {
            return Err(ErrorKind::SizeLimitExceeded(limit.unwrap()));
        }

        match output.write(&bytes).await {
            Ok(source) => source,
            Err(_) => return Err(ErrorKind::WriteFailed),
        };
        match output.flush().await {
            Ok(_) => (),
            Err(_) => return Err(ErrorKind::WriteFailed),
        };

        buffer.append(&mut bytes);
        buffer = (&buffer[buffer.len()-5..]).to_vec();
        if vec_has_sequence(&buffer, &[48, 13, 10, 13, 10]) { // last chunk
            break;
        }
        buffer = (&buffer[buffer.len()-5..]).to_vec();
    }
    Ok(())
}

/// Streams body data of known size from input to output. An exact body length
/// (e.g. `Content-Length` header) must be provided for this transfer type.
/// 
/// The method expects that the input holds only body data. This means that we
/// have to read input protocol headers before we call this method.
pub async fn forward_sized_body<I, O>(input: &mut I, output: &mut O, length: usize) -> Result<(), ErrorKind>
    where
    I: Read + Unpin + ?Sized,
    O: Write + Unpin + ?Sized,
{
    let mut count = 0;
    loop {
        let mut bytes = [0u8; 1024];
        let size = match input.read(&mut bytes).await {
            Ok(size) => size,
            Err(_) => return Err(ErrorKind::ReadFailed),
        };
        let bytes = &mut bytes[0..size].to_vec();
        count += size;

        match output.write(&bytes).await {
            Ok(size) => size,
            Err(_) => return Err(ErrorKind::WriteFailed),
        };
        match output.flush().await {
            Ok(_) => (),
            Err(_) => return Err(ErrorKind::WriteFailed),
        };

        if size == 0 || count == length {
            break;
        } else if count > length {
            return Err(ErrorKind::SizeLimitExceeded(length));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[async_std::test]
    async fn read_header_value() {
        assert_eq!(read_header(&vec![
            "GET / HTTP/1.1".to_string(),
            "Host: google.com".to_string(),
            "Connection: close".to_string(),
        ], "Host"), Some("google.com".to_string()));
    }

    #[async_std::test]
    async fn checks_vector_has_sequence() {
        assert!(vec_has_sequence(&[1, 4, 6, 10, 21, 5, 150], &[10, 21, 5]));
        assert!(!vec_has_sequence(&[1, 4, 6, 10, 21, 5, 150], &[10, 5]));
    }
}
