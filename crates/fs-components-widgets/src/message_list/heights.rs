//! Heights of all messages: prefix sums over a Fenwick tree.
//!
//! Pattern: none of the table fits (a binary indexed tree that holds the height of every message). Messages not yet seen carry
//! an estimate; once drawn they carry an estimate from their text, and once laid out, the
//! measured height. Sums run in `f64` so 100 000 messages add up exactly.

/// What the list knows about the height of every message.
#[derive(Clone, Debug)]
pub struct Heights {
    value: Vec<f32>,
    exact: Vec<bool>,
    tree: Vec<f64>,
    default: f32,
}

fn low(i: usize) -> usize {
    i & i.wrapping_neg()
}

impl Heights {
    /// `count` messages, all of `default` pixels.
    pub fn new(count: usize, default: f32) -> Self {
        let mut h = Self {
            value: Vec::new(),
            exact: Vec::new(),
            tree: vec![0.0],
            default,
        };
        h.grow(count);
        h
    }

    /// Make room for `count` messages (never fewer).
    pub fn grow(&mut self, count: usize) {
        for _ in self.value.len()..count {
            let i = self.value.len() + 1;
            self.value.push(self.default);
            self.exact.push(false);
            // the node covers `low(i)` entries; all earlier ones are known: add them up
            let mut node = f64::from(self.default);
            let mut j = i - 1;
            while j > i - low(i) {
                node += self.tree[j];
                j -= low(j);
            }
            self.tree.push(node);
        }
    }

    /// `n` older messages were inserted at the front (their heights are not known yet).
    pub fn prepend(&mut self, n: usize) {
        let mut value = vec![self.default; n];
        value.extend_from_slice(&self.value);
        let mut exact = vec![false; n];
        exact.extend_from_slice(&self.exact);
        let default = self.default;
        *self = Self::new(0, default);
        self.grow(value.len());
        for (i, (v, e)) in value.into_iter().zip(exact).enumerate() {
            if (v - default).abs() > f32::EPSILON || e {
                self.set(i, v, e);
            }
        }
    }

    /// Forget everything and start with `count` unknown heights.
    pub fn reset(&mut self, count: usize) {
        *self = Self::new(count, self.default);
    }

    /// How many messages.
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Are there none?
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// The height of message `i`.
    pub fn get(&self, i: usize) -> f32 {
        self.value[i]
    }

    /// Was message `i` measured in a layout?
    pub fn is_exact(&self, i: usize) -> bool {
        self.exact[i]
    }

    /// Set a height; `exact` when it was measured. A measured one is never replaced by an estimate.
    /// Returns the change in pixels.
    pub fn set(&mut self, i: usize, height: f32, exact: bool) -> f32 {
        if self.exact[i] && !exact {
            return 0.0;
        }
        let delta = height - self.value[i];
        self.value[i] = height;
        self.exact[i] = exact;
        let mut k = i + 1;
        while k < self.tree.len() {
            self.tree[k] += f64::from(delta);
            k += low(k);
        }
        delta
    }

    /// Distance from the top to the top of message `i` (`i == len` gives the total).
    #[allow(clippy::cast_possible_truncation)]
    pub fn top(&self, i: usize) -> f32 {
        let mut sum = 0.0;
        let mut k = i.min(self.value.len());
        while k > 0 {
            sum += self.tree[k];
            k -= low(k);
        }
        sum as f32
    }

    /// The text changed: take an estimate again, also over a measured height.
    pub fn reestimate(&mut self, i: usize, height: f32) {
        self.exact[i] = false;
        self.set(i, height, false);
    }

    /// Total height.
    pub fn total(&self) -> f32 {
        self.top(self.value.len())
    }

    /// The message that holds the position `y` pixels from the top (the last one past the end).
    pub fn at(&self, y: f32) -> usize {
        let n = self.value.len();
        if n == 0 {
            return 0;
        }
        let (mut pos, mut rest) = (0usize, f64::from(y.max(0.0)));
        let mut step = n.next_power_of_two();
        while step > 0 {
            let next = pos + step;
            if next <= n && self.tree[next] <= rest {
                pos = next;
                rest -= self.tree[next];
            }
            step >>= 1;
        }
        pos.min(n - 1)
    }
}
