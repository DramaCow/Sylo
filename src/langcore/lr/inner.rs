use std::collections::{BTreeSet, HashMap, VecDeque};
use std::iter::once;
use std::rc::Rc;
use crate::langcore::cfg::Symbol;

pub trait BuildItemSets<T: Ord + std::hash::Hash + std::fmt::Debug> {
    fn start_item(&self) -> T;
    fn advance(&self, item: &T) -> T;
    fn symbol_at_dot(&self, item: &T) -> Option<Symbol>;
    fn closure(&self, items: &BTreeSet<T>) -> BTreeSet<T>;

    fn goto<'a, I: Iterator<Item=&'a T>>(&self, items: I, x: &Symbol) -> BTreeSet<T>
    where
        T: 'a,
    {
        self.closure(&items.filter_map(|item| {
            if let Some(y) = self.symbol_at_dot(item) {
                if *x == y {
                    return Some(self.advance(item));
                }
            }
            None
        }).collect::<BTreeSet<T>>())
    }

    fn build(&self) -> (Vec<Vec<T>>, Vec<HashMap<Symbol, usize>>) {
        let initial_items = Rc::new(
            self.closure(&once(self.start_item()).collect())
        );

        let mut itemsets = vec![initial_items.clone()];
        let mut gotos: Vec<HashMap<Symbol, usize>> = vec![HashMap::new()];

        // Item sets we've seen so far mapped to indices in itemsets vector.
        let mut table: HashMap<_, usize> = once((initial_items.clone(), 0)).collect();

        // Queue of itemsets to process.
        // NOTE: A stack could be used here instead; but by using a queue,
        //       the iteration step of the outer-most loop (i) will correspond
        //       to the index of the item set in CC we are currently
        //       transitioning from.
        let mut queue: VecDeque<_> = once(initial_items).collect();

        let mut i = 0_usize;

        while let Some(item_set) = queue.pop_front() {
            let mut iter1 = item_set.iter();
            let mut iter2 = iter1.clone();

            while let Some(item) = iter1.next() {
                if let Some(x) = self.symbol_at_dot(item) {
                    // x has already been processed
                    if gotos[i].contains_key(&x) {
                        continue;
                    }
                    
                    // NOTE: Previously processed items in item_set (those before
                    //       iter2) are guaranteed to not contribute to the output
                    //       item set. As such, goto is only required to process
                    //       from iter2 onwards.
                    let temp = self.goto(iter2, &x);
                    
                    // Check if temp is already in itemsets. If not, we
                    // add to itemsets and push on to process queue.
                    let j = if let Some(&index) = table.get(&temp) {
                        index
                    } else {
                        let new_index = itemsets.len();
                        let temp_rc = Rc::new(temp);

                        itemsets.push(temp_rc.clone());
                        gotos.push(HashMap::new());

                        table.insert(temp_rc.clone(), new_index);
                        queue.push_back(temp_rc);

                        new_index
                    };

                    // Record transition on x
                    gotos[i].insert(x, j);

                    iter2 = iter1.clone();
                }
            }

            i += 1;
        }

        // forces out-of-scope early so all
        // reference counts get decremented.
        drop(table);

        let itemsets: Vec<Vec<T>> = itemsets.into_iter()
            .map(Rc::try_unwrap)
            .map(Result::unwrap)
            .map(|items| items.into_iter().collect())
            .collect();

        (itemsets, gotos)
    }
}