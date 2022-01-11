use std::{error::Error, fmt};

/// Decompress 'source' with lz5 algorithm.
/// Reference: http://patrickjohnston.org/bank/80#fB0FF
pub fn decompress_lz5(source: &[u8]) -> Result<Vec<u8>, Lz5Error> {
    let mut source = source.into_iter();
    let mut output = Vec::new();

    loop {
        let data = *source.next().ok_or(Lz5Error)?;

        // End of file data = 0xFF
        if data == 0b1111_1111 {
            break;
        }

        let (command, number_of_bytes) = if (data & 0b1110_0000) == 0b1110_0000 {
            // Extended command = 111C_CCBB BBBB_BBBB (Two Bytes)
            //  First three bits as 1 means an extended command.
            //  CCC = Actual command for operation.
            //  BB_BBBB_BBBB = 10 bits for number of bytes to operate.
            (
                // Command:
                (data & 0b0001_1100) << 3,
                // Number of Bytes:
                ((data as usize & 0b0000_0011) << 8)
                    + (*source.next().ok_or(Lz5Error)? as usize)
                    + 1,
            )
        } else {
            // Normal command = CCCB_BBBB (One Byte)
            //  CCC = Actual command for operation.
            //  B_BBBB = 5 bits for number of bytes to operate.
            (data & 0b1110_0000, (data as usize & 0b0001_1111) + 1)
        };

        let decompressed_data = match command {
            0x00 => decompress_direct_copy(&mut source, number_of_bytes),
            0x20 => decompress_byte_fill(&mut source, number_of_bytes),
            0x40 => decompress_word_fill(&mut source, number_of_bytes),
            0x60 => decompress_incrementing_fill(&mut source, number_of_bytes)?,
            0x80..=0xBF => decompress_offset_dictionary(
                &mut source,
                &output,
                number_of_bytes,
                (command & 0b0010_0000) == 0b0010_0000,
            )?,
            0xC0..=0xE0 => decompress_sliding_dictionary(
                &mut source,
                &output,
                number_of_bytes,
                (command & 0b0010_0000) == 0b0010_0000,
            )?,
            _ => return Err(Lz5Error),
        };

        output.extend(decompressed_data);
    }

    Ok(output)
}

/// Copy 'number_of_bytes' from 'source' as is. Basically these bytes were not compressed.
fn decompress_direct_copy<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    number_of_bytes: usize,
) -> Vec<u8> {
    source.take(number_of_bytes).copied().collect()
}

/// Copy one byte from 'source' a 'number_of_bytes' times.
fn decompress_byte_fill<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    number_of_bytes: usize,
) -> Vec<u8> {
    source
        .take(1)
        .copied()
        .collect::<Vec<_>>()
        .repeat(number_of_bytes)
}

/// Copy two bytes from 'source' a 'number_of_bytes' times.
/// If 'number_of_bytes' is odd, the low byte will be copied one less time.
fn decompress_word_fill<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    number_of_bytes: usize,
) -> Vec<u8> {
    source
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .cycle()
        .take(number_of_bytes)
        .copied()
        .collect()
}

/// Copy one byte from 'source' a 'number_of_bytes' times, but incrementing it's value each time.
fn decompress_incrementing_fill<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    number_of_bytes: usize,
) -> Result<Vec<u8>, Lz5Error> {
    let data = *source.next().ok_or(Lz5Error)?;
    Ok((0..=number_of_bytes - 1)
        .map(|value| value.wrapping_add(data as usize) as u8)
        .collect())
}

/// Copy a 'number_of_bytes' from 'output' starting at a two bytes 'offset'.
fn decompress_offset_dictionary<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    output: &Vec<u8>,
    number_of_bytes: usize,
    invert: bool,
) -> Result<Vec<u8>, Lz5Error> {
    let offset = u16::from_le_bytes([
        *source.next().ok_or(Lz5Error)?,
        *source.next().ok_or(Lz5Error)?,
    ]) as usize;

    if offset <= output.len() {
        Ok(copy_dictionary(&output[offset..], number_of_bytes, invert))
    } else {
        Err(Lz5Error)
    }
}

/// Copy a 'number_of_bytes' from 'output' starting at 'output.len()' minus the next byte.
fn decompress_sliding_dictionary<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    output: &Vec<u8>,
    number_of_bytes: usize,
    invert: bool,
) -> Result<Vec<u8>, Lz5Error> {
    let offset = output
        .len()
        .checked_sub(*source.next().ok_or(Lz5Error)? as usize)
        .ok_or(Lz5Error)?;
    Ok(copy_dictionary(&output[offset..], number_of_bytes, invert))
}

/// Copy a 'number_of_bytes' from 'offseted_output', inverting all bits if required.
fn copy_dictionary(offseted_output: &[u8], number_of_bytes: usize, invert: bool) -> Vec<u8> {
    let decompressed_data = offseted_output
        .into_iter()
        .cycle()
        .take(number_of_bytes)
        .copied();
    if invert {
        decompressed_data.map(|value| !value).collect()
    } else {
        decompressed_data.collect()
    }
}

/// Compress 'source' with lz5 algorithm.
pub fn compress_lz5(source: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut direct_copy_address = None;

    let mut address = 0;
    while address < source.len() {
        let mut command = 0x00; // Direct copy.
        let mut bytes_compressed = 1;

        let byte_fill_size = count_byte_fill(&source[address..]);
        let word_fill_size = count_word_fill(&source[address..]);
        let incrementing_fill_size = count_incrementing_fill(&source[address..]);
        let (sliding_size, window_address) =
            count_sliding_dictonary(&source[address..], &source[..address]);

        for (compressed_size, compression_command) in [
            (byte_fill_size, 0x20),
            (word_fill_size, 0x40),
            (incrementing_fill_size, 0x60),
            (sliding_size, 0x80),
        ] {
            if bytes_compressed < compressed_size {
                bytes_compressed = compressed_size;
                command = compression_command;
            }
        }

        if command == 0x00 {
            if direct_copy_address.is_none() {
                direct_copy_address = Some(address)
            }
            address += 1;
        } else {
            // Before processing a command, direct copy anything we have behind.
            if let Some(copy_address) = direct_copy_address {
                let number_of_bytes = address - copy_address;
                if number_of_bytes > 0 {
                    output.extend(compress_direct_copy(
                        &source[copy_address..],
                        number_of_bytes,
                    ));
                }
                direct_copy_address = None;
            }

            let (compressed_data, compressed_size) = match command {
                0x20 => compress_byte_fill(&source[address..], bytes_compressed),
                0x40 => compress_word_fill(&source[address..], bytes_compressed),
                0x60 => compress_incrementing_fill(&source[address..], bytes_compressed),
                _ => compress_offset_dictionary(address, window_address, sliding_size),
            };
            output.extend(compressed_data);
            address += compressed_size;
        }

        // If we reached maximum direct copy size, then we have to copy.
        if let Some(copy_address) = direct_copy_address {
            let number_of_bytes = address - copy_address;
            if number_of_bytes == 0b11_1111_1111 {
                output.extend(compress_direct_copy(
                    &source[copy_address..],
                    number_of_bytes,
                ));
                direct_copy_address = None;
            }
        }
    }

    // Copy any left over Direct Copy data.
    if let Some(copy_address) = direct_copy_address {
        let number_of_bytes = address - copy_address;
        if number_of_bytes > 0 {
            output.extend(compress_direct_copy(
                &source[copy_address..],
                number_of_bytes,
            ));
        }
    }

    output.push(0xFF);
    output
}

/// Count first byte from 'source' as many times as possible.
fn count_byte_fill(source: &[u8]) -> usize {
    let byte = source[0];
    let mut number_of_bytes = 1;

    while Some(&byte) == source.get(number_of_bytes) {
        number_of_bytes += 1;
    }

    // It takes two bytes for the command + data. If it doesn't compress at least two, it's not worth it.
    if number_of_bytes > 2 {
        number_of_bytes
    } else {
        0
    }
}

/// Count two bytes from 'source' as many times as possible.
fn count_word_fill(source: &[u8]) -> usize {
    let word = if source.len() > 1 {
        &source[0..2]
    } else {
        &source[0..1]
    };

    let mut number_of_bytes = 2;

    while Some(&word[0]) == source.get(number_of_bytes) {
        number_of_bytes += 1;
        if Some(&word[1]) == source.get(number_of_bytes) {
            number_of_bytes += 1;
        } else {
            break;
        }
    }

    // It takes three bytes for the command + data. If it doesn't compress at least three, it's not worth it.
    if number_of_bytes > 3 {
        number_of_bytes
    } else {
        0
    }
}

/// Count a incrementing sequence of the first byte from 'source'.
fn count_incrementing_fill(source: &[u8]) -> usize {
    let byte = source[0];
    let mut number_of_bytes: usize = 1;

    while Some(&byte.wrapping_add(number_of_bytes as u8)) == source.get(number_of_bytes) {
        number_of_bytes += 1;
    }

    // It takes two bytes for the command + data. If it doesn't compress at least two, it's not worth it.
    if number_of_bytes > 2 {
        number_of_bytes
    } else {
        0
    }
}

/// Count the biggest prefix sequence from 'source' existing in 'sliding_window'.
fn count_sliding_dictonary(source: &[u8], sliding_window: &[u8]) -> (usize, usize) {
    let mut number_of_bytes = 0;
    let mut window_address = 0;

    for address in 0..sliding_window.len() {
        for (bytes, window_byte) in sliding_window[address..].iter().chain(source).enumerate() {
            if let Some(&source_byte) = source.get(bytes) {
                if source_byte == *window_byte {
                    if number_of_bytes <= bytes {
                        number_of_bytes = bytes;
                        window_address = address;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    // 'number_of_bytes' have the last index. We add one for total number of bytes.
    number_of_bytes += 1;

    // It takes two bytes for the command + data. If it doesn't compress at least two, it's not worth it.
    if number_of_bytes > 2 {
        (number_of_bytes, window_address)
    } else {
        (0, 0)
    }
}

/// Return 'bytes' from 'source'.
fn compress_direct_copy(source: &[u8], number_of_bytes: usize) -> Vec<u8> {
    let (mut compression, bytes) = create_command_data(0x00, number_of_bytes);
    compression.extend(source[..bytes].iter());

    compression
}

/// Return command followed by 'source[0]' byte to fill.
fn compress_byte_fill(source: &[u8], number_of_bytes: usize) -> (Vec<u8>, usize) {
    let (mut compression, bytes) = create_command_data(0x20, number_of_bytes);
    compression.push(source[0]);

    (compression, bytes)
}

/// Return command followed by 'source[0], source[1]' word to fill.
fn compress_word_fill(source: &[u8], number_of_bytes: usize) -> (Vec<u8>, usize) {
    let (mut compression, bytes) = create_command_data(0x40, number_of_bytes);
    compression.push(source[0]);
    compression.push(source[1]);

    (compression, bytes)
}

/// Return command followed by 'source[0]' byte to incremental fill.
fn compress_incrementing_fill(source: &[u8], number_of_bytes: usize) -> (Vec<u8>, usize) {
    let (mut compression, bytes) = create_command_data(0x60, number_of_bytes);
    compression.push(source[0]);

    (compression, bytes)
}

/// Return command followed by offset to dictionary.
/// This ignores the Invert functionality supported by Lz5.
fn compress_offset_dictionary(
    source_address: usize,
    window_offset: usize,
    window_size: usize,
) -> (Vec<u8>, usize) {
    if window_size <= 0xFF {
        let (mut compression, bytes) = create_command_data(0xC0, window_size);
        compression.push((source_address - window_offset) as u8);
        (compression, bytes)
    } else {
        let (mut compression, bytes) = create_command_data(0x80, window_size);
        compression.push((window_offset & 0x1111_1111_0000_0000 >> 8) as u8);
        compression.push((window_offset & 0x0000_0000_1111_1111) as u8);
        (compression, bytes)
    }
}

/// Return a one or two byte long 'command + number_of_bytes'.
fn create_command_data(command: usize, number_of_bytes: usize) -> (Vec<u8>, usize) {
    let total_number_of_bytes = number_of_bytes.min(0b11_1111_1111);
    let number_of_bytes = total_number_of_bytes - 1; // One is add at decompression by default.

    // Extended command = 111C_CCBB BBBB_BBBB (Two Bytes)
    if number_of_bytes > 0b0001_1111 {
        (
            vec![
                (0b1110_0000 + (command >> 3) + (number_of_bytes >> 8)) as u8,
                (number_of_bytes & 0b00_1111_1111) as u8,
            ],
            total_number_of_bytes,
        )
    } else {
        // Normal command = CCCB_BBBB (One Byte)
        (
            vec![(command + number_of_bytes) as u8],
            total_number_of_bytes,
        )
    }
}

pub struct Lz5Error;

impl Error for Lz5Error {}

impl fmt::Display for Lz5Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to decompress data using LZ5 algorithm.")
    }
}

impl fmt::Debug for Lz5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to decompress data using LZ5 algorithm.")
    }
}
