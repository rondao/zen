pub fn decompress_lz5(source: &[u8]) -> Vec<u8> {
    let mut source = source.into_iter();
    let mut output = Vec::new();

    loop {
        let data = *source.next().unwrap();

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
                (data & 0b0001_1100) << 3,
                ((data as usize & 0b0000_0011) << 8) + (*source.next().unwrap() as usize) + 1,
            )
        } else {
            // Normal command = CCCB_BBBB (One Byte)
            //  CCC = Actual command for operation.
            //  B_BBBB = 5 bits for number of bytes to operate.
            (data & 0b1110_0000, (data as usize & 0b0001_1111) + 1)
        };

        let decompressed_data = match command {
            0x00 => direct_copy(&mut source, number_of_bytes),
            0x20 => byte_fill(&mut source, number_of_bytes),
            0x40 => word_fill(&mut source, number_of_bytes),
            0x60 => incrementing_fill(&mut source, number_of_bytes),
            0x80..=0xBF => offset_dictionary(
                &mut source,
                &output,
                number_of_bytes,
                (command & 0b0010_0000) == 0b0010_0000,
            ),
            0xC0..=0xE0 => sliding_dictionary(
                &mut source,
                &output,
                number_of_bytes,
                (command & 0b0010_0000) == 0b0010_0000,
            ),
            _ => panic!("Invalid command for decompression."),
        };

        output.extend(decompressed_data);
    }

    output
}

/// Copy 'number_of_bytes' from 'source' as is. Basically these bytes were not compressed.
fn direct_copy<'s>(source: &mut impl Iterator<Item = &'s u8>, number_of_bytes: usize) -> Vec<u8> {
    source.take(number_of_bytes).copied().collect()
}

/// Copy one byte from 'source' a 'number_of_bytes' times.
fn byte_fill<'s>(source: &mut impl Iterator<Item = &'s u8>, number_of_bytes: usize) -> Vec<u8> {
    source
        .take(1)
        .copied()
        .collect::<Vec<_>>()
        .repeat(number_of_bytes)
}

/// Copy two bytes from 'source' a 'number_of_bytes' times.
/// If 'number_of_bytes' is odd, the low byte will be copied one less time.
fn word_fill<'s>(source: &mut impl Iterator<Item = &'s u8>, number_of_bytes: usize) -> Vec<u8> {
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
fn incrementing_fill<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    number_of_bytes: usize,
) -> Vec<u8> {
    let data = *source.next().unwrap();
    (data..=(data - 1) + number_of_bytes as u8).collect()
}

/// Copy a 'number_of_bytes' from 'output' starting at a two bytes 'offset'.
fn offset_dictionary<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    output: &Vec<u8>,
    number_of_bytes: usize,
    invert: bool,
) -> Vec<u8> {
    let offset = u16::from_le_bytes([*source.next().unwrap(), *source.next().unwrap()]) as usize;
    copy_dictionary(&output[offset..], number_of_bytes, invert)
}

/// Copy a 'number_of_bytes' from 'output' starting at 'output.len()' minus the next byte.
fn sliding_dictionary<'s>(
    source: &mut impl Iterator<Item = &'s u8>,
    output: &Vec<u8>,
    number_of_bytes: usize,
    invert: bool,
) -> Vec<u8> {
    let offset = output.len() - *source.next().unwrap() as usize;
    copy_dictionary(&output[offset..], number_of_bytes, invert)
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
