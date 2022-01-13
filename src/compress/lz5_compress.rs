/// Compress 'source' with Lz5 algorithm.
pub fn compress(source: &[u8]) -> Vec<u8> {
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
    if (source_address - window_offset) <= 0xFF {
        let (mut compression, bytes) = create_command_data(0xC0, window_size);
        compression.push((source_address - window_offset) as u8);
        (compression, bytes)
    } else {
        let (mut compression, bytes) = create_command_data(0x80, window_size);
        compression.push((window_offset & 0b0000_0000_1111_1111) as u8);
        compression.push(((window_offset & 0b1111_1111_0000_0000) >> 8) as u8);
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
