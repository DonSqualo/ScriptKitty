//! Pure Rust FDTD (Finite-Difference Time-Domain) electromagnetic solver
//!
//! Implements the Yee algorithm for full-wave EM simulation.
//! Key features:
//! - 3D staggered grid (E on edges, H on faces)
//! - Leapfrog time-stepping
//! - PML absorbing boundaries
//! - Gaussian pulse sources for broadband excitation
//! - Harminv-like resonance detection via FFT
//!
//! References:
//! - Yee, "Numerical solution of initial boundary value problems" (1966)
//! - Taflove & Hagness, "Computational Electrodynamics" (2005)

use std::f64::consts::PI;

/// Speed of light in vacuum (m/s)
const C0: f64 = 299_792_458.0;
/// Permittivity of free space (F/m)
const EPS0: f64 = 8.854_187_817e-12;
/// Permeability of free space (H/m)
const MU0: f64 = 1.256_637_062e-6;

/// Material properties at a grid point
#[derive(Debug, Clone, Copy)]
pub struct Material {
    /// Relative permittivity (εr)
    pub eps_r: f64,
    /// Relative permeability (μr)  
    pub mu_r: f64,
    /// Electric conductivity (S/m)
    pub sigma_e: f64,
    /// Magnetic conductivity (Ω/m)
    pub sigma_m: f64,
}

impl Material {
    pub fn vacuum() -> Self {
        Self {
            eps_r: 1.0,
            mu_r: 1.0,
            sigma_e: 0.0,
            sigma_m: 0.0,
        }
    }

    pub fn air() -> Self {
        Self::vacuum()
    }

    /// Perfect electric conductor (PEC)
    pub fn pec() -> Self {
        Self {
            eps_r: 1.0,
            mu_r: 1.0,
            sigma_e: 1e10, // Very high conductivity
            sigma_m: 0.0,
        }
    }

    /// Copper (approximate)
    pub fn copper() -> Self {
        Self {
            eps_r: 1.0,
            mu_r: 1.0,
            sigma_e: 5.8e7, // S/m
            sigma_m: 0.0,
        }
    }

    /// Dielectric with given permittivity
    pub fn dielectric(eps_r: f64) -> Self {
        Self {
            eps_r,
            mu_r: 1.0,
            sigma_e: 0.0,
            sigma_m: 0.0,
        }
    }
}

/// Source type for excitation
#[derive(Debug, Clone)]
pub enum Source {
    /// Gaussian pulse centered at fcen with width fwidth
    GaussianPulse {
        fcen: f64,      // Center frequency (Hz)
        fwidth: f64,    // Frequency width (Hz)
        amplitude: f64, // Peak amplitude
        position: [usize; 3], // Grid position
        component: Component, // Which field component to excite
    },
    /// Continuous wave source
    ContinuousWave {
        frequency: f64,
        amplitude: f64,
        position: [usize; 3],
        component: Component,
    },
}

/// Field component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    Ex, Ey, Ez,
    Hx, Hy, Hz,
}

/// Field monitor (probe)
#[derive(Debug, Clone)]
pub struct Monitor {
    pub position: [usize; 3],
    pub component: Component,
    pub samples: Vec<f64>,
}

/// PML (Perfectly Matched Layer) parameters
/// Uses CPML (Convolutional PML) for better performance
#[derive(Debug, Clone)]
pub struct PmlConfig {
    pub thickness: usize, // Number of cells
    pub sigma_max: f64,   // Maximum conductivity (auto-computed if 0)
    pub order: f64,       // Polynomial order (typically 3-4)
    pub kappa_max: f64,   // Maximum kappa (stretching factor, typically 1-15)
    pub alpha_max: f64,   // Maximum alpha for CFS-PML (typically 0-0.3)
}

impl Default for PmlConfig {
    fn default() -> Self {
        Self {
            thickness: 10,
            sigma_max: 0.0, // Auto-compute based on cell size
            order: 3.0,
            kappa_max: 1.0,
            alpha_max: 0.0,
        }
    }
}

impl PmlConfig {
    /// Compute optimal sigma_max for given cell size
    pub fn optimal_sigma_max(dx: f64, order: f64) -> f64 {
        // σ_max = (m+1) / (150 * π * dx) where m is the polynomial order
        // This gives ~-40 dB reflection for typical PML thickness
        (order + 1.0) / (150.0 * PI * dx)
    }
}

/// FDTD simulation configuration
#[derive(Debug, Clone)]
pub struct FdtdConfig {
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    /// Cell size (m)
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    /// Time step (s) - must satisfy CFL condition
    pub dt: f64,
    /// PML configuration
    pub pml: PmlConfig,
    /// Total simulation time (s)
    pub total_time: f64,
}

impl FdtdConfig {
    /// Create config with automatic CFL-stable time step
    pub fn new(nx: usize, ny: usize, nz: usize, cell_size: f64) -> Self {
        let dx = cell_size;
        let dy = cell_size;
        let dz = cell_size;
        
        // CFL condition: dt <= 1/(c * sqrt(1/dx² + 1/dy² + 1/dz²))
        let cfl_factor = 0.9; // Safety margin
        let dt = cfl_factor / (C0 * (1.0/(dx*dx) + 1.0/(dy*dy) + 1.0/(dz*dz)).sqrt());
        
        Self {
            nx, ny, nz,
            dx, dy, dz,
            dt,
            pml: PmlConfig::default(),
            total_time: 0.0, // Set later
        }
    }

    /// Calculate number of time steps
    pub fn num_steps(&self) -> usize {
        (self.total_time / self.dt).ceil() as usize
    }
}

/// CPML coefficients for one direction
#[derive(Debug, Clone)]
struct CpmlCoeffs {
    /// b coefficient: b = exp(-(σ/κ + α) * Δt / ε₀)
    b: Vec<f64>,
    /// c coefficient: c = σ * (b - 1) / (σ * κ + α * κ²)
    c: Vec<f64>,
}

impl CpmlCoeffs {
    fn new(n: usize) -> Self {
        Self {
            b: vec![0.0; n],
            c: vec![0.0; n],
        }
    }
}

/// 3D FDTD simulation with CPML boundaries
pub struct FdtdSimulation {
    pub config: FdtdConfig,
    
    // Electric field components (on cell edges)
    ex: Vec<f64>,
    ey: Vec<f64>,
    ez: Vec<f64>,
    
    // Magnetic field components (on cell faces)
    hx: Vec<f64>,
    hy: Vec<f64>,
    hz: Vec<f64>,
    
    // Material properties (at each grid point)
    // Update coefficients (pre-computed from materials)
    ca_ex: Vec<f64>, cb_ex: Vec<f64>,
    ca_ey: Vec<f64>, cb_ey: Vec<f64>,
    ca_ez: Vec<f64>, cb_ez: Vec<f64>,
    da_hx: Vec<f64>, db_hx: Vec<f64>,
    da_hy: Vec<f64>, db_hy: Vec<f64>,
    da_hz: Vec<f64>, db_hz: Vec<f64>,
    
    // CPML coefficients for each direction
    cpml_x: CpmlCoeffs,
    cpml_y: CpmlCoeffs,
    cpml_z: CpmlCoeffs,
    
    // CPML auxiliary fields (ψ convolution terms)
    // Only non-zero in PML regions
    psi_ex_y: Vec<f64>, psi_ex_z: Vec<f64>,
    psi_ey_x: Vec<f64>, psi_ey_z: Vec<f64>,
    psi_ez_x: Vec<f64>, psi_ez_y: Vec<f64>,
    psi_hx_y: Vec<f64>, psi_hx_z: Vec<f64>,
    psi_hy_x: Vec<f64>, psi_hy_z: Vec<f64>,
    psi_hz_x: Vec<f64>, psi_hz_y: Vec<f64>,
    
    // Sources
    sources: Vec<Source>,
    
    // Monitors
    monitors: Vec<Monitor>,
    
    // Current time step
    time_step: usize,
    
    // Whether PML is enabled
    pml_enabled: bool,
}

impl FdtdSimulation {
    /// Create a new FDTD simulation
    pub fn new(config: FdtdConfig) -> Self {
        let n = config.nx * config.ny * config.nz;
        
        // Initialize fields to zero
        let ex = vec![0.0; n];
        let ey = vec![0.0; n];
        let ez = vec![0.0; n];
        let hx = vec![0.0; n];
        let hy = vec![0.0; n];
        let hz = vec![0.0; n];
        
        // Initialize update coefficients for vacuum
        let ca = vec![1.0; n];
        let cb = vec![config.dt / (EPS0 * config.dx); n];
        let da = vec![1.0; n];
        let db = vec![config.dt / (MU0 * config.dx); n];
        
        // Initialize CPML coefficients
        let cpml_x = Self::compute_cpml_coeffs(config.nx, config.pml.thickness, config.dx, config.dt, &config.pml);
        let cpml_y = Self::compute_cpml_coeffs(config.ny, config.pml.thickness, config.dy, config.dt, &config.pml);
        let cpml_z = Self::compute_cpml_coeffs(config.nz, config.pml.thickness, config.dz, config.dt, &config.pml);
        
        // PML auxiliary fields
        let psi = vec![0.0; n];
        
        Self {
            config,
            ex, ey, ez,
            hx, hy, hz,
            ca_ex: ca.clone(), cb_ex: cb.clone(),
            ca_ey: ca.clone(), cb_ey: cb.clone(),
            ca_ez: ca.clone(), cb_ez: cb.clone(),
            da_hx: da.clone(), db_hx: db.clone(),
            da_hy: da.clone(), db_hy: db.clone(),
            da_hz: da, db_hz: db,
            cpml_x, cpml_y, cpml_z,
            psi_ex_y: psi.clone(), psi_ex_z: psi.clone(),
            psi_ey_x: psi.clone(), psi_ey_z: psi.clone(),
            psi_ez_x: psi.clone(), psi_ez_y: psi.clone(),
            psi_hx_y: psi.clone(), psi_hx_z: psi.clone(),
            psi_hy_x: psi.clone(), psi_hy_z: psi.clone(),
            psi_hz_x: psi.clone(), psi_hz_y: psi,
            sources: Vec::new(),
            monitors: Vec::new(),
            time_step: 0,
            pml_enabled: true,
        }
    }
    
    /// Compute CPML coefficients for one direction
    fn compute_cpml_coeffs(n: usize, thickness: usize, dx: f64, dt: f64, pml: &PmlConfig) -> CpmlCoeffs {
        let mut coeffs = CpmlCoeffs::new(n);
        
        if thickness == 0 {
            return coeffs;
        }
        
        // Compute optimal sigma_max if not specified
        let sigma_max = if pml.sigma_max > 0.0 {
            pml.sigma_max
        } else {
            PmlConfig::optimal_sigma_max(dx, pml.order)
        };
        
        let kappa_max = pml.kappa_max;
        let alpha_max = pml.alpha_max;
        let order = pml.order;
        
        // Compute coefficients for each position
        for i in 0..n {
            // Distance into PML (0 at interface, 1 at boundary)
            let rho = if i < thickness {
                // Lower PML region
                (thickness - i) as f64 / thickness as f64
            } else if i >= n - thickness {
                // Upper PML region
                (i - (n - thickness - 1)) as f64 / thickness as f64
            } else {
                // Interior (no PML)
                0.0
            };
            
            if rho > 0.0 {
                // Polynomial grading
                let rho_pow = rho.powf(order);
                
                let sigma = sigma_max * rho_pow;
                let kappa = 1.0 + (kappa_max - 1.0) * rho_pow;
                let alpha = alpha_max * (1.0 - rho); // Decreases towards boundary
                
                // CPML coefficients
                // b = exp(-(σ/κ + α) * Δt / ε₀)
                let exponent = -(sigma / kappa + alpha) * dt / EPS0;
                coeffs.b[i] = exponent.exp();
                
                // c = σ * (b - 1) / (σ * κ + α * κ²)
                let denom = sigma * kappa + alpha * kappa * kappa;
                if denom.abs() > 1e-20 {
                    coeffs.c[i] = sigma * (coeffs.b[i] - 1.0) / denom;
                }
            }
        }
        
        coeffs
    }

    /// Linear index from 3D coordinates
    #[inline]
    fn idx(&self, i: usize, j: usize, k: usize) -> usize {
        k * self.config.ny * self.config.nx + j * self.config.nx + i
    }

    /// Set material at a grid point
    pub fn set_material(&mut self, i: usize, j: usize, k: usize, mat: Material) {
        let idx = self.idx(i, j, k);
        let dt = self.config.dt;
        let dx = self.config.dx;
        
        // Compute update coefficients
        // Ca = (1 - σΔt/(2ε)) / (1 + σΔt/(2ε))
        // Cb = (Δt/(εΔx)) / (1 + σΔt/(2ε))
        let eps = EPS0 * mat.eps_r;
        let sigma = mat.sigma_e;
        
        let factor_e = sigma * dt / (2.0 * eps);
        let ca = (1.0 - factor_e) / (1.0 + factor_e);
        let cb = (dt / (eps * dx)) / (1.0 + factor_e);
        
        self.ca_ex[idx] = ca;
        self.cb_ex[idx] = cb;
        self.ca_ey[idx] = ca;
        self.cb_ey[idx] = cb;
        self.ca_ez[idx] = ca;
        self.cb_ez[idx] = cb;
        
        // Da = (1 - σ*Δt/(2μ)) / (1 + σ*Δt/(2μ))
        // Db = (Δt/(μΔx)) / (1 + σ*Δt/(2μ))
        let mu = MU0 * mat.mu_r;
        let sigma_m = mat.sigma_m;
        
        let factor_h = sigma_m * dt / (2.0 * mu);
        let da = (1.0 - factor_h) / (1.0 + factor_h);
        let db = (dt / (mu * dx)) / (1.0 + factor_h);
        
        self.da_hx[idx] = da;
        self.db_hx[idx] = db;
        self.da_hy[idx] = da;
        self.db_hy[idx] = db;
        self.da_hz[idx] = da;
        self.db_hz[idx] = db;
    }

    /// Set material for a region
    pub fn set_material_region(
        &mut self,
        i_min: usize, i_max: usize,
        j_min: usize, j_max: usize,
        k_min: usize, k_max: usize,
        mat: Material,
    ) {
        for k in k_min..k_max.min(self.config.nz) {
            for j in j_min..j_max.min(self.config.ny) {
                for i in i_min..i_max.min(self.config.nx) {
                    self.set_material(i, j, k, mat);
                }
            }
        }
    }

    /// Add a source
    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    /// Add a monitor
    pub fn add_monitor(&mut self, position: [usize; 3], component: Component) -> usize {
        let monitor = Monitor {
            position,
            component,
            samples: Vec::with_capacity(self.config.num_steps()),
        };
        self.monitors.push(monitor);
        self.monitors.len() - 1
    }

    /// Enable or disable PML boundaries
    pub fn set_pml_enabled(&mut self, enabled: bool) {
        self.pml_enabled = enabled;
    }

    /// Current simulation time (s)
    pub fn current_time(&self) -> f64 {
        self.time_step as f64 * self.config.dt
    }

    /// Check if position is in PML region for a given direction
    #[inline]
    fn in_pml_x(&self, i: usize) -> bool {
        let pml_thick = self.config.pml.thickness;
        i < pml_thick || i >= self.config.nx - pml_thick
    }
    
    #[inline]
    fn in_pml_y(&self, j: usize) -> bool {
        let pml_thick = self.config.pml.thickness;
        j < pml_thick || j >= self.config.ny - pml_thick
    }
    
    #[inline]
    fn in_pml_z(&self, k: usize) -> bool {
        let pml_thick = self.config.pml.thickness;
        k < pml_thick || k >= self.config.nz - pml_thick
    }

    /// Update E-field (half step) with CPML
    fn update_e(&mut self) {
        let nx = self.config.nx;
        let ny = self.config.ny;
        let nz = self.config.nz;
        let pml_enabled = self.pml_enabled && self.config.pml.thickness > 0;
        
        // Update Ex: dEx/dt = (dHz/dy - dHy/dz) / ε
        for k in 0..nz-1 {
            for j in 0..ny-1 {
                for i in 0..nx {
                    let idx = self.idx(i, j, k);
                    let idx_jm1 = if j > 0 { self.idx(i, j-1, k) } else { idx };
                    let idx_km1 = if k > 0 { self.idx(i, j, k-1) } else { idx };
                    
                    let dhz_dy = self.hz[idx] - self.hz[idx_jm1];
                    let dhy_dz = self.hy[idx] - self.hy[idx_km1];
                    
                    // Standard update
                    self.ex[idx] = self.ca_ex[idx] * self.ex[idx]
                        + self.cb_ex[idx] * (dhz_dy - dhy_dz);
                    
                    // CPML corrections in PML regions
                    if pml_enabled {
                        if self.in_pml_y(j) {
                            // Update psi_ex_y and add contribution
                            self.psi_ex_y[idx] = self.cpml_y.b[j] * self.psi_ex_y[idx]
                                + self.cpml_y.c[j] * dhz_dy;
                            self.ex[idx] += self.cb_ex[idx] * self.psi_ex_y[idx];
                        }
                        if self.in_pml_z(k) {
                            // Update psi_ex_z and add contribution
                            self.psi_ex_z[idx] = self.cpml_z.b[k] * self.psi_ex_z[idx]
                                + self.cpml_z.c[k] * dhy_dz;
                            self.ex[idx] -= self.cb_ex[idx] * self.psi_ex_z[idx];
                        }
                    }
                }
            }
        }
        
        // Update Ey: dEy/dt = (dHx/dz - dHz/dx) / ε
        for k in 0..nz-1 {
            for j in 0..ny {
                for i in 0..nx-1 {
                    let idx = self.idx(i, j, k);
                    let idx_im1 = if i > 0 { self.idx(i-1, j, k) } else { idx };
                    let idx_km1 = if k > 0 { self.idx(i, j, k-1) } else { idx };
                    
                    let dhx_dz = self.hx[idx] - self.hx[idx_km1];
                    let dhz_dx = self.hz[idx] - self.hz[idx_im1];
                    
                    // Standard update
                    self.ey[idx] = self.ca_ey[idx] * self.ey[idx]
                        + self.cb_ey[idx] * (dhx_dz - dhz_dx);
                    
                    // CPML corrections
                    if pml_enabled {
                        if self.in_pml_z(k) {
                            self.psi_ey_z[idx] = self.cpml_z.b[k] * self.psi_ey_z[idx]
                                + self.cpml_z.c[k] * dhx_dz;
                            self.ey[idx] += self.cb_ey[idx] * self.psi_ey_z[idx];
                        }
                        if self.in_pml_x(i) {
                            self.psi_ey_x[idx] = self.cpml_x.b[i] * self.psi_ey_x[idx]
                                + self.cpml_x.c[i] * dhz_dx;
                            self.ey[idx] -= self.cb_ey[idx] * self.psi_ey_x[idx];
                        }
                    }
                }
            }
        }
        
        // Update Ez: dEz/dt = (dHy/dx - dHx/dy) / ε
        for k in 0..nz {
            for j in 0..ny-1 {
                for i in 0..nx-1 {
                    let idx = self.idx(i, j, k);
                    let idx_im1 = if i > 0 { self.idx(i-1, j, k) } else { idx };
                    let idx_jm1 = if j > 0 { self.idx(i, j-1, k) } else { idx };
                    
                    let dhy_dx = self.hy[idx] - self.hy[idx_im1];
                    let dhx_dy = self.hx[idx] - self.hx[idx_jm1];
                    
                    // Standard update
                    self.ez[idx] = self.ca_ez[idx] * self.ez[idx]
                        + self.cb_ez[idx] * (dhy_dx - dhx_dy);
                    
                    // CPML corrections
                    if pml_enabled {
                        if self.in_pml_x(i) {
                            self.psi_ez_x[idx] = self.cpml_x.b[i] * self.psi_ez_x[idx]
                                + self.cpml_x.c[i] * dhy_dx;
                            self.ez[idx] += self.cb_ez[idx] * self.psi_ez_x[idx];
                        }
                        if self.in_pml_y(j) {
                            self.psi_ez_y[idx] = self.cpml_y.b[j] * self.psi_ez_y[idx]
                                + self.cpml_y.c[j] * dhx_dy;
                            self.ez[idx] -= self.cb_ez[idx] * self.psi_ez_y[idx];
                        }
                    }
                }
            }
        }
    }

    /// Update H-field (half step) with CPML
    fn update_h(&mut self) {
        let nx = self.config.nx;
        let ny = self.config.ny;
        let nz = self.config.nz;
        let pml_enabled = self.pml_enabled && self.config.pml.thickness > 0;
        
        // Update Hx: dHx/dt = -(dEz/dy - dEy/dz) / μ
        for k in 0..nz-1 {
            for j in 0..ny-1 {
                for i in 0..nx {
                    let idx = self.idx(i, j, k);
                    let idx_jp1 = self.idx(i, (j+1).min(ny-1), k);
                    let idx_kp1 = self.idx(i, j, (k+1).min(nz-1));
                    
                    let dez_dy = self.ez[idx_jp1] - self.ez[idx];
                    let dey_dz = self.ey[idx_kp1] - self.ey[idx];
                    
                    // Standard update
                    self.hx[idx] = self.da_hx[idx] * self.hx[idx]
                        - self.db_hx[idx] * (dez_dy - dey_dz);
                    
                    // CPML corrections
                    if pml_enabled {
                        if self.in_pml_y(j) {
                            self.psi_hx_y[idx] = self.cpml_y.b[j] * self.psi_hx_y[idx]
                                + self.cpml_y.c[j] * dez_dy;
                            self.hx[idx] -= self.db_hx[idx] * self.psi_hx_y[idx];
                        }
                        if self.in_pml_z(k) {
                            self.psi_hx_z[idx] = self.cpml_z.b[k] * self.psi_hx_z[idx]
                                + self.cpml_z.c[k] * dey_dz;
                            self.hx[idx] += self.db_hx[idx] * self.psi_hx_z[idx];
                        }
                    }
                }
            }
        }
        
        // Update Hy: dHy/dt = -(dEx/dz - dEz/dx) / μ
        for k in 0..nz-1 {
            for j in 0..ny {
                for i in 0..nx-1 {
                    let idx = self.idx(i, j, k);
                    let idx_ip1 = self.idx((i+1).min(nx-1), j, k);
                    let idx_kp1 = self.idx(i, j, (k+1).min(nz-1));
                    
                    let dex_dz = self.ex[idx_kp1] - self.ex[idx];
                    let dez_dx = self.ez[idx_ip1] - self.ez[idx];
                    
                    // Standard update
                    self.hy[idx] = self.da_hy[idx] * self.hy[idx]
                        - self.db_hy[idx] * (dex_dz - dez_dx);
                    
                    // CPML corrections
                    if pml_enabled {
                        if self.in_pml_z(k) {
                            self.psi_hy_z[idx] = self.cpml_z.b[k] * self.psi_hy_z[idx]
                                + self.cpml_z.c[k] * dex_dz;
                            self.hy[idx] -= self.db_hy[idx] * self.psi_hy_z[idx];
                        }
                        if self.in_pml_x(i) {
                            self.psi_hy_x[idx] = self.cpml_x.b[i] * self.psi_hy_x[idx]
                                + self.cpml_x.c[i] * dez_dx;
                            self.hy[idx] += self.db_hy[idx] * self.psi_hy_x[idx];
                        }
                    }
                }
            }
        }
        
        // Update Hz: dHz/dt = -(dEy/dx - dEx/dy) / μ
        for k in 0..nz {
            for j in 0..ny-1 {
                for i in 0..nx-1 {
                    let idx = self.idx(i, j, k);
                    let idx_ip1 = self.idx((i+1).min(nx-1), j, k);
                    let idx_jp1 = self.idx(i, (j+1).min(ny-1), k);
                    
                    let dey_dx = self.ey[idx_ip1] - self.ey[idx];
                    let dex_dy = self.ex[idx_jp1] - self.ex[idx];
                    
                    // Standard update
                    self.hz[idx] = self.da_hz[idx] * self.hz[idx]
                        - self.db_hz[idx] * (dey_dx - dex_dy);
                    
                    // CPML corrections
                    if pml_enabled {
                        if self.in_pml_x(i) {
                            self.psi_hz_x[idx] = self.cpml_x.b[i] * self.psi_hz_x[idx]
                                + self.cpml_x.c[i] * dey_dx;
                            self.hz[idx] -= self.db_hz[idx] * self.psi_hz_x[idx];
                        }
                        if self.in_pml_y(j) {
                            self.psi_hz_y[idx] = self.cpml_y.b[j] * self.psi_hz_y[idx]
                                + self.cpml_y.c[j] * dex_dy;
                            self.hz[idx] += self.db_hz[idx] * self.psi_hz_y[idx];
                        }
                    }
                }
            }
        }
    }

    /// Apply sources at current time
    fn apply_sources(&mut self) {
        let t = self.current_time();
        
        for source in &self.sources {
            match source {
                Source::GaussianPulse { fcen, fwidth, amplitude, position, component } => {
                    // Gaussian pulse: A * exp(-((t - t0) / τ)²) * sin(2πf(t - t0))
                    let tau = 1.0 / (PI * fwidth);
                    let t0 = 4.0 * tau; // Delay to start at near-zero
                    let envelope = (-((t - t0) / tau).powi(2)).exp();
                    let carrier = (2.0 * PI * fcen * (t - t0)).sin();
                    let value = amplitude * envelope * carrier;
                    
                    let idx = self.idx(position[0], position[1], position[2]);
                    match component {
                        Component::Ex => self.ex[idx] += value,
                        Component::Ey => self.ey[idx] += value,
                        Component::Ez => self.ez[idx] += value,
                        Component::Hx => self.hx[idx] += value,
                        Component::Hy => self.hy[idx] += value,
                        Component::Hz => self.hz[idx] += value,
                    }
                }
                Source::ContinuousWave { frequency, amplitude, position, component } => {
                    let value = amplitude * (2.0 * PI * frequency * t).sin();
                    let idx = self.idx(position[0], position[1], position[2]);
                    match component {
                        Component::Ex => self.ex[idx] += value,
                        Component::Ey => self.ey[idx] += value,
                        Component::Ez => self.ez[idx] += value,
                        Component::Hx => self.hx[idx] += value,
                        Component::Hy => self.hy[idx] += value,
                        Component::Hz => self.hz[idx] += value,
                    }
                }
            }
        }
    }

    /// Record monitor values
    fn record_monitors(&mut self) {
        let nx = self.config.nx;
        let ny = self.config.ny;
        
        for monitor in &mut self.monitors {
            // Inline idx calculation to avoid borrow conflict
            let idx = monitor.position[2] * ny * nx + monitor.position[1] * nx + monitor.position[0];
            let value = match monitor.component {
                Component::Ex => self.ex[idx],
                Component::Ey => self.ey[idx],
                Component::Ez => self.ez[idx],
                Component::Hx => self.hx[idx],
                Component::Hy => self.hy[idx],
                Component::Hz => self.hz[idx],
            };
            monitor.samples.push(value);
        }
    }

    /// Advance simulation by one time step
    pub fn step(&mut self) {
        // Leapfrog: E at integer steps, H at half-integer steps
        self.update_h();
        self.apply_sources();
        self.update_e();
        self.record_monitors();
        self.time_step += 1;
    }

    /// Run simulation to completion
    pub fn run(&mut self) {
        let num_steps = self.config.num_steps();
        for _ in 0..num_steps {
            self.step();
        }
    }

    /// Run simulation until fields decay below threshold
    pub fn run_until_decay(&mut self, monitor_idx: usize, threshold: f64, max_steps: usize) {
        let mut peak = 0.0f64;
        
        for step in 0..max_steps {
            self.step();
            
            if let Some(monitor) = self.monitors.get(monitor_idx) {
                if let Some(&last) = monitor.samples.last() {
                    let abs_val = last.abs();
                    if abs_val > peak {
                        peak = abs_val;
                    } else if abs_val < threshold * peak && step > 100 {
                        // Fields have decayed sufficiently
                        break;
                    }
                }
            }
        }
    }

    /// Get monitor samples
    pub fn get_monitor_samples(&self, idx: usize) -> Option<&[f64]> {
        self.monitors.get(idx).map(|m| m.samples.as_slice())
    }

    /// Get field value at a point
    pub fn get_field(&self, i: usize, j: usize, k: usize, component: Component) -> f64 {
        let idx = self.idx(i, j, k);
        match component {
            Component::Ex => self.ex[idx],
            Component::Ey => self.ey[idx],
            Component::Ez => self.ez[idx],
            Component::Hx => self.hx[idx],
            Component::Hy => self.hy[idx],
            Component::Hz => self.hz[idx],
        }
    }

    /// Extract 2D field slice for visualization
    pub fn get_field_slice(&self, plane: FieldPlane, component: Component) -> Vec<f64> {
        let (ni, nj, _plane_idx) = match plane {
            FieldPlane::XY(k) => (self.config.nx, self.config.ny, k),
            FieldPlane::XZ(j) => (self.config.nx, self.config.nz, j),
            FieldPlane::YZ(i) => (self.config.ny, self.config.nz, i),
        };
        
        let mut slice = Vec::with_capacity(ni * nj);
        
        match plane {
            FieldPlane::XY(k) => {
                for j in 0..nj {
                    for i in 0..ni {
                        slice.push(self.get_field(i, j, k, component));
                    }
                }
            }
            FieldPlane::XZ(j) => {
                for k in 0..nj {
                    for i in 0..ni {
                        slice.push(self.get_field(i, j, k, component));
                    }
                }
            }
            FieldPlane::YZ(i) => {
                for k in 0..nj {
                    for j in 0..ni {
                        slice.push(self.get_field(i, j, k, component));
                    }
                }
            }
        }
        
        slice
    }
}

/// Field plane for 2D slices
#[derive(Debug, Clone, Copy)]
pub enum FieldPlane {
    XY(usize), // Plane at z=k
    XZ(usize), // Plane at y=j
    YZ(usize), // Plane at x=i
}

/// Result of resonance analysis
#[derive(Debug, Clone)]
pub struct ResonanceResult {
    pub frequency: f64,  // Hz
    pub q_factor: f64,   // Quality factor
    pub amplitude: f64,  // Relative amplitude
}

/// Find resonant frequencies from time-domain data using FFT
pub fn find_resonances(
    samples: &[f64],
    dt: f64,
    min_freq: f64,
    max_freq: f64,
) -> Vec<ResonanceResult> {
    if samples.len() < 64 {
        return Vec::new();
    }
    
    // Zero-pad to next power of 2 for FFT efficiency
    let n = samples.len().next_power_of_two();
    let mut padded = vec![0.0; n];
    padded[..samples.len()].copy_from_slice(samples);
    
    // Apply Hanning window to reduce spectral leakage
    for (i, sample) in padded.iter_mut().enumerate().take(samples.len()) {
        let window = 0.5 * (1.0 - (2.0 * PI * i as f64 / samples.len() as f64).cos());
        *sample *= window;
    }
    
    // Compute FFT (simple DFT for now - could optimize with real FFT library)
    let spectrum = dft(&padded);
    
    // Find peaks in frequency range
    let df = 1.0 / (n as f64 * dt);
    let min_bin = (min_freq / df).floor() as usize;
    let max_bin = ((max_freq / df).ceil() as usize).min(n / 2);
    
    let magnitudes: Vec<f64> = spectrum[min_bin..max_bin]
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();
    
    // Find local maxima
    let mut peaks = Vec::new();
    let threshold = magnitudes.iter().cloned().fold(0.0f64, f64::max) * 0.1;
    
    for i in 1..magnitudes.len()-1 {
        if magnitudes[i] > magnitudes[i-1] 
            && magnitudes[i] > magnitudes[i+1]
            && magnitudes[i] > threshold
        {
            let freq = (min_bin + i) as f64 * df;
            
            // Estimate Q factor from peak width (FWHM)
            // Find half-power points
            let half_power = magnitudes[i] / 2.0f64.sqrt();
            let mut i_low = i;
            let mut i_high = i;
            while i_low > 0 && magnitudes[i_low] > half_power {
                i_low -= 1;
            }
            while i_high < magnitudes.len() - 1 && magnitudes[i_high] > half_power {
                i_high += 1;
            }
            let bandwidth = (i_high - i_low) as f64 * df;
            let q = if bandwidth > 0.0 { freq / bandwidth } else { 100.0 };
            
            peaks.push(ResonanceResult {
                frequency: freq,
                q_factor: q,
                amplitude: magnitudes[i],
            });
        }
    }
    
    // Sort by amplitude (strongest first)
    peaks.sort_by(|a, b| b.amplitude.partial_cmp(&a.amplitude).unwrap_or(std::cmp::Ordering::Equal));
    
    peaks
}

/// Simple DFT implementation (for small arrays)
fn dft(input: &[f64]) -> Vec<(f64, f64)> {
    let n = input.len();
    let mut output = Vec::with_capacity(n);
    
    for k in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;
        for (j, &x) in input.iter().enumerate() {
            let angle = -2.0 * PI * k as f64 * j as f64 / n as f64;
            re += x * angle.cos();
            im += x * angle.sin();
        }
        output.push((re, im));
    }
    
    output
}

/// Compute S11 from incident and reflected waves
pub fn compute_s11(incident: &[f64], reflected: &[f64], dt: f64) -> Vec<(f64, f64)> {
    if incident.len() != reflected.len() || incident.len() < 64 {
        return Vec::new();
    }
    
    let n = incident.len().next_power_of_two();
    
    // FFT of incident wave
    let mut inc_padded = vec![0.0; n];
    inc_padded[..incident.len()].copy_from_slice(incident);
    let inc_spectrum = dft(&inc_padded);
    
    // FFT of reflected wave
    let mut ref_padded = vec![0.0; n];
    ref_padded[..reflected.len()].copy_from_slice(reflected);
    let ref_spectrum = dft(&ref_padded);
    
    // S11 = reflected / incident
    let df = 1.0 / (n as f64 * dt);
    let mut s11 = Vec::with_capacity(n / 2);
    
    for i in 0..n/2 {
        let freq = i as f64 * df;
        let (inc_re, inc_im) = inc_spectrum[i];
        let (ref_re, ref_im) = ref_spectrum[i];
        
        let inc_mag_sq = inc_re * inc_re + inc_im * inc_im;
        if inc_mag_sq > 1e-20 {
            // Complex division: (ref_re + j*ref_im) / (inc_re + j*inc_im)
            let s11_re = (ref_re * inc_re + ref_im * inc_im) / inc_mag_sq;
            let s11_im = (ref_im * inc_re - ref_re * inc_im) / inc_mag_sq;
            let s11_mag = (s11_re * s11_re + s11_im * s11_im).sqrt();
            let s11_db = 20.0 * s11_mag.max(1e-10).log10();
            s11.push((freq, s11_db));
        } else {
            s11.push((freq, -100.0)); // Very low return loss
        }
    }
    
    s11
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdtd_config_cfl() {
        let config = FdtdConfig::new(50, 50, 50, 1e-3); // 1mm cells
        // CFL: dt <= 1/(c * sqrt(3/dx²)) ≈ 1.92e-12 for 1mm
        assert!(config.dt > 0.0);
        assert!(config.dt < 2e-12);
    }

    #[test]
    fn test_fdtd_vacuum_propagation() {
        // Create small simulation
        let mut config = FdtdConfig::new(20, 20, 20, 1e-2); // 1cm cells
        config.total_time = 1e-9; // 1 ns
        
        let mut sim = FdtdSimulation::new(config.clone());
        
        // Add point source at center
        sim.add_source(Source::GaussianPulse {
            fcen: 1e9,
            fwidth: 0.5e9,
            amplitude: 1.0,
            position: [10, 10, 10],
            component: Component::Ez,
        });
        
        // Monitor at center
        let mon_idx = sim.add_monitor([10, 10, 10], Component::Ez);
        
        // Run a few steps
        for _ in 0..100 {
            sim.step();
        }
        
        // Check that we recorded samples
        let samples = sim.get_monitor_samples(mon_idx).unwrap();
        assert_eq!(samples.len(), 100);
        
        // Some samples should be non-zero (source was active)
        assert!(samples.iter().any(|&s| s.abs() > 1e-10));
    }

    #[test]
    fn test_resonance_detection() {
        // Create synthetic signal with known resonance
        let dt = 1e-12; // 1 ps
        let n = 4096;
        let f_res = 5e9; // 5 GHz resonance
        
        let mut samples = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 * dt;
            // Damped sinusoid (resonance)
            let decay = (-t / 1e-9).exp();
            samples.push(decay * (2.0 * PI * f_res * t).sin());
        }
        
        let resonances = find_resonances(&samples, dt, 1e9, 10e9);
        
        // Should find the resonance near 5 GHz
        assert!(!resonances.is_empty());
        let main_res = &resonances[0];
        let freq_error = (main_res.frequency - f_res).abs() / f_res;
        // Allow 3% error due to FFT bin resolution
        assert!(freq_error < 0.03, "Frequency error too large: {}", freq_error);
    }

    #[test]
    fn test_material_coefficients() {
        let mut config = FdtdConfig::new(10, 10, 10, 1e-3);
        config.total_time = 1e-12;
        
        let mut sim = FdtdSimulation::new(config);
        
        // Set a conductive region
        sim.set_material_region(2, 8, 2, 8, 2, 8, Material::copper());
        
        // Copper has high conductivity, so Ca should be < 1
        let idx = sim.idx(5, 5, 5);
        assert!(sim.ca_ex[idx] < 1.0, "Ca should be damped for conductor");
    }

    #[test]
    fn test_cpml_coefficients() {
        // Test that CPML coefficients are computed correctly
        let mut config = FdtdConfig::new(50, 50, 50, 1e-3);
        config.pml.thickness = 10;
        config.total_time = 1e-12;
        
        let sim = FdtdSimulation::new(config.clone());
        
        // Check that coefficients are computed in PML region
        // Interior should have b=0, c=0 (no PML effect)
        let interior_idx = 25;
        assert!((sim.cpml_x.b[interior_idx]).abs() < 1e-10, "Interior b should be 0");
        assert!((sim.cpml_x.c[interior_idx]).abs() < 1e-10, "Interior c should be 0");
        
        // PML region should have non-zero b (absorption)
        let pml_idx = 2; // Well into PML region
        assert!(sim.cpml_x.b[pml_idx] > 0.0, "PML b should be positive");
        
        // b should approach 1 at the interior interface
        let interface_idx = config.pml.thickness;
        assert!(sim.cpml_x.b[interface_idx].abs() < 0.1, "b at interface should be small");
    }

    #[test]
    fn test_pml_absorption() {
        // Test that PML absorbs waves (energy decays with PML, reflects without)
        let cell_size = 1e-2; // 1cm cells
        let grid_size = 40;
        let pml_thick = 8;
        
        // Calculate total field energy
        fn field_energy(sim: &FdtdSimulation) -> f64 {
            let mut energy = 0.0;
            for k in 0..sim.config.nz {
                for j in 0..sim.config.ny {
                    for i in 0..sim.config.nx {
                        let idx = sim.idx(i, j, k);
                        energy += sim.ex[idx].powi(2) + sim.ey[idx].powi(2) + sim.ez[idx].powi(2);
                        energy += sim.hx[idx].powi(2) + sim.hy[idx].powi(2) + sim.hz[idx].powi(2);
                    }
                }
            }
            energy
        }
        
        // Create simulation WITH PML
        let mut config_pml = FdtdConfig::new(grid_size, grid_size, grid_size, cell_size);
        config_pml.pml.thickness = pml_thick;
        config_pml.total_time = 5e-9; // 5 ns
        
        let mut sim_pml = FdtdSimulation::new(config_pml);
        sim_pml.add_source(Source::GaussianPulse {
            fcen: 1e9,
            fwidth: 0.8e9,
            amplitude: 1.0,
            position: [grid_size/2, grid_size/2, grid_size/2],
            component: Component::Ez,
        });
        
        // Create simulation WITHOUT PML (reflecting boundaries)
        let mut config_no_pml = FdtdConfig::new(grid_size, grid_size, grid_size, cell_size);
        config_no_pml.pml.thickness = 0;
        config_no_pml.total_time = 5e-9;
        
        let mut sim_no_pml = FdtdSimulation::new(config_no_pml);
        sim_no_pml.set_pml_enabled(false);
        sim_no_pml.add_source(Source::GaussianPulse {
            fcen: 1e9,
            fwidth: 0.8e9,
            amplitude: 1.0,
            position: [grid_size/2, grid_size/2, grid_size/2],
            component: Component::Ez,
        });
        
        // Run both for a while (let pulse hit boundaries)
        let steps = 300;
        for _ in 0..steps {
            sim_pml.step();
            sim_no_pml.step();
        }
        
        let energy_pml = field_energy(&sim_pml);
        let energy_no_pml = field_energy(&sim_no_pml);
        
        // PML should have significantly less energy (absorbed at boundaries)
        // Without PML, energy reflects back
        assert!(
            energy_pml < energy_no_pml * 0.5,
            "PML should absorb energy: with_pml={:.2e}, without_pml={:.2e}",
            energy_pml, energy_no_pml
        );
    }
}
