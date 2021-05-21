use std::fmt::Debug;
use std::cmp::min;

#[must_use]
pub fn tarjan<'a, S, L, U, T>(initial_values: Vec<T>, successors: S, union: U) -> Vec<T>
where
    S: FnMut(usize) -> L,
    L: IntoIterator<Item = &'a usize>,
    U: FnMut(&T, &T) -> T,
    T: Clone + Debug,
{
    let num_nodes = initial_values.len();

    let mut tar = Tarjan {
        successors,
        union,
        stack: Vec::new(),
        low_link: vec![0; num_nodes],
        result: initial_values,
    };

    #[allow(clippy::needless_range_loop)]
    for x in 0..num_nodes {
        if tar.is_unvisited(x) {
            tar.traverse(x);
        }
    }

    tar.result
}

// =================
// === INTERNALS ===
// =================

struct Tarjan<'a, S, L, U, T>
where
    S: FnMut(usize) -> L,
    L: IntoIterator<Item = &'a usize>,
    U: FnMut(&T, &T) -> T,
    T: Clone + Debug,
{
    successors: S,
    union: U,
    stack: Vec<usize>,
    low_link: Vec<usize>,
    result: Vec<T>,
}

impl<'a, S, L, U, T> Tarjan<'a, S, L, U, T>
where
    S: FnMut(usize) -> L,
    L: IntoIterator<Item = &'a usize>,
    U: FnMut(&T, &T) -> T,
    T: Clone + Debug,
{
    const INF: usize = usize::max_value();

    fn traverse(&mut self, x: usize) {
        self.stack.push(x);
        let d = self.stack.len();
        self.low_link[x] = d;

        for &y in (self.successors)(x) {
            if self.is_unvisited(y) {
                self.traverse(y);
                self.result[x] = (self.union)(&self.result[x], &self.result[y]); // F[x] = F[x] U F[y]
            } else if self.low_link[y] == Self::INF {
                self.result[x] = (self.union)(&self.result[x], &self.result[y]); // F[x] = F[x] U F[y]
            }
            self.low_link[x] = min(self.low_link[x], self.low_link[y]);
            // self.result[x] = (self.union)(&self.result[x], &self.result[y]); // F[x] = F[x] U F[y]
        }

        if self.low_link[x] == d {
            while let Some(top) = self.stack.pop() {
                self.low_link[top] = Self::INF;
                if top == x {
                    break;
                }
                self.result[top] = self.result[x].clone(); // F[top] = F[x]
            }
        }
    }

    fn is_unvisited(&self, x: usize) -> bool {
        self.low_link[x] == 0
    }
}

#[cfg(test)]
mod tests {
    use super::tarjan;
    use std::iter::once;
    use std::collections::HashSet;

    #[test]
    fn tarjan_test_1() {
        let successors = |x: usize| -> &[usize] {
            match x {
                0 => &[1],
                1 => &[2],
                2 => &[0, 3, 5],
                4 => &[3],
                _ => &[],
            }
        };
        let counts = tarjan(vec![1; 6], successors, |a: &usize, b: &usize| a + b);
        assert_eq!(counts, &[5, 5, 5, 1, 2, 1]);
    }

    #[test]
    fn tarjan_test_2() {
        let successors = |x: usize| -> &[usize] {
            match x {
                0 => &[1, 4],
                1 => &[5],
                2 => &[1, 3, 6],
                3 => &[6],
                4 => &[0, 5],
                5 => &[2, 6],
                6 => &[7],
                7 => &[3],
                _ => &[],
            }
        };
        let counts = tarjan(vec![1; 8], successors, |a: &usize, b: &usize| a + b);
        assert_eq!(counts, &[8, 6, 6, 3, 8, 6, 3, 3]);
    }

    #[test]
    fn tarjan_test_3() {
        let successors = |x: usize| -> &[usize] {
            match x {
                0 | 2 => &[1],
                _ => &[],
            }
        };
        let counts = tarjan(vec![1; 3], successors, |a: &usize, b: &usize| a + b);
        // let union = |a: &HashSet<usize>, b: &HashSet<usize>| -> HashSet<usize> {
        //     a.union(b).copied().collect()
        // };
        // let sccs = tarjan((0..3).map(|x| once(x).collect::<HashSet<usize>>()).collect(), successors, union);
        // println!("{:?}", sccs);
        assert_eq!(counts, &[2, 1, 2]);
    }

    #[test]
    fn tarjan_test_4() {
        let successors = |x: usize| -> &[usize] {
            match x {
                0 => &[1, 2],
                2 => &[1],
                _ => &[],
            }
        };
        let counts = tarjan(vec![1; 3], successors, |a: &usize, b: &usize| a + b);
        assert_eq!(counts, &[3, 1, 2]);
    }
}