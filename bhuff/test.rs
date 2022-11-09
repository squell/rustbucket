use std::io;
use std::io::Read;
use std::collections::HashMap;
use std::fmt::Debug;

fn io_contents() -> Option<String> {
    let mut str = String::new();
    match io::stdin().read_to_string(&mut str) {
      Ok(_) => Some(str),
      _     => None,
    }
}

//frequencies :: (Ord a) => [a] -> [(a,Int)]
//frequencies = map (\x->(head x, length x)) . group . sort

type FreqTable = Vec<(char,u32)>;

fn frequency_table(s: String) -> FreqTable {
    let text: Vec<char> = { let mut tmp: Vec<char> = s.chars().collect(); tmp.sort_unstable(); tmp };

    type FreqPair = (char, u32);
    let mut vec: Vec<FreqPair> = Vec::new();
    let mut last: FreqPair = (text[0], 1); // this can fail

    for &c in text.iter().skip(1) {
        if last.0 == c {
            last.1 += 1
        } else {
            vec.push(last);
            last = (c, 1)
        }
    };
    vec.push(last);
    vec
}

// data Btree a = Tip a | Bin (Btree a) (Btree a)
#[derive(Debug,Clone,Copy)]
enum BTree<'a,T> {
    Tip(T), Bin(&'a BTree<'a,T>, &'a BTree<'a,T>)
}

// huffman :: [(a,Int)] -> Btree a
// huffman = go . sortOn fst . map (\(k,n)->(n, Tip k))
//   where go (code1:code2:rest) = go $ insertBy (\x y->fst x `compare` fst y) (code1<+>code2) rest
//         go [(_,tree)] = tree
//         go [] = error "huffman: no data to encode"
// 
//         (n1,t1) <+> (n2,t2) = (n1+n2, Bin t1 t2)

trait Alloc<'b,T> {
    fn obtain(self, x:T) -> (&'b T, Self);
}

//fn huffman_tree<'b>(freq: &FreqTable, alloc: fn()->&'b mut BTree<'b, char>) -> Option<BTree<'b, char>>
fn huffman_tree<'b,F>(freq: &FreqTable, mut alloc: F) -> Option<BTree<'b, char>> 
  //where F: FnMut() -> &'b mut BTree<'b, char>
  //where F: FnMut(BTree<'b,char>) -> &'b BTree<'b, char>
  where F: Alloc<'b,BTree<'b,char>>
{
    let mut queue: Vec<(u32, BTree<'b, char>)> = freq.iter().map(|&(chr,n)| (n, BTree::Tip(chr))).collect();
    // brrr                                ^^^                  ^^^^

    /* interesting mistakes
    fn my_first<'a, A,B>(x: &'a (A,B)) -> &'a A {
        &x.0
    }
    //let first: for<'a> fn(&'a (u32,BTree<char>)) -> &'a u32 = |x| my_first(x);
    //let first: fn(&(u32,BTree<char>)) -> u32 = |x| *my_first(x);
    //let first: fn(&(u32,BTree<char>)) -> u32 = |x| x.0;
    */
    fn first<'a, A: Copy,B>(x: &'a (A,B)) -> A {
        x.0
    }

    queue.sort_unstable_by_key(first);
    /*
    let insert = |el: (u32,BTree<char>)| { let pos = queue.binary_search_by_key(&el.0,first).unwrap_or_else(|x|x); queue.insert(pos, el) };
    let insert = |el: (u32,BTree<'b,char>)| {
                     let pos = queue.binary_search_by_key(&el.0,first).unwrap_or_else(|x|x);
                     queue.insert(pos, el)
                 };
    */
    fn insert<'a>(queue: &mut Vec<(u32,BTree<'a, char>)>, el: (u32,BTree<'a, char>)) {
        let pos = queue.binary_search_by_key(&el.0,first).unwrap_or_else(|x|x);
        queue.insert(pos, el);
    };
    type _Pair<'a> = (u32, BTree<'a, char>);

    while queue.len() > 1 {
        let ((n1,t1),(n2,t2)) = { let mut items = queue.drain(0..=1); (items.next().unwrap(), items.next().unwrap()) }; // !!!!
        //let (u1,u2) = (alloc(), alloc());
        //*u1 = t1;
        //*u2 = t2;
        //let (u1,u2) = (alloc(t1), alloc(t2));
        //let (u1,u2) = (alloc.obtain(t1), alloc.obtain(t2));
        let (u1,u2) = {
            let (e1, next) = alloc.obtain(t1);
            let (e2, next) = next .obtain(t2);
            alloc = next;
            (e1,e2)
        };
        insert (&mut queue, (n1+n2, BTree::Bin(u1,u2))); /* al die moeite van de lokale closure om niets :( */
    }
    queue.pop().map(|x| x.1)
}

//codes :: Btree a -> [(a,[Bit])]
//codes (Tip x)   = [ (x, []) ]
//codes (Bin l r) = [ (xl,O:cl) | (xl,cl) <- codes l ] ++
//                  [ (xr,I:cr) | (xr,cr) <- codes r ]

type BitString = String;

/*
impl<'a> Iterator for BinIterator<'a> {
    type Item = (char,BitString);

    fn next(&mut self) -> Option<Self::Item> {
        if self.prefix {
            self.child.next().map(
                |(c,mut s)| (c, { s.insert(0,'1'); s }))
        } else {
            self.child.next().map_or_else(
                || { self.prefix = true; swap(self.child, self.sibling); self.next() },
                        |(c, mut s)| Some((c, { s.insert(0,'0'); s }))
            )
        }
    }
}
*/

fn codes (huftree: &BTree<char>) -> HashMap<char, BitString> {
    //let mut map: HashMap<i32, i32> = [(1,2)].iter().map(|x|*x).collect(); GRRR
    let mut map;
    match *huftree {
        BTree::Tip(c) =>     { map = HashMap::new(); map.insert(c, BitString::new()); },
        BTree::Bin(t1,t2) => { map = codes(t1); let mut map2 = codes(t2);
                               map .values_mut().for_each(|s| s.insert(0, '0'));
                               map2.values_mut().for_each(|s| s.insert(0, '1'));
                               map.extend(map2); }
    };
    map
}

/* dirty hack
fn bad_plumber<'a,'b> () -> &'a mut BTree<'b, char> {
    let tmp = Box::new(BTree::Tip(' '));
    Box::leak(tmp)
}

fn plumber<'a>(store: &'a mut Vec<BTree<'a,char>>) -> &'a mut BTree<'a, char> {
    store.push(BTree::Tip(' '));
    store.last_mut().unwrap()
}

struct MyAlloc<T> {
    storage: Vec<T>
}

impl<'a> MyAlloc<BTree<'a, char>> {
    pub fn come_get_some(&'a mut self) -> &'a mut BTree<'a, char> {
        self.storage.push(BTree::Tip(' '));
        self.storage.last_mut().unwrap()
    }

    pub fn new() -> Self {
        MyAlloc { storage: Vec::new() }
    }
}
*/

struct BadPlumber { }

impl<'b,T> Alloc<'b,T> for BadPlumber where T: Debug {
    fn obtain(self, x:T) -> (&'b T, Self) {
        let tmp = Box::new(x);
        (Box::leak(tmp), self)
    }
}

struct LocalPlumber<'b,T> {
    store: &'b mut [T]
}

impl<'b,T> Alloc<'b,T> for LocalPlumber<'b,T> {
    fn obtain(self, x:T) -> (&'b T, Self) {
        let (cell,rest) = self.store.split_at_mut(1);
        let elem = &mut cell[0];
        *elem = x;
        return (elem, LocalPlumber { store: rest })
    }
}

/*
impl<'b,T> Alloc<'b,T> for fn()->&'b mut T {  // *grin*
    fn obtain(self, x:T) -> (&'b T, Self) {
        loop{}
    }
}
*/

// using rust 1.48 on debian stable; i wanted to use "Option" as the result for main
mod eigen_exit;

fn main() -> Result<(),()> {
    eigen_exit::trampoline(
        || {
            let ftab = frequency_table(io_contents()?);
            let mut storage = [BTree::Tip(' '); 300];
            let alloc = LocalPlumber { store: &mut storage };
            let tree = huffman_tree(&ftab, alloc)?;
            let cmap = codes(&tree);
            for (c, code) in cmap {
                println!("{:?} => {}", c, code)
            }
            Some(())
        }
    )
}
