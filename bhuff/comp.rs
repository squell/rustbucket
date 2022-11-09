use std::io;
use std::io::{Error,ErrorKind,Read};
use std::collections::HashMap;
use std::hash::Hash;

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
        let ((n1,t1),(n2,t2)) = { let mut items = queue.drain(0..=1); (items.next().unwrap(), items.next().unwrap()) }; // !!!!
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

fn io_contents() -> Option<String> {
    let mut str = String::new();
    io::stdin().read_to_string(&mut str).map(|_| str).ok()
}

fn main() -> Result<(),Error> {
    (|| {
        let ftab = frequency_table(io_contents()?.chars())?;
        let prealloc = &mut [BTree::Tip(' '); 300];
        let tree = huffman_tree(&ftab, LocalPlumber(prealloc))?;
        let cmap = codes(&tree);
        for (c, code) in cmap {
            println!("{:?} => {:?}", c, code)
        };
        Some(())
    })
    ().ok_or(Error::new(ErrorKind::Other, "a useless error message"))
}
