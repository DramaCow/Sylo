use std::cmp::{min, Ordering};

/// Given a directed graph where each node is associated with a set and
/// adjacency is defined by `successors` operator, this function computes the
/// union over the associated sets in the transitive closure of each node.
/// 
/// # Examples
/// 
/// ```
/// use std::collections::HashSet;
/// use sylo::utils::transitive_closure;
/// 
/// // Initialized to unit sets containing themselves.
/// let mut reachable_nodes: Vec<HashSet<usize>> = (0..6).map(|x| {
///     let mut set = HashSet::new();
///     set.insert(x);
///     set
/// }).collect();
/// 
/// // Successors are given by an adjacency list expressed as a function.
/// let successors = |x: usize| {
///     match x {
///         0 => (&[1]).iter().copied(),
///         1 => (&[2]).iter().copied(),
///         2 => (&[0, 3, 5]).iter().copied(),
///         4 => (&[3]).iter().copied(),
///         _ => (&[]).iter().copied(),
///     }
/// };
/// 
/// // Extend is a generalization of an inplace union operation.
/// let extend = |a: &mut HashSet<usize>, b: &HashSet<usize>| {
///     a.extend(b);
/// };
/// 
/// let has_cycle = transitive_closure(&mut reachable_nodes, successors, extend);
/// let counts: Vec<usize> = reachable_nodes.iter().map(HashSet::len).collect();
/// 
/// assert_eq!(counts, &[5, 5, 5, 1, 2, 1]);
/// assert_eq!(has_cycle, true);
/// ```
pub fn transitive_closure<S, L, F, T>(values: &mut Vec<T>, successors: S, extend: F) -> bool
where
    S: FnMut(usize) -> L,
    L: IntoIterator<Item = usize>,
    F: FnMut(&mut T, &T),
    T: Clone,
{
    let num_nodes = values.len();

    let mut tar = Tarjan {
        successors,
        extend,
        vertices:    vec![0; num_nodes].into_boxed_slice(),
        stack_depth: 0,
        low_link:    vec![0; num_nodes].into_boxed_slice(),
        list_index:  num_nodes + 1,
        has_cycle:   false,
        values,
    };

    for x in 0..num_nodes {
        if tar.low_link[x] == 0 {
            tar.traverse(x);
        }
    }

    for x in 0..num_nodes {
        let head_x = tar.vertices[tar.low_link[x] - 1];
        if x != head_x {
            tar.values[x] = tar.values[head_x].clone();
        }
    }

    tar.has_cycle
}

// =================
// === INTERNALS ===
// =================

struct Tarjan<'a, S, L, F, T>
where
    S: FnMut(usize) -> L,
    L: IntoIterator<Item = usize>,
    F: FnMut(&mut T, &T),
    T: Clone,
{
    successors: S,
    extend: F,
    vertices: Box<[usize]>, //< vertices[..stack_depth] is the stack
    stack_depth: usize,
    low_link: Box<[usize]>,
    list_index: usize,
    has_cycle: bool,
    values: &'a mut Vec<T>,
}

impl<'a, S, L, F, T> Tarjan<'a, S, L, F, T>
where
    S: FnMut(usize) -> L,
    L: IntoIterator<Item = usize>,
    F: FnMut(&mut T, &T),
    T: Clone,
{
    fn traverse(&mut self, x: usize) {
        self.stack_push(x);
        self.low_link[x] = self.stack_depth;

        for y in (self.successors)(x) {
            // if y hasn't been seen yet
            if self.low_link[y] == 0 {
                self.traverse(y);
            }
            
            // if y is on the stack (and as such must be in the same scc as x)
            if self.low_link[y] <= self.stack_depth {
                self.low_link[x] = min(self.low_link[x], self.low_link[y]);
            } else {
                let head_y = self.vertices[self.low_link[y] - 1]; // head of scc containing y
                self.extend(x, head_y);
            }
        }

        let head_x = self.vertices[self.low_link[x] - 1];

        if x == head_x {
            self.list_index -= 1;

            loop {
                let w = self.stack_pop();
                self.low_link[w] = self.list_index;

                if w == x {
                    break;
                }

                self.has_cycle = true;
                self.extend(x, w);
            }

            self.vertices[self.list_index - 1] = x;
        }
    }

    fn stack_push(&mut self, x: usize) {
        self.vertices[self.stack_depth] = x;
        self.stack_depth += 1;
    }

    fn stack_pop(&mut self) -> usize {
        if self.stack_depth > 0 {
            self.stack_depth -= 1;
            self.vertices[self.stack_depth]
        } else {
            panic!()
        }
    }

    fn extend(&mut self, x: usize, y: usize) {
        match x.cmp(&y) {
            Ordering::Greater => {
                // result: [... y ... | x ...]
                let (left, right) = self.values.split_at_mut(x);
                (self.extend)(&mut right[0], &left[y]);
            },
            Ordering::Less => {
                // result: [... x ... | y ...]
                let (left, right) = self.values.split_at_mut(y);
                (self.extend)(&mut left[x], &right[0]);
            },
            Ordering::Equal => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::transitive_closure;
    use std::iter::once;
    use std::collections::HashSet;

    fn extend(a: &mut HashSet<usize>, b: &HashSet<usize>) {
        a.extend(b);
    }

    fn unit_sets(count: usize) -> Vec<HashSet<usize>> {
        (0..count).map(|x| once(x).collect::<HashSet<usize>>()).collect()
    }

    #[test]
    fn tarjan_test_1() {
        let successors = |x: usize| {
            match x {
                0 => (&[1]).iter().copied(),
                1 => (&[2]).iter().copied(),
                2 => (&[0, 3, 5]).iter().copied(),
                4 => (&[3]).iter().copied(),
                _ => (&[]).iter().copied(),
            }
        };
        let mut reachable_nodes = unit_sets(6);
        let has_cycle = transitive_closure(&mut reachable_nodes, successors, extend);
        let counts: Vec<usize> = reachable_nodes.iter().map(HashSet::len).collect();
        assert_eq!(counts, &[5, 5, 5, 1, 2, 1]);
        assert!(has_cycle);
    }

    #[test]
    fn tarjan_test_2() {
        let successors = |x: usize| {
            match x {
                0 => (&[1, 4]).iter().copied(),
                1 => (&[5]).iter().copied(),
                2 => (&[1, 3, 6]).iter().copied(),
                3 => (&[6]).iter().copied(),
                4 => (&[0, 5]).iter().copied(),
                5 => (&[2, 6]).iter().copied(),
                6 => (&[7]).iter().copied(),
                7 => (&[3]).iter().copied(),
                _ => (&[]).iter().copied(),
            }
        };
        let mut reachable_nodes = unit_sets(8);
        let has_cycle = transitive_closure(&mut reachable_nodes, successors, extend);
        let counts: Vec<usize> = reachable_nodes.iter().map(HashSet::len).collect();
        assert_eq!(counts, &[8, 6, 6, 3, 8, 6, 3, 3]);
        assert!(has_cycle);
    }

    #[test]
    fn tarjan_test_3() {
        let successors = |x: usize| {
            match x {
                0 | 2 => (&[1]).iter().copied(),
                _ => (&[]).iter().copied(),
            }
        };
        let mut reachable_nodes = unit_sets(3);
        let has_cycle = transitive_closure(&mut reachable_nodes, successors, extend);
        let counts: Vec<usize> = reachable_nodes.iter().map(HashSet::len).collect();
        assert_eq!(counts, &[2, 1, 2]);
        assert!(!has_cycle);
    }

    #[test]
    fn tarjan_test_4() {
        let successors = |x: usize| {
            match x {
                0 => (&[1, 2]).iter().copied(),
                2 => (&[1]).iter().copied(),
                _ => (&[]).iter().copied(),
            }
        };
        let mut reachable_nodes = unit_sets(3);
        let has_cycle = transitive_closure(&mut reachable_nodes, successors, extend);
        let counts: Vec<usize> = reachable_nodes.iter().map(HashSet::len).collect();
        assert_eq!(counts, &[3, 1, 2]);
        assert!(!has_cycle);
    }

    #[test]
    fn tarjan_test_5() {
        let successors = |x: usize| {
            match x {
                0 => (&[2, 3]).iter().copied(),
                2 | 3 => (&[1]).iter().copied(),
                _ => (&[]).iter().copied(),
            }
        };
        let mut reachable_nodes = unit_sets(4);
        let has_cycle = transitive_closure(&mut reachable_nodes, successors, extend);
        let counts: Vec<usize> = reachable_nodes.iter().map(HashSet::len).collect();
        assert_eq!(counts, &[4, 1, 2, 2]);
        assert!(!has_cycle);
    }
}