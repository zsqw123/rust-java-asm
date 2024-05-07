use java_asm_internal::err::{AsmErr, AsmResult};

/// Java MUTF-8 has 2 differences from UTF-8:
/// 1. null characters (U+0000) are encoded as 2 bytes: 0xC0 0x80 (0x00 in UTF-8). So that 
///  MUTF-8 strings never have embedded nulls.
/// 2. characters outside the Basic Multilingual Plane (U+10000 to U+10FFFF) are encoded as 
///  two-times-three-byte (6 bytes) format. (4 bytes in UTF-8).
///
/// ```text
/// UTF-8              MUTF-8
/// // !special case!
/// 0000 0000         1100 0000 1000 0000 (00C0 0080)
///
/// // same, 1byte, 7bit, 0001 -> 007F, next: 0080
/// 0000 0001         0000 0001 (0001)
/// 0___ ____         0___ ____
/// 0111 1111         0111 1111 (007F)
///
/// // same, 2 byte, 11bit, 0080 -> 07FF, next: E0, C0
/// 1100 0010 1000 0000      1100 0010 1000 0000 (0080)
/// 110_ ____ 10__ ____      110_ ____ 10__ ____
/// 1101 1111 1011 1111      1101 1111 1011 1111 (07FF)
///
/// // same, 3 byte, 16bit, 0800 -> FFFF, next: F0, C0, C0
/// 1110 0000 1010 0000 1000 0000      1110 0000 1010 0000 1000 0000 (0800)
/// 1110 ____ 10__ ____ 10__ ____      1110 ____ 10__ ____ 10__ ____
/// 1110 1111 1011 1111 1011 1111      1110 1111 1011 1111 1011 1111 (FFFF)
/// ```
///
/// 4 byte UTF-8 case:
/// ```text
/// UTF8, 4 byte, 21bit, 10000 -> 10FFFF
/// 1111 0000, 1000 0001, 1000 0000, 1000 0000
/// 1111 0___, 10__ ____, 10__ ____, 10__ ____
/// 1111 0100, 1000 1111, 1011 1111, 1011 1111
///
/// MUTF8, 6 byte, 21bit, 10000 -> 10FFFF
/// 1110 1101, 1010 ____, 10__ ____ // 10 byte
///      ~~~~    ~~ UTF16 high surrogate
/// 1110 1101, 1011 ____, 10__ ____ // 10 byte
///      ~~~~    ~~ UTF16 low surrogate
/// ``` 
/// MUTF-8 uses 20 bit to encode 4 byte UTF-8 which needs 21 bit to encode.
/// This logic is actually uses UTF-16 surrogate pair to complete. I have marked them 
/// at the above example. Although UTF-8 need 21 bit, but MUTF-8 only needs to express 
/// (0x10FFFF - 0x010000) = 0xFFFFF, and 0xFFFFF only needs 20 bit. We can simply add 
/// 0x010000 to the 20 bit value to get the 21 bit UTF-8 value.
pub fn mutf8_to_utf8(mutf8: &[u8]) -> AsmResult<Vec<u8>> {
    let len = mutf8.len();
    let mut utf8 = Vec::with_capacity(len);
    let mut current_offset = 0;
    while current_offset < len {
        // 1 byte
        let byte1 = mutf8[current_offset];
        if byte1 >= 0x01 && byte1 <= 0x7F {
            utf8.push(byte1);
            current_offset += 1;
            continue;
        }
        // 2 bytes
        let byte2 = mutf8[current_offset + 1];
        if byte1 >= 0xC0 && byte1 <= 0xDF {
            if byte2 == 0x80 {
                utf8.push(0x00); // special for MUTF-8
            } else {
                utf8.push(byte1);
                utf8.push(byte2);
            }
            current_offset += 2;
            continue;
        }
        let byte3 = mutf8[current_offset + 2];
        // 6 bytes
        if byte1 == 0xED && byte2 >= 0xA0 && byte2 <= 0xAF {
            let byte5 = mutf8[current_offset + 4];
            let byte6 = mutf8[current_offset + 5];
            let code1 = (byte2 as u32 & 0x0F) << 16;
            let code2 = (byte3 as u32 & 0x3F) << 12;
            let code3 = (byte5 as u32 & 0x0F) << 6;
            let code4 = byte6 as u32 & 0x3F;
            let code = 0x0100 + (code1 | code2 | code3 | code4);
            let utf1 = 0xF0 | ((code >> 18) as u8);
            let utf2 = 0x80 | ((code >> 12) as u8);
            let utf3 = 0x80 | ((code >> 6) as u8);
            let utf4 = 0x80 | (code as u8);
            utf8.push(utf1);
            utf8.push(utf2);
            utf8.push(utf3);
            utf8.push(utf4);
            current_offset += 6;
            continue;
        }
        // 3 bytes
        if byte1 >= 0xE0 && byte1 <= 0xEF {
            utf8.push(byte1);
            utf8.push(byte2);
            utf8.push(byte3);
            current_offset += 3;
            continue;
        }
        return AsmErr::ReadMUTF8(format!("unknown MUTF-8 first byte: 0x{:X}", byte1)).e();
    }
    Ok(utf8)
}

pub fn mutf8_to_string(mutf8: &[u8]) -> AsmResult<String> {
    let utf8 = mutf8_to_utf8(mutf8)?;
    match String::from_utf8(utf8) {
        Ok(str) => Ok(str),
        Err(e) => Err(AsmErr::ReadUTF8(e.to_string())),
    }
}

pub fn utf8_to_mutf8(utf8: &[u8]) -> AsmResult<Vec<u8>> {
    let len = utf8.len();
    let mut mutf8 = Vec::with_capacity(len);
    let mut current_offset = 0;
    while current_offset < len {
        let byte1 = utf8[current_offset];
        // 1 byte
        if byte1 == 0x00 { // special for MUTF-8
            mutf8.push(0xC0);
            mutf8.push(0x80);
            current_offset += 1;
            continue;
        }
        if byte1 >= 0x01 && byte1 <= 0x7F {
            mutf8.push(byte1);
            current_offset += 1;
            continue;
        }
        // 2 bytes
        let byte2 = utf8[current_offset + 1];
        if byte1 >= 0xC0 && byte1 <= 0xDF {
            mutf8.push(byte1);
            mutf8.push(byte2);
            current_offset += 2;
            continue;
        }
        let byte3 = utf8[current_offset + 2];
        // 3 bytes
        if byte1 >= 0xE0 && byte1 <= 0xEF {
            mutf8.push(byte1);
            mutf8.push(byte2);
            mutf8.push(byte3);
            current_offset += 3;
            continue;
        }
        // 4 bytes
        if byte1 >= 0xF0 && byte1 <= 0xF4 {
            let byte4 = utf8[current_offset + 3];
            let code = ((byte1 as u32 & 0x07) << 18) | ((byte2 as u32 & 0x3F) << 12) | 
                ((byte3 as u32 & 0x3F) << 6) | (byte4 as u32 & 0x3F);
            let code = code - 0x010000;
            mutf8.push(0xED);
            mutf8.push(0xA0 | ((code >> 16) as u8));
            mutf8.push(0x80 | ((code >> 10) as u8));
            mutf8.push(0xED);
            mutf8.push(0xB0 | ((code >> 6) as u8));
            mutf8.push(0x80 | (code as u8));
            current_offset += 4;
            continue;
        }
        return AsmErr::ReadUTF8(format!("unknown UTF-8 first byte: 0x{:X}", byte1)).e();
    };
    Ok(mutf8)
}
