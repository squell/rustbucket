/* Burrows-Wheeler Transform */

fn bw_transform<T: Ord+Copy>(data: &Vec<T>) -> Vec<Option<T>> {
    let mut range: Vec<usize> = (0..=data.len()).collect();
    range.sort_unstable_by_key(|&i| &data[i..]);
    return range.iter().map(|&i| if i > 0 { Some(data[i-1]) } else { None } ).collect();
}

//this can be done as an iterator to get 'early output'
fn bw_reverse<T: Ord+Copy>(data: &Vec<Option<T>>) -> Vec<T> {
    let mut range: Vec<(Option<T>,usize)> = data.iter().cloned().zip(0..).collect();
    let mut out = Vec::new();
    out.reserve(range.len()-1);
    range.sort_unstable_by_key(|&(c,_)| c);
    let mut i = range[0].1;
    for _ in 1..range.len() {
        out.push(range[i].0.unwrap());
        i = range[i].1;
    }
    return out
}
