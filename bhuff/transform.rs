/* Burrows-Wheeler Transform
 * https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform
 */

fn bw_transform<T: Ord+Copy>(data: &Vec<T>) -> Vec<Option<T>> {
    let mut range: Vec<usize> = (0..=data.len()).collect();
    range.sort_by_key(|&i| &data[i..]);
    return range.iter().map(|&i| if i > 0 { Some(data[i-1]) } else { None } ).collect();
}

//this can be done as an iterator to get 'early output'
fn bw_reverse<T: Ord+Copy>(data: &Vec<Option<T>>) -> Vec<T> {
    let mut range: Vec<(Option<T>,usize)> = data.iter().cloned().zip(0..).collect();
    let mut out = Vec::new();
    out.reserve(range.len()-1);
    range.sort_by_key(|&(c,_)| c);
    let mut i = range[0].1;
    for _ in 1..range.len() {
        out.push(range[i].0.unwrap());
        i = range[i].1;
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
