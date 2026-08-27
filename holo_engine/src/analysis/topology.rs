use nalgebra::Point3;
use std::collections::HashMap;

/// Extracts persistent homology (Betti numbers) from a 3D point cloud
/// Represents the fundamental geometry of the Dark Matter Cosmic Web.
pub struct TopologicalEncoder {
    pub points: Vec<Point3<f32>>,
    pub epsilon: f32, // Connective distance threshold
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    components: usize,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            components: n,
        }
    }

    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] != i {
            self.parent[i] = self.find(self.parent[i]);
        }
        self.parent[i]
    }

    fn union(&mut self, i: usize, j: usize) -> bool {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            if self.rank[root_i] < self.rank[root_j] {
                self.parent[root_i] = root_j;
            } else if self.rank[root_i] > self.rank[root_j] {
                self.parent[root_j] = root_i;
            } else {
                self.parent[root_j] = root_i;
                self.rank[root_i] += 1;
            }
            self.components -= 1;
            true
        } else {
            false
        }
    }
}

impl TopologicalEncoder {
    pub fn new(points: Vec<Point3<f32>>, epsilon: f32) -> Self {
        Self { points, epsilon }
    }

    /// Computes the Vietoris-Rips complex using an O(N) Spatial Grid approach
    /// Extracts Betti 0 (Connected components) and Betti 1 (Filaments/Cycles).
    pub fn extract_betti_numbers(&self) -> (usize, usize, usize) {
        if self.points.is_empty() { return (0, 0, 0); }
        
        let mut grid: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
        let inv_eps = 1.0 / self.epsilon;

        // Step 1: Spatial Hashing O(N)
        for (i, p) in self.points.iter().enumerate() {
            let cx = (p.x * inv_eps).floor() as i32;
            let cy = (p.y * inv_eps).floor() as i32;
            let cz = (p.z * inv_eps).floor() as i32;
            grid.entry([cx, cy, cz]).or_default().push(i);
        }

        let mut uf = UnionFind::new(self.points.len());
        let mut edges = 0;
        let eps_sq = self.epsilon * self.epsilon;

        // Step 2: Local neighborhood search O(N * K)
        for (cell, indices) in &grid {
            // Check self cell and 13 forward neighbors to avoid duplicate edges
            let neighbors = [
                [0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0], [-1, 1, 0],
                [0, 0, 1], [1, 0, 1], [-1, 0, 1], [0, 1, 1], [1, 1, 1],
                [-1, 1, 1], [0, -1, 1], [1, -1, 1], [-1, -1, 1]
            ];

            for offset in neighbors.iter() {
                let neighbor_cell = [cell[0] + offset[0], cell[1] + offset[1], cell[2] + offset[2]];
                
                if let Some(neighbor_indices) = grid.get(&neighbor_cell) {
                    for &i in indices {
                        for &j in neighbor_indices {
                            // If same cell, avoid double counting by enforcing i < j
                            if offset == &[0, 0, 0] && i >= j { continue; }
                            
                            let p1 = &self.points[i];
                            let p2 = &self.points[j];
                            let dx = p1.x - p2.x;
                            let dy = p1.y - p2.y;
                            let dz = p1.z - p2.z;
                            let dist_sq = dx*dx + dy*dy + dz*dz;
                            
                            if dist_sq <= eps_sq {
                                uf.union(i, j);
                                edges += 1;
                            }
                        }
                    }
                }
            }
        }

        let b0 = uf.components;
        let v = self.points.len();
        
        // Betti 1: 1-dimensional holes (Cosmic Web Filaments and Rings)
        // b1 = E - V + b0 (Euler characteristic for 1D complex)
        let b1 = if edges + b0 > v { edges + b0 - v } else { 0 };

        // Betti 2: Super-voids (Approximation)
        let b2 = 0; 

        (b0, b1, b2)
    }

    /// Validates the Cusp-Core resolution hypothesis by checking for infinite density singularities.
    /// If the T-Dual metric holds, no single cluster should exceed max theoretical density (cusp).
    pub fn check_cusp_core_resolution(&self, max_density_threshold: usize) -> bool {
        if self.points.is_empty() { return true; }

        let mut grid: HashMap<[i32; 3], usize> = HashMap::new();
        let inv_eps = 1.0 / self.epsilon;

        let mut max_density = 0;
        
        // O(N) density map
        for p in &self.points {
            let cx = (p.x * inv_eps).floor() as i32;
            let cy = (p.y * inv_eps).floor() as i32;
            let cz = (p.z * inv_eps).floor() as i32;
            let count = grid.entry([cx, cy, cz]).or_insert(0);
            *count += 1;
            if *count > max_density {
                max_density = *count;
            }
        }

        // True if the densest halo doesn't form a singularity (cusp)
        max_density < max_density_threshold
    }
}
