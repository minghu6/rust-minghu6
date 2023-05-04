use coll::set;

use crate::{Graph, test::path::Path, sp::*};



impl Graph {
    pub(crate) fn fix_one_negative_cycle_johnson(
        &mut self,
        g2: &mut Graph,
        cycle: &[usize],
        positive: bool,
    ) {
        g2.verify_negative_cycle(cycle).unwrap();

        let p = Path::from_cycle(&g2, cycle).freeze();
        let mut totw = p.weight();
        assert!(totw < 0);

        for (u, v, w) in p.iter() {
            if self.contains_edge((u, v)) {
                if w < 0 {
                    // reverse weight
                    totw += 2 * (-w);
                    set!(self.w => (u, v) => -w);

                    if !self.is_dir {
                        set!(self.w => (v, u) => -w);
                    }

                    if !positive {
                        if totw >= 0 {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn fix_one_negative_cycle_normal(
        &mut self,
        cycle: &[usize],
        positive: bool,
    ) {
        self.verify_negative_cycle(cycle).unwrap();

        let p = Path::from_cycle(self, cycle).freeze();
        let mut totw = p.weight();
        assert!(totw < 0);

        for (u, v, w) in p.iter() {
            if w < 0 {
                // reverse weight
                totw += 2 * (-w);
                set!(self.w => (u, v) => -w);

                if !self.is_dir {
                    set!(self.w => (v, u) => -w);
                }

                if !positive {
                    if totw >= 0 {
                        break;
                    }
                }
            }
        }
    }

    pub fn fix_negative_cycle_floyd(&mut self, positive: bool) {
        loop {
            if let Err(cycle) = SPFloyd::new(&self) {
                self.fix_one_negative_cycle_normal(&cycle, positive);
            } else {
                break;
            }
        }
    }

    pub fn fix_negative_cycle_johnson(&mut self, positive: bool) {
        loop {
            if let Err((mut g2, cycle)) = SPJohnson::new(&self) {
                self.fix_one_negative_cycle_johnson(&mut g2, &cycle, positive);
            } else {
                break;
            }
        }
    }

    pub fn fix_negative_cycle_bellmanford(&mut self, positive: bool) {
        let vs: Vec<usize> = self.vertexs().collect();
        let n = vs.len();
        let mut i = 0;
        loop {
            if i >= n {
                break;
            }

            if let Err(cycle) = SPBellmanFord::new(&self, vs[i]) {
                self.fix_one_negative_cycle_normal(&cycle, positive);

                continue;
            } else {
                i += 1;
            }
        }
    }

    pub fn fix_negative_cycle_spfa(&mut self, positive: bool) {
        let vs: Vec<usize> = self.vertexs().collect();
        let n = vs.len();
        let mut i = 0;
        loop {
            if i >= n {
                break;
            }

            if let Err(cycle) = SPFA::new(&self, vs[i]) {
                self.fix_one_negative_cycle_normal(&cycle, positive);

                continue;
            } else {
                i += 1;
            }
        }
    }
}

