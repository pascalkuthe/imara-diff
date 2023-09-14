use crate::intern::Token;

pub fn common_prefix(file1: &[Token], file2: &[Token]) -> u32 {
    let mut off = 0;
    for (token1, token2) in file1.iter().zip(file2) {
        if token1 != token2 {
            break;
        }
        off += 1;
    }
    off
}

pub fn common_postfix(file1: &[Token], file2: &[Token]) -> u32 {
    let mut off = 0;
    for (token1, token2) in file1.iter().rev().zip(file2.iter().rev()) {
        if token1 != token2 {
            break;
        }
        off += 1;
    }
    off
}

pub fn common_edges(file1: &[Token], file2: &[Token]) -> (u32, u32) {
    let prefix = common_prefix(file1, file2);
    let postfix = common_postfix(&file1[prefix as usize..], &file2[prefix as usize..]);
    (prefix, postfix)
}

pub fn strip_common_prefix(file1: &mut &[Token], file2: &mut &[Token]) -> u32 {
    let off = common_prefix(file1, file2);
    *file1 = &file1[off as usize..];
    *file2 = &file2[off as usize..];
    off
}

pub fn strip_common_postfix(file1: &mut &[Token], file2: &mut &[Token]) -> u32 {
    let off = common_postfix(file1, file2);
    *file1 = &file1[..file1.len() - off as usize];
    *file2 = &file2[..file2.len() - off as usize];
    off
}

pub fn sqrt(val: usize) -> u32 {
    let nbits = (usize::BITS - val.leading_zeros()) / 2;
    1 << nbits
}
