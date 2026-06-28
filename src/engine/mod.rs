pub mod camera;
pub mod combat;
pub mod effects;
pub mod enemy;
pub mod input;
pub mod map;
pub mod pathfinding;
pub mod projectile;
pub mod selection;
pub mod unit;

use camera::Camera;
use combat::CombatSystem;
use effects::EffectsSystem;
use enemy::{Enemy, EnemyType};
use input::InputHandler;
use map::Map;
use projectile::Projectile;
use unit::{Unit, UnitType};

pub struct Game {
    pub map:              Map,
    pub units:            Vec<Unit>,
    pub enemies:          Vec<Enemy>,
    pub projectiles:      Vec<Projectile>,
    pub camera:           Camera,
    pub effects:          EffectsSystem,
    pub input_handler:    InputHandler,
    pub combat:           CombatSystem,
    pub time:             f32,
    pub minimap_visible:  bool,
    pub paused:           bool,
    pub wave_number:      u32,
    pub wave_timer:       f32,
    pub wave_countdown:   f32,
    enemy_id_counter:     usize,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            map:             Map::new(),
            units: vec![
                Unit::new(0, 5, 5, UnitType::Warrior),
                Unit::new(1, 7, 5, UnitType::Archer),
                Unit::new(2, 9, 5, UnitType::Scout),
                Unit::new(3, 5, 7, UnitType::Warrior),
                Unit::new(4, 7, 7, UnitType::Archer),
            ],
            enemies:          vec![],
            projectiles:      vec![],
            camera:           Camera::new(),
            effects:          EffectsSystem::new(),
            input_handler:    InputHandler::new(),
            combat:           CombatSystem::new(),
            time:             0.0,
            minimap_visible:  true,
            paused:           false,
            wave_number:      0,
            wave_timer:       4.0,   // first wave in 4 s
            wave_countdown:   4.0,
            enemy_id_counter: 0,
        };
        g.spawn_initial_enemies();
        g
    }

    fn next_enemy_id(&mut self) -> usize {
        self.enemy_id_counter += 1;
        self.enemy_id_counter
    }

    fn spawn_initial_enemies(&mut self) {
        for (x, y, et) in [
            (50, 10, EnemyType::Grunt),
            (52, 12, EnemyType::Grunt),
            (48, 14, EnemyType::Archer),
            (55, 30, EnemyType::Grunt),
            (50, 50, EnemyType::Brute),
            (45, 55, EnemyType::Grunt),
        ] {
            let id = self.next_enemy_id();
            self.enemies.push(Enemy::new(id, x, y, et));
        }
    }

    pub fn spawn_wave(&mut self) {
        self.wave_number += 1;
        let w = self.wave_number;

        let n_grunts  = 3 + w * 2;
        let n_archers = 1 + w;
        let n_brutes  = if w >= 2 { w - 1 } else { 0 };

        let spawn_ring: Vec<(i32, i32)> = vec![
            (55, 5), (58, 8), (55, 15), (58, 22), (55, 30),
            (58, 38), (55, 48), (48, 58), (35, 58), (22, 58),
        ];
        let mut si = (self.wave_number as usize * 3) % spawn_ring.len();

        let mut add = |et: EnemyType, g: &mut Game| {
            let (x, y) = spawn_ring[si % spawn_ring.len()];
            si += 1;
            let id = g.next_enemy_id();
            g.enemies.push(Enemy::new(id, x, y, et));
        };

        for _ in 0..n_grunts  { add(EnemyType::Grunt,  self); }
        for _ in 0..n_archers { add(EnemyType::Archer, self); }
        for _ in 0..n_brutes  { add(EnemyType::Brute,  self); }

        self.wave_timer    = 30.0 + w as f32 * 5.0;
        self.wave_countdown = self.wave_timer;
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused { return; }
        self.time += dt;

        // Wave countdown
        self.wave_timer    -= dt;
        self.wave_countdown = self.wave_timer.max(0.0);
        if self.wave_timer <= 0.0 {
            self.spawn_wave();
        }

        // Update units (movement, animation)
        for u in &mut self.units { u.update(&self.map, dt); }

        // Combat – swap vecs out to avoid simultaneous borrows
        let mut units       = std::mem::take(&mut self.units);
        let mut enemies     = std::mem::take(&mut self.enemies);
        let mut projectiles = std::mem::take(&mut self.projectiles);

        self.combat.update(dt, &mut units, &mut enemies, &mut projectiles, &mut self.effects, &self.map);

        self.units       = units;
        self.enemies     = enemies;
        self.projectiles = projectiles;

        self.effects.update(dt);
        self.camera.smooth_update(dt);
    }

    pub fn selected_count(&self) -> usize {
        self.units.iter().filter(|u| u.selected).count()
    }
}