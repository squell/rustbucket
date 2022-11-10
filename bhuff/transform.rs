/* Burrows-Wheeler Transform
 * https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform
 */

// these two functions can probably work in-place
fn bw_transform<T: Ord+Copy>(data: &Vec<T>) -> (usize, Vec<T>) {
    let mut range: Vec<usize> = (0..=data.len()).collect();
    range.sort_by_key(|&i| &data[i..]);
    let startpos = range.iter().position(|&i| i==0).unwrap();
    let vec = range.iter().filter_map(|&i| if i > 0 { Some(data[i-1]) } else { None } ).collect();
    return (startpos, vec);
}

//this can be done as an iterator to get 'early output'
fn bw_reverse<T: Ord+Copy>((startpos,data): &(usize,Vec<T>)) -> Vec<T> {
    let mut range: Vec<(T,usize)> = data.iter().cloned().zip((0..=data.len()).filter(|&n|n!=*startpos)).collect();
    let mut out = Vec::new();
    out.reserve(range.len());
    range.sort_by_key(|&(c,_)| c);
    let mut i = *startpos;
    while i > 0 {
        out.push(range[i-1].0);
        i = range[i-1].1;
    }
    return out
}

/* The obligatory 'move to front' transformation:
 * https://en.wikipedia.org/wiki/Move-to-front_transform
 */

fn move_to_front(data: &mut Vec<u8>) {
    let mut alphabet: [u8; 256] = [0; 256];
    for i in 0..=255 {
        alphabet[i] = i as u8
    }
    for byte in data.iter_mut() {
        let index = alphabet.iter().position(|&c|c==*byte).unwrap();
        alphabet.copy_within(0..index, 1);
        alphabet[0] = *byte;
        *byte = index as u8;
    }
}

fn unmove_to_front(data: &mut Vec<u8>) {
    let mut alphabet: [u8; 256] = [0; 256];
    for i in 0..=255 {
        alphabet[i] = i as u8
    }
    for index in data.iter_mut() {
        let pos = *index as usize;
        let c = alphabet[pos];
        alphabet.copy_within(0..pos, 1);
        alphabet[0] = c;
        *index = c;
    }
}

/* combined BW & MTF transformation */
pub fn transform(input: impl Iterator<Item=u8>) -> (usize, Vec<u8>) {
    use transform::*;
    let (startpos, mut vec) = bw_transform(&input.collect());
    move_to_front(&mut vec);
    (startpos, vec)
}

pub fn untransform(startpos: usize, input: impl Iterator<Item=u8>) -> Vec<u8> {
    use transform::*;
    let mut vec = input.collect();
    unmove_to_front(&mut vec);
    bw_reverse(&(startpos, vec))
}
