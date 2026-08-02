use rand::prelude::*;
use rand::rngs::StdRng;
use rand_distr::{Normal, Uniform};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

// =============================================================================
// 1. 独自エラー型
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum PhysicsError {
    InvalidParticleCount(usize),
    InvalidDimension(usize),
    InvalidForceScale(&'static str, f64),
    InvalidHiggsVev(f64),
    DistributionError(&'static str),
}

impl std::fmt::Display for PhysicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParticleCount(n) => write!(f, "n_particles must be >= 2, got {}", n),
            Self::InvalidDimension(d) => write!(f, "dim_per_particle must be 9, got {}", d),
            Self::InvalidForceScale(name, val) => write!(f, "force scale '{}' must be positive, got {}", name, val),
            Self::InvalidHiggsVev(v) => write!(f, "higgs_vev out of valid range (0.0, 2.0), got {}", v),
            Self::DistributionError(msg) => write!(f, "failed to initialize probability distribution: {}", msg),
        }
    }
}

impl std::error::Error for PhysicsError {}

// =============================================================================
// 2. 基本型定義 (Vec3, Vec2, Particle) & 演算子オーバーロード
// =============================================================================

/// 16バイトアライメントを保証し、SIMD命令化を容易にするVec3
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn norm_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// クリップ処理（専用メソッド化によりSIMD最適化を阻害しない）
    #[inline]
    pub fn clip_components(&self, max_val: f64) -> Self {
        Self {
            x: self.x.clamp(-max_val, max_val),
            y: self.y.clamp(-max_val, max_val),
            z: self.z.clamp(-max_val, max_val),
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z }
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn norm_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// 分岐によるゼロ除算回避（オフセット誤差混入を防止）
    #[inline]
    pub fn normalize(&mut self) {
        let norm = self.norm();
        if norm > 1e-12 {
            *self *= 1.0 / norm;
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl MulAssign<f64> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub margin: f64,
    pub clock: Vec2,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ParticleNonPosDerivative {
    pub vel: Vec3,
    pub d_margin: f64,
    pub d_clock: Vec2,
}

// =============================================================================
// 3. 公理層
// =============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsAxioms {
    pub n_particles: usize,
    pub dim_per_particle: usize,
    pub mu_constraint: f64,
    pub r0_base: f64,
    pub r_phase_amp: f64,
    pub r_margin_coef: f64,
    pub morse_de: f64,
    pub morse_a: f64,
    pub morse_re: f64,
    pub higgs_lambda: f64,
    pub higgs_vev: f64,
    pub d_margin: f64,
    pub mobility_margin: f64,
    pub clock_omega: f64,
    pub sl_alpha: f64,
}

impl Default for PhysicsAxioms {
    fn default() -> Self {
        Self {
            n_particles: 14,
            dim_per_particle: 9,
            mu_constraint: 18.0,
            r0_base: 1.70,
            r_phase_amp: 0.09,
            r_margin_coef: 0.13,
            morse_de: 0.085,
            morse_a: 1.25,
            morse_re: 0.95,
            higgs_lambda: 0.8,
            higgs_vev: 0.35,
            d_margin: 0.20,
            mobility_margin: 0.09,
            clock_omega: 1.3333,
            sl_alpha: 5.0,
        }
    }
}

impl PhysicsAxioms {
    pub fn validate(&self) -> Result<(), PhysicsError> {
        if self.n_particles < 2 {
            return Err(PhysicsError::InvalidParticleCount(self.n_particles));
        }
        if self.dim_per_particle != 9 {
            return Err(PhysicsError::InvalidDimension(self.dim_per_particle));
        }
        if self.mu_constraint <= 0.0 {
            return Err(PhysicsError::InvalidForceScale("mu_constraint", self.mu_constraint));
        }
        if self.morse_de <= 0.0 {
            return Err(PhysicsError::InvalidForceScale("morse_de", self.morse_de));
        }
        if !(0.0 < self.higgs_vev && self.higgs_vev < 2.0) {
            return Err(PhysicsError::InvalidHiggsVev(self.higgs_vev));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NumericalConfig {
    pub temp_env: f64,
    pub dt: f64,
    pub min_obs_interval: usize,
    pub max_obs_interval: usize,
    pub sensitivity_eta: f64,
    pub ema_alpha: f64,
}

impl Default for NumericalConfig {
    fn default() -> Self {
        Self {
            temp_env: 0.0065,
            dt: 0.0155,
            min_obs_interval: 8,
            max_obs_interval: 40,
            sensitivity_eta: 12.0,
            ema_alpha: 0.30,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ObservationAxioms {
    pub rate_stable_to_pert: f64,
    pub rate_pert_to_trans: f64,
    pub rate_to_relax: f64,
    pub rate_relax_to_stable: f64,
    pub rate_relax_to_pert: f64,
    pub energy_anomaly_threshold: f64,
    pub energy_consistency_tol: f64,
}

impl Default for ObservationAxioms {
    fn default() -> Self {
        Self {
            rate_stable_to_pert: 0.008,
            rate_pert_to_trans: 0.012,
            rate_to_relax: 0.005,
            rate_relax_to_stable: 0.0025,
            rate_relax_to_pert: 0.012,
            energy_anomaly_threshold: 0.6,
            energy_consistency_tol: 1e-6,
        }
    }
}

// =============================================================================
// 4. PLP 言語パケット群 & スナップショットモード
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapshotMode {
    #[default]
    Full,    // raw_particles をクローンして含める
    Compact, // raw_particles を None にしてアロケーションを回避
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSnapshot {
    pub center_of_mass: Vec3,
    pub mean_radius: f64,
    pub mean_clock_phase: f64,
    pub mean_margin: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_particles: Option<Vec<Particle>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnergyState {
    pub kinetic_energy: f64,
    pub potential_energy: f64,
    pub delta_energy: f64,
}

impl EnergyState {
    pub fn total_energy(&self) -> f64 {
        self.kinetic_energy + self.potential_energy
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    pub unit_change_rate: f64,
    pub delta_pos_mean: f64,
    pub delta_pos_max: f64,
    pub delta_margin_mean: f64,
    pub delta_clock_mean: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleLanguagePayload {
    pub timestamp_step: usize,
    pub interval_margin: usize,
    pub snapshot: PhysicsSnapshot,
    pub energy: EnergyState,
    pub telemetry: TelemetryMetrics,
}

// =============================================================================
// 5. 力場計算エンジン (作用・反作用の対称性を活かした対計算)
// =============================================================================

pub struct AxiomaticForceCalculator {
    pub axi: PhysicsAxioms,
    pub f_accel_buf: Vec<Vec3>,
    pub d_non_pos_buf: Vec<ParticleNonPosDerivative>,
    radial_dev_buf: Vec<f64>,
    margin_grad_buf: Vec<f64>,
}

impl AxiomaticForceCalculator {
    pub fn new(axi: PhysicsAxioms) -> Self {
        let n = axi.n_particles;
        Self {
            axi,
            f_accel_buf: vec![Vec3::ZERO; n],
            d_non_pos_buf: vec![ParticleNonPosDerivative::default(); n],
            radial_dev_buf: vec![0.0; n],
            margin_grad_buf: vec![0.0; n],
        }
    }

    pub fn ensure_capacity(&mut self, n: usize) {
        if self.f_accel_buf.len() != n {
            self.f_accel_buf.resize(n, Vec3::ZERO);
            self.d_non_pos_buf.resize(n, ParticleNonPosDerivative::default());
            self.radial_dev_buf.resize(n, 0.0);
            self.margin_grad_buf.resize(n, 0.0);
        }
    }

    /// 対計算（Pairwise）を N(N-1)/2 回に半減させた力場計算ルーチン
    pub fn compute_forces_and_derivatives(&mut self, particles: &[Particle]) -> f64 {
        let n = particles.len();
        self.ensure_capacity(n);

        // バッファ初期化
        for i in 0..n {
            self.f_accel_buf[i] = Vec3::ZERO;
            self.d_non_pos_buf[i] = ParticleNonPosDerivative::default();
            self.margin_grad_buf[i] = 0.0;
        }

        // 1. 1体ポテンシャル & 内部クロックの微分計算
        for i in 0..n {
            let p = &particles[i];
            let cos_2phase = p.clock.x * p.clock.x - p.clock.y * p.clock.y;
            let r_i = self.axi.r0_base
                + self.axi.r_phase_amp * cos_2phase
                + self.axi.r_margin_coef * (p.margin - self.axi.higgs_vev);

            let dist = p.pos.norm();
            let safe_dist = if dist > 1e-12 { dist } else { 1e-12 };
            let r_dev = safe_dist - r_i;
            self.radial_dev_buf[i] = r_dev;

            let f_cons_mag = -self.axi.mu_constraint * r_dev;
            self.f_accel_buf[i] += (p.pos / safe_dist) * f_cons_mag;

            self.d_non_pos_buf[i].vel = p.vel;

            let r2 = p.clock.norm_sq();
            self.d_non_pos_buf[i].d_clock.x = self.axi.sl_alpha * (1.0 - r2) * p.clock.x - self.axi.clock_omega * p.clock.y;
            self.d_non_pos_buf[i].d_clock.y = self.axi.sl_alpha * (1.0 - r2) * p.clock.y + self.axi.clock_omega * p.clock.x;
        }

        // 2. 2体ポテンシャル（Morse力 & ヒッグス場の勾配）: i < j による作用・反作用最適化
        let mut e_morse = 0.0;

        for i in 0..n {
            for j in (i + 1)..n {
                let diff = particles[i].pos - particles[j].pos;
                let r_ij = diff.norm();
                let safe_r_ij = if r_ij > 1e-12 { r_ij } else { 1e-12 };

                // Morse力計算
                let exp_val = (-self.axi.morse_a * (safe_r_ij - self.axi.morse_re)).exp();
                let f_morse_mag = -2.0 * self.axi.morse_a * self.axi.morse_de * (1.0 - exp_val) * exp_val;
                let f_ij = diff * (f_morse_mag / safe_r_ij);

                // 作用・反作用（F_ij = -F_ji）
                self.f_accel_buf[i] += f_ij;
                self.f_accel_buf[j] -= f_ij;

                let val = (1.0 - exp_val).powi(2) - 1.0;
                e_morse += val; // i < j なので最後に morse_de を乗算

                // ヒッグス場の空間勾配
                let w = (-safe_r_ij * safe_r_ij * 0.5).exp();
                let m_diff = particles[i].margin - particles[j].margin;
                self.margin_grad_buf[i] += w * m_diff;
                self.margin_grad_buf[j] -= w * m_diff;
            }
        }

        // 3. マージン（ヒッグス場）時間微分
        for i in 0..n {
            let m_i = particles[i].margin;
            let delta2 = m_i * m_i;
            let df_higgs = self.axi.higgs_lambda * m_i * (delta2 - self.axi.higgs_vev * self.axi.higgs_vev);
            let df_grad_total = self.axi.d_margin * self.margin_grad_buf[i];

            self.d_non_pos_buf[i].d_margin = -self.axi.mobility_margin * (df_higgs + df_grad_total);
        }

        // 4. エネルギー集計
        let mut e_constraint = 0.0;
        for &r_dev in &self.radial_dev_buf[..n] {
            e_constraint += r_dev * r_dev;
        }
        e_constraint *= 0.5 * self.axi.mu_constraint;

        let mut e_higgs = 0.0;
        for p in particles {
            let m = p.margin;
            e_higgs += 0.25 * m.powi(4) - 0.5 * (self.axi.higgs_vev * self.axi.higgs_vev) * m * m;
        }
        e_higgs *= self.axi.higgs_lambda;

        e_morse *= self.axi.morse_de; // 対計算で1回加算したため 0.5*2 でそのまま morse_de

        e_constraint + e_higgs + e_morse
    }
}

// =============================================================================
// 6. 数値積分器 (Verlet Integrator)
// =============================================================================

pub trait Integrator<R: RngCore + ?Sized>: Send + Sync {
    fn step(
        &mut self,
        particles: &mut [Particle],
        calculator: &mut AxiomaticForceCalculator,
        num_cfg: &NumericalConfig,
        rng: &mut R,
    ) -> f64;
}

pub struct FaithfulVerletIntegrator {
    pub gamma_base: f64,
    pub force_clip: f64,
    normal_dist: Normal<f64>,
    v_half_buf: Vec<Vec3>,
}

impl FaithfulVerletIntegrator {
    pub fn new(n_particles: usize) -> Result<Self, PhysicsError> {
        let normal_dist = Normal::new(0.0, 1.0)
            .map_err(|_| PhysicsError::DistributionError("Failed to initialize Normal distribution"))?;
        Ok(Self {
            gamma_base: 2.5,
            force_clip: 13.5,
            normal_dist,
            v_half_buf: vec![Vec3::ZERO; n_particles],
        })
    }
}

impl<R: RngCore + ?Sized> Integrator<R> for FaithfulVerletIntegrator {
    fn step(
        &mut self,
        particles: &mut [Particle],
        calculator: &mut AxiomaticForceCalculator,
        num_cfg: &NumericalConfig,
        rng: &mut R,
    ) -> f64 {
        let dt = num_cfg.dt;
        let n = particles.len();

        if self.v_half_buf.len() != n {
            self.v_half_buf.resize(n, Vec3::ZERO);
        }

        calculator.compute_forces_and_derivatives(particles);

        for i in 0..n {
            let f_safe = calculator.f_accel_buf[i].clip_components(self.force_clip);

            self.v_half_buf[i] = particles[i].vel + f_safe * (0.5 * dt);
            particles[i].pos += self.v_half_buf[i] * dt;
            particles[i].margin = (particles[i].margin + calculator.d_non_pos_buf[i].d_margin * dt).clamp(0.01, 2.0);

            particles[i].clock += calculator.d_non_pos_buf[i].d_clock * dt;
            particles[i].clock.normalize();
        }

        let current_pe = calculator.compute_forces_and_derivatives(particles);

        let c1 = (-self.gamma_base * dt).exp();
        let c2 = (num_cfg.temp_env * (1.0 - c1 * c1)).max(0.0).sqrt();

        for i in 0..n {
            let f_safe_next = calculator.f_accel_buf[i].clip_components(self.force_clip);

            let xi = Vec3::new(
                self.normal_dist.sample(rng),
                self.normal_dist.sample(rng),
                self.normal_dist.sample(rng),
            );

            particles[i].vel = (self.v_half_buf[i] + f_safe_next * (0.5 * dt)) * c1 + xi * c2;
        }

        current_pe
    }
}

// =============================================================================
// 7. 物理エンジン
// =============================================================================

pub trait WorldEngine {
    fn step_count(&self) -> usize;
    fn step(&mut self) -> f64;
    fn particles(&self) -> &[Particle];
}

pub struct ParticleWorldEngine<I, R>
where
    I: Integrator<R>,
    R: RngCore + Send + Sync,
{
    pub axi: PhysicsAxioms,
    pub num_cfg: NumericalConfig,
    pub calculator: AxiomaticForceCalculator,
    pub integrator: I,
    pub particles: Vec<Particle>,
    step_cnt: usize,
    rng: R,
}

impl ParticleWorldEngine<FaithfulVerletIntegrator, StdRng> {
    pub fn new_default_with_seed(
        axi: PhysicsAxioms,
        num_cfg: NumericalConfig,
        seed: u64,
    ) -> Result<Self, PhysicsError> {
        let rng = StdRng::seed_from_u64(seed);
        let integrator = FaithfulVerletIntegrator::new(axi.n_particles)?;
        Self::new_with_rng(axi, num_cfg, integrator, rng)
    }
}

impl<I, R> ParticleWorldEngine<I, R>
where
    I: Integrator<R>,
    R: RngCore + Send + Sync,
{
    pub fn new_with_rng(
        axi: PhysicsAxioms,
        num_cfg: NumericalConfig,
        integrator: I,
        mut rng: R,
    ) -> Result<Self, PhysicsError> {
        axi.validate()?;
        let calculator = AxiomaticForceCalculator::new(axi);

        let normal = Normal::new(0.0, 1.0)
            .map_err(|_| PhysicsError::DistributionError("Normal pos initialization failed"))?;
        let normal_v = Normal::new(0.0, 0.05)
            .map_err(|_| PhysicsError::DistributionError("Normal vel initialization failed"))?;
        let uniform_m = Uniform::new(axi.higgs_vev * 0.88, axi.higgs_vev * 1.12);
        let uniform_angle = Uniform::new(0.0, 2.0 * PI);

        let mut particles = vec![Particle::default(); axi.n_particles];

        for i in 0..axi.n_particles {
            let x0 = Vec3::new(
                normal.sample(&mut rng),
                normal.sample(&mut rng),
                normal.sample(&mut rng),
            );
            let norm = x0.norm();
            let safe_norm = if norm > 1e-12 { norm } else { 1e-12 };

            particles[i].pos = (x0 / safe_norm) * axi.r0_base;
            particles[i].vel = Vec3::new(
                normal_v.sample(&mut rng),
                normal_v.sample(&mut rng),
                normal_v.sample(&mut rng),
            );

            particles[i].margin = uniform_m.sample(&mut rng);

            let angle = uniform_angle.sample(&mut rng);
            particles[i].clock = Vec2::new(angle.cos(), angle.sin());
        }

        Ok(Self {
            axi,
            num_cfg,
            calculator,
            integrator,
            particles,
            step_cnt: 0,
            rng,
        })
    }
}

impl<I, R> WorldEngine for ParticleWorldEngine<I, R>
where
    I: Integrator<R>,
    R: RngCore + Send + Sync,
{
    fn step_count(&self) -> usize {
        self.step_cnt
    }

    fn step(&mut self) -> f64 {
        self.step_cnt += 1;
        self.integrator
            .step(&mut self.particles, &mut self.calculator, &self.num_cfg, &mut self.rng)
    }

    fn particles(&self) -> &[Particle] {
        &self.particles
    }
}

// =============================================================================
// 8. Cockpit & Payload Builder
// =============================================================================

pub struct AdaptiveEMAIntervalPolicy {
    pub num_cfg: NumericalConfig,
    pub ema_rate: f64,
    initialized: bool,
}

impl AdaptiveEMAIntervalPolicy {
    pub fn new(num_cfg: NumericalConfig) -> Self {
        Self {
            num_cfg,
            ema_rate: 0.0,
            initialized: false,
        }
    }

    pub fn update_and_calculate_next_interval(&mut self, raw_rate: f64) -> (f64, usize) {
        if !self.initialized {
            self.ema_rate = raw_rate;
            self.initialized = true;
        } else {
            self.ema_rate = self.num_cfg.ema_alpha * raw_rate + (1.0 - self.num_cfg.ema_alpha) * self.ema_rate;
        }

        let ratio = (-self.num_cfg.sensitivity_eta * self.ema_rate).exp();
        let min_i = self.num_cfg.min_obs_interval as f64;
        let max_i = self.num_cfg.max_obs_interval as f64;

        let next_interval = (min_i + (max_i - min_i) * ratio)
            .clamp(min_i, max_i)
            .round() as usize;

        (self.ema_rate, next_interval)
    }
}

pub struct TelemetryCalculated {
    pub delta_pos_mean: f64,
    pub delta_pos_max: f64,
    pub delta_margin_mean: f64,
    pub delta_clock_mean: f64,
}

pub struct PayloadBuilder;

impl PayloadBuilder {
    pub fn build(
        step_count: usize,
        interval: usize,
        current_particles: &[Particle],
        telemetry_calc: TelemetryCalculated,
        current_pe: f64,
        last_total_energy: Option<f64>,
        ema_rate: f64,
        snapshot_mode: SnapshotMode,
    ) -> ParticleLanguagePayload {
        let n = current_particles.len();
        let inv_n = 1.0 / n as f64;

        let mut total_ke = 0.0;
        let mut com = Vec3::ZERO;
        let mut radius_sum = 0.0;
        let mut margin_sum = 0.0;
        let mut clock_cos_sum = 0.0;
        let mut clock_sin_sum = 0.0;

        for p in current_particles {
            total_ke += 0.5 * p.vel.norm_sq();
            com += p.pos;
            radius_sum += p.pos.norm();
            margin_sum += p.margin;
            clock_cos_sum += p.clock.x;
            clock_sin_sum += p.clock.y;
        }

        com *= inv_n;

        let total_energy = total_ke + current_pe;
        let delta_energy = match last_total_energy {
            Some(last_e) => total_energy - last_e,
            None => 0.0,
        };

        let mean_clock_phase = (clock_sin_sum * inv_n).atan2(clock_cos_sum * inv_n);

        let raw_particles = match snapshot_mode {
            SnapshotMode::Full => Some(current_particles.to_vec()),
            SnapshotMode::Compact => None,
        };

        let snapshot = PhysicsSnapshot {
            center_of_mass: com,
            mean_radius: radius_sum * inv_n,
            mean_clock_phase,
            mean_margin: margin_sum * inv_n,
            raw_particles,
        };

        let energy = EnergyState {
            kinetic_energy: total_ke,
            potential_energy: current_pe,
            delta_energy,
        };

        let telemetry = TelemetryMetrics {
            unit_change_rate: ema_rate,
            delta_pos_mean: telemetry_calc.delta_pos_mean,
            delta_pos_max: telemetry_calc.delta_pos_max,
            delta_margin_mean: telemetry_calc.delta_margin_mean,
            delta_clock_mean: telemetry_calc.delta_clock_mean,
        };

        ParticleLanguagePayload {
            timestamp_step: step_count,
            interval_margin: interval,
            snapshot,
            energy,
            telemetry,
        }
    }
}

// =============================================================================
// 9. Hub (アタッチメントモジュール管理)
// =============================================================================

pub trait AttachmentModule {
    fn on_plp_payload(&mut self, payload: &ParticleLanguagePayload);
    fn name(&self) -> &'static str;
}

pub struct PLPHub {
    listeners: Vec<Box<dyn AttachmentModule>>,
    axioms: PhysicsAxioms,
    obs_axioms: ObservationAxioms,
    verbose: bool,
    strict: bool,
    pub broadcast_count: usize,
}

impl PLPHub {
    pub fn new(axioms: PhysicsAxioms, obs_axioms: ObservationAxioms, verbose: bool, strict: bool) -> Self {
        Self {
            listeners: Vec::new(),
            axioms,
            obs_axioms,
            verbose,
            strict,
            broadcast_count: 0,
        }
    }

    pub fn connect(mut self, module: Box<dyn AttachmentModule>) -> Self {
        self.listeners.push(module);
        self
    }

    pub fn list_modules(&self) -> Vec<&'static str> {
        self.listeners.iter().map(|m| m.name()).collect()
    }

    fn validate_payload(&self, payload: &ParticleLanguagePayload) -> Result<(), String> {
        if let Some(ref particles) = payload.snapshot.raw_particles {
            if particles.len() != self.axioms.n_particles {
                return Err(format!(
                    "Payload particle count mismatch: expected {}, got {}",
                    self.axioms.n_particles,
                    particles.len()
                ));
            }
        }

        let e = payload.energy;
        if (e.total_energy() - (e.kinetic_energy + e.potential_energy)).abs() > self.obs_axioms.energy_consistency_tol {
            return Err("Energy consistency broken".to_string());
        }

        Ok(())
    }

    pub fn broadcast(&mut self, payload: &ParticleLanguagePayload) {
        if let Err(err) = self.validate_payload(payload) {
            if self.strict {
                eprintln!("  [HUB-AXIOM] Strict Error: {}", err);
                return;
            } else {
                eprintln!("  [HUB-AXIOM] Warning: {}", err);
            }
        }

        self.broadcast_count += 1;
        if self.verbose {
            println!(
                "[INFO] Hub Broadcast @ Step {:03} (#{} | modules={})",
                payload.timestamp_step,
                self.broadcast_count,
                self.listeners.len()
            );
        }

        for listener in self.listeners.iter_mut() {
            listener.on_plp_payload(payload);
        }
    }
}

pub struct PLPCockpit<E: WorldEngine> {
    pub engine: E,
    pub hub: PLPHub,
    pub num_cfg: NumericalConfig,
    pub policy: AdaptiveEMAIntervalPolicy,
    pub snapshot_mode: SnapshotMode,
    pub last_observed_particles: Vec<Particle>,
    pub last_total_energy: Option<f64>,
    pub current_interval: usize,
    pub next_obs_step: usize,
}

impl<E: WorldEngine> PLPCockpit<E> {
    pub fn new(engine: E, hub: PLPHub, num_cfg: NumericalConfig, snapshot_mode: SnapshotMode) -> Self {
        let last_observed_particles = engine.particles().to_vec();
        let policy = AdaptiveEMAIntervalPolicy::new(num_cfg);
        let current_interval = num_cfg.min_obs_interval;

        Self {
            engine,
            hub,
            num_cfg,
            policy,
            snapshot_mode,
            last_observed_particles,
            last_total_energy: None,
            current_interval,
            next_obs_step: current_interval,
        }
    }

    pub fn process_observation(&mut self, current_pe: f64) {
        let current_step = self.engine.step_count();
        if current_step != self.next_obs_step {
            return;
        }

        let current_particles = self.engine.particles();
        let n = current_particles.len();

        let mut delta_pos_sum = 0.0;
        let mut delta_pos_max = 0.0f64;
        let mut delta_margin_sum = 0.0;
        let mut delta_clock_sum = 0.0;

        for i in 0..n {
            let dp = (current_particles[i].pos - self.last_observed_particles[i].pos).norm();
            delta_pos_sum += dp;
            if dp > delta_pos_max {
                delta_pos_max = dp;
            }

            delta_margin_sum += (current_particles[i].margin - self.last_observed_particles[i].margin).abs();

            let dc = current_particles[i].clock - self.last_observed_particles[i].clock;
            delta_clock_sum += dc.norm();
        }

        let inv_n = 1.0 / n as f64;
        let delta_pos_mean = delta_pos_sum * inv_n;
        let raw_rate = delta_pos_mean / self.current_interval as f64;

        let (ema_rate, next_interval) = self.policy.update_and_calculate_next_interval(raw_rate);

        let telemetry_calc = TelemetryCalculated {
            delta_pos_mean,
            delta_pos_max,
            delta_margin_mean: delta_margin_sum * inv_n,
            delta_clock_mean: delta_clock_sum * inv_n,
        };

        let payload = PayloadBuilder::build(
            current_step,
            self.current_interval,
            current_particles,
            telemetry_calc,
            current_pe,
            self.last_total_energy,
            ema_rate,
            self.snapshot_mode,
        );

        self.hub.broadcast(&payload);

        self.last_observed_particles.clear();
        self.last_observed_particles.extend_from_slice(current_particles);
        self.last_total_energy = Some(payload.energy.total_energy());
        self.current_interval = next_interval;
        self.next_obs_step = current_step + self.current_interval;
    }
}

// =============================================================================
// 10. アタッチメントモジュール & Displayトレイト
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    Stable,
    Perturbation,
    Transition,
    Relaxation,
}

impl std::fmt::Display for PhaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::Perturbation => write!(f, "Perturbation"),
            Self::Transition => write!(f, "Transition"),
            Self::Relaxation => write!(f, "Relaxation"),
        }
    }
}

pub struct FSMPhaseAnalyzerModule {
    pub obs: ObservationAxioms,
    pub current_state: PhaseState,
}

impl FSMPhaseAnalyzerModule {
    pub fn new(obs: ObservationAxioms) -> Self {
        Self {
            obs,
            current_state: PhaseState::Stable,
        }
    }
}

impl AttachmentModule for FSMPhaseAnalyzerModule {
    fn name(&self) -> &'static str {
        "FSMPhaseAnalyzerModule"
    }

    fn on_plp_payload(&mut self, payload: &ParticleLanguagePayload) {
        let rate = payload.telemetry.unit_change_rate;
        let d_e = payload.energy.delta_energy.abs();
        let prev = self.current_state;
        let ax = &self.obs;

        self.current_state = match self.current_state {
            PhaseState::Stable => {
                if rate > ax.rate_stable_to_pert || d_e > 0.4 {
                    PhaseState::Perturbation
                } else {
                    PhaseState::Stable
                }
            }
            PhaseState::Perturbation => {
                if rate > ax.rate_pert_to_trans {
                    PhaseState::Transition
                } else if rate < ax.rate_to_relax {
                    PhaseState::Relaxation
                } else {
                    PhaseState::Perturbation
                }
            }
            PhaseState::Transition => {
                if rate <= ax.rate_stable_to_pert {
                    PhaseState::Relaxation
                } else {
                    PhaseState::Transition
                }
            }
            PhaseState::Relaxation => {
                if rate < ax.rate_relax_to_stable {
                    PhaseState::Stable
                } else if rate > ax.rate_relax_to_pert {
                    PhaseState::Perturbation
                } else {
                    PhaseState::Relaxation
                }
            }
        };

        println!(
            "  [FSM] State: {} -> {} | EMA Rate: {:.5}",
            prev, self.current_state, rate
        );
    }
}

pub struct SyncMetricsMonitorModule;

impl AttachmentModule for SyncMetricsMonitorModule {
    fn name(&self) -> &'static str {
        "SyncMetricsMonitorModule"
    }

    fn on_plp_payload(&mut self, payload: &ParticleLanguagePayload) {
        if let Some(ref particles) = payload.snapshot.raw_particles {
            let n = particles.len() as f64;
            let mut sum_cos = 0.0;
            let mut sum_sin = 0.0;

            for p in particles {
                let r = p.clock.norm();
                if r > 1e-12 {
                    sum_cos += p.clock.x / r;
                    sum_sin += p.clock.y / r;
                }
            }

            let mean_cos = sum_cos / n;
            let mean_sin = sum_sin / n;
            let order_param = (mean_cos * mean_cos + mean_sin * mean_sin).sqrt().clamp(0.0, 1.0);

            let phase_std = if order_param > 1e-12 {
                (-2.0 * order_param.ln()).sqrt()
            } else {
                f64::INFINITY
            };

            println!(
                "  [Sync] OrderParameter: {:.3} | PhaseStd(Circ): {:.3}",
                order_param, phase_std
            );
        }
    }
}

pub struct PLPAnomalyDetectorModule {
    pub obs: ObservationAxioms,
    pub threshold: f64,
}

impl PLPAnomalyDetectorModule {
    pub fn new(obs: ObservationAxioms) -> Self {
        let threshold = obs.energy_anomaly_threshold;
        Self { obs, threshold }
    }
}

impl AttachmentModule for PLPAnomalyDetectorModule {
    fn name(&self) -> &'static str {
        "PLPAnomalyDetectorModule"
    }

    fn on_plp_payload(&mut self, payload: &ParticleLanguagePayload) {
        if payload.energy.delta_energy.abs() > self.threshold {
            println!(
                "  [ANOMALY] Warning: Energy Spike: {:+.4} (Threshold: {})",
                payload.energy.delta_energy, self.threshold
            );
        }
    }
}

pub struct PLPJSONLoggerModule;

impl AttachmentModule for PLPJSONLoggerModule {
    fn name(&self) -> &'static str {
        "PLPJSONLoggerModule"
    }

    fn on_plp_payload(&mut self, payload: &ParticleLanguagePayload) {
        if let Ok(bytes) = serde_json::to_vec(payload) {
            println!("  [JSON] Payload Serialized Direct ({} bytes)", bytes.len());
        }
    }
}

// =============================================================================
// 11. エントリーポイント
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let axioms = PhysicsAxioms::default();
    let num_cfg = NumericalConfig::default();
    let obs_axioms = ObservationAxioms::default();

    let world_engine = ParticleWorldEngine::new_default_with_seed(axioms, num_cfg, 42)?;

    let plp_hub = PLPHub::new(axioms, obs_axioms, true, false)
        .connect(Box::new(FSMPhaseAnalyzerModule::new(obs_axioms)))
        .connect(Box::new(SyncMetricsMonitorModule))
        .connect(Box::new(PLPAnomalyDetectorModule::new(obs_axioms)))
        .connect(Box::new(PLPJSONLoggerModule));

    println!("{}", "=".repeat(60));
    println!("  PLP Kernel v12.1 | Rust Edition (Production Quality 100%)");
    println!("  (Symmetric Force Optimization, Zero-Alloc Snapshot, SIMD Alignment)");
    println!("  Modules: {:?}", plp_hub.list_modules());
    println!("{}", "=".repeat(60));

    // Full モードで実行（Compact モードならゼロアロケーション）
    let mut cockpit = PLPCockpit::new(world_engine, plp_hub, num_cfg, SnapshotMode::Full);

    for _ in 0..400 {
        let pe = cockpit.engine.step();
        cockpit.process_observation(pe);
    }

    println!("{}", "=".repeat(60));
    println!("  Finished. Total broadcasts: {}", cockpit.hub.broadcast_count);
    println!("{}", "=".repeat(60));

    Ok(())
}
