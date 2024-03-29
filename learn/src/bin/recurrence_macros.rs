macro_rules! count_exprs {
    () => (0);
    ($head:expr) => (1);
    ($head:expr, $($tail:expr),*) => (1 + count_exprs!($($tail:expr),*))
} 

macro_rules! recurrence {
    ($seq:ident[$ind:ident]: $sty:ty =$($inits:expr),+; ... ;$recur:expr) => {
        {
            use std::ops::Index;    
            const MEM_SIZE: usize = count_exprs!($($inits),+);

            struct Recurrence {
                mem: [$sty; MEM_SIZE],
                pos: usize,
            }

            impl Iterator for Recurrence {
                type Item = $sty;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.pos < MEM_SIZE {
                        let next_val = self.mem[self.pos];
                        self.pos += 1;
                        Some(next_val)
                    } else {
                        let next_val = {
                            let $ind = self.pos;
                            let $seq = IndexOffset { slice: &self.mem, offset:$ind};
                            $recur
                        };

                        {
                            use std::mem::swap;
                            let mut swap_tmp = next_val;
                            for i in (0..MEM_SIZE).rev() {
                                swap(&mut swap_tmp, &mut self.mem[i]);
                            }
                        }

                        self.pos += 1;
                        Some(next_val)
                    }
                }
            }

            struct IndexOffset<'a> {
                slice:&'a [$sty; MEM_SIZE],
                offset: usize,
            }
            
            impl<'a> Index<usize> for IndexOffset<'a> {
                type Output = $sty;
            
                fn index<'b>(&'b self, index: usize) -> &Self::Output {
                    use std::num::Wrapping;
            
                    let index = Wrapping(index);
                    let offset = Wrapping(self.offset);
                    let window = Wrapping(MEM_SIZE);
            
                    let read_index = index - offset + window;
                    &self.slice[read_index.0]
                }
            }
            
            Recurrence {
                mem: [$($inits),+],
                pos: 0,
            }
        }
    };
}

fn main() {
    let fib = recurrence!(f[n]:usize = 1;...;f[n - 1] * n);

    for e in fib.take(10) {
        println!("{}", e);
    }
}
