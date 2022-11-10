use std::io;
use std::io::{Error,ErrorKind,Read,Write};
use std::collections::HashMap;
use std::hash::Hash;
use std::iter;
use std::env;

//frequencies :: (Ord a) => [a] -> [(a,Int)]
//frequencies = map (\x->(head x, length x)) . group . sort

type FreqTable<T> = Vec<(T,usize)>;

fn frequency_table<T, Iter>(stream: Iter) -> Option<FreqTable<T>>
  where Iter: Iterator<Item=T>,
        T: Ord + Copy
{
    let text = { let mut tmp: Vec<T> = stream.collect(); tmp.sort_unstable(); tmp };

    let mut vec  = Vec::new();
    let mut last = (*text.get(0)?, 1);

    for &c in text.iter().skip(1) {
        if last.0 == c {
            last.1 += 1
        } else {
            vec.push(last);
            last = (c, 1)
        }
    };
    vec.push(last);
    Some(vec)
}

// data Btree a = Tip a | Bin (Btree a) (Btree a)
#[derive(Debug,Clone,Copy)]
enum BTree<'a,T> {
    Tip(T),
    Bin(&'a BTree<'a,T>, &'a BTree<'a,T>)
}

// huffman :: [(a,Int)] -> Btree a
// huffman = go . sortOn fst . map (\(k,n)->(n, Tip k))
//   where go (code1:code2:rest) = go $ insertBy (\x y->fst x `compare` fst y) (code1<+>code2) rest
//         go [(_,tree)] = tree
//         go [] = error "huffman: no data to encode"
// 
//         (n1,t1) <+> (n2,t2) = (n1+n2, Bin t1 t2)

mod plumbing;
use plumbing::{LocalPlumber,Alloc};

fn huffman_tree<'b,F,T>(freq: &FreqTable<T>, mut alloc: F) -> Option<BTree<'b, T>>
  where F: Alloc<'b, BTree<'b,T>>,
        T: Copy
{
    type Pair<'b,T> = (usize, BTree<'b, T>);
    let mut queue: Vec<Pair<T>> = freq.iter().map(|&(chr,n)| (n, BTree::Tip(chr))).collect();

    queue.sort_unstable_by_key(|x|x.0);

    while queue.len() > 1 {
        let ((n1,t1),(n2,t2)) = { let mut items = queue.drain(0..=1); (items.next().unwrap(), items.next().unwrap()) };
        let (u1, new_alloc) = alloc.obtain(t1);
        let (u2, new_alloc) = new_alloc.obtain(t2);
        alloc = new_alloc;
        let el  = (n1+n2, BTree::Bin(u1,u2));
        let pos = queue.binary_search_by_key(&el.0, |x|x.0).unwrap_or_else(|x|x);
        queue.insert(pos, el);
    }

    queue.pop().map(|x| x.1)
}

//codes :: Btree a -> [(a,[Bit])]
//codes (Tip x)   = [ (x, []) ]
//codes (Bin l r) = [ (xl,O:cl) | (xl,cl) <- codes l ] ++
//                  [ (xr,I:cr) | (xr,cr) <- codes r ]

mod bitstring;
use bitstring::Bits;

type BitString = bitstring::RealBits;

fn codes<T: Eq + Hash + Copy> (huftree: &BTree<T>) -> HashMap<T, BitString> {
    fn walk<T: Eq + Hash + Copy>(map: &mut HashMap<T,BitString>, node: &BTree<T>, code: BitString) {
        match node {
               BTree::Tip(c)     => { map.insert(*c, code); },
               BTree::Bin(t1,t2) => { walk(map, t1, code.append(false));
                                      walk(map, t2, code.append(true)); }
        }
    }

    let mut map = HashMap::new();
    walk(&mut map, huftree, Bits::new());
    map
}

fn bits_to_bytes(stream: impl Iterator<Item=bool>) -> impl Iterator<Item=u8> {
    iter::once(0)
    .chain(
        stream.chain(iter::repeat(false).take(7))
              .scan(0, |acc, b| { *acc = ((*acc << 1) + (b as u8)) & 0xFF; Some(*acc) })
    )
    .step_by(8)
    .skip(1)
}

fn bytes_to_bits(stream: impl Iterator<Item=u8>) -> impl Iterator<Item=bool> {
    stream.flat_map(|x|bitstring::RealBits::from_u8(x))
}

mod transform;
use transform::{transform,untransform};

//not done: correct generation of huffman trees in case the input only has one byte
fn emit_hufftree(input: impl Iterator<Item=u8>) -> Option<()> {
    let (_, input) = transform(input);
    let ftab = frequency_table(input.iter().cloned())?;

    let prealloc = &mut [BTree::Tip(0); 510]; // 256 Tip + 255 Bin - 1 node in local variable
    let tree = huffman_tree(&ftab, LocalPlumber(prealloc))?;
    Some(println!("{:?}", &tree))
}

fn compress(tree: &BTree<u8>, input: impl Iterator<Item=u8>) -> Option<()> {
    let (bw_pos, input) = transform(input);

    let cmap = codes(&tree);
    let compressed_bits = input.iter().flat_map(|x| *cmap.get(&x).unwrap());

    let mut bin_out = io::BufWriter::new(io::stdout());
    bin_out.write_all(&input.len().to_ne_bytes()).ok()?;
    bin_out.write_all(&bw_pos.to_ne_bytes()).ok()?;
    for byte in bits_to_bytes(compressed_bits) {
        bin_out.write_all(&[byte]).ok()?;
    };

    Some(())
}

fn get_usize(input: &mut impl Iterator<Item=u8>) -> usize {
    let mut data = [0u8; 8];
    for i in 0..=7 {
        data[i] = input.next().unwrap()
    }
    return usize::from_ne_bytes(data)
}

fn decompress(root: &BTree<u8>, mut input: impl Iterator<Item=u8>) -> Option<()> {
    debug_assert!(if let BTree::Tip(_) = root { false } else { true });

    let inp_len = get_usize(&mut input);
    let bw_pos  = get_usize(&mut input);
    let mut out = Vec::<u8>::new();
    out.reserve(inp_len+1);

    let mut bin_out = io::BufWriter::new(io::stdout());
    let mut node = &root;
    for bit in bytes_to_bits(input) {
        loop {
            match *node {
                BTree::Tip(byte)  => { out.push(*byte); node = &root },
                BTree::Bin(t1,t2) => { node = if !bit { t1 } else { t2 }; break },
            }
        }
    };

    out.truncate(inp_len);
    bin_out.write_all(&untransform(bw_pos, out.iter().cloned())).ok()?;
    Some(())
}

static HUFFTREE : &BTree<u8> = { 
    use BTree::{Tip,Bin}; 
    include!("hufftree.in") 
};

fn main() -> Result<(),Error> {
    (|| {
        let args: Vec<String> = env::args().collect();
        let input = io::BufReader::new(io::stdin()).bytes().map(|x|x.unwrap());
        match args.get(1).map(|x|x.as_str()) {
            Some("-train") => emit_hufftree(input),
            Some("-d")     => decompress(HUFFTREE, input),
            None           => compress(HUFFTREE, input),
            _              => None,
        }
    })
    ().ok_or(Error::new(ErrorKind::Other, "a useless error message"))
}
