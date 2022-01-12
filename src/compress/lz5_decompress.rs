use super::Lz5Error;

/// Decompress 'source' with Lz5 algorithm.
/// Reference: http://patrickjohnston.org/bank/80#fB0FF
pub fn decompress(source: &[u8]) -> Result<Vec<u8>, Lz5Error> {
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
