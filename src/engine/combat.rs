//! Resolves all combat: unit AI aggro, enemy AI, projectile flight, death.
use crate::engine::{
    unit::Unit,
    enemy::{Enemy, AiState},
    projectile::{Projectile, ProjOwner},
    effects::EffectsSystem,
    pathfinding::find_path,
    map::Map,
};
use egui::Pos2;

pub struct CombatSystem {
    pub proj_counter:    usize,
    pub score:           u32,
    pub killed_enemies:  u32,
    pub lost_units:      u32,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self { proj_counter: 0, score: 0, killed_enemies: 0, lost_units: 0 }
    }

    fn next_pid(&mut self) -> usize { self.proj_counter += 1; self.proj_counter }

    pub fn update(
        &mut self,
        dt:          f32,
        units:       &mut Vec<Unit>,
        enemies:     &mut Vec<Enemy>,
        projectiles: &mut Vec<Projectile>,
        effects:     &mut EffectsSystem,
        map:         &Map,
    ) {
        const TS: f32 = 32.0;

        // 1. Unit combat AI
        for unit in units.iter_mut() {
            if unit.dead { continue; }

            // Find nearest live enemy
            let nearest = enemies.iter()
                .filter(|e| !e.dead)
                .map(|e| {
                    let dx = e.pixel_x - unit.pixel_x;
                    let dy = e.pixel_y - unit.pixel_y;
                    (e.id, (dx * dx + dy * dy).sqrt() / TS)
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            if let Some((eid, dist_t)) = nearest {
                let range = unit.unit_type.attack_range_tiles();
                if dist_t <= range {
                    // In range – stand and attack
                    unit.attack_target = Some(eid);
                    unit.attacking     = true;
                    unit.path.clear();

                    if unit.attack_cooldown <= 0.0 {
                        unit.attack_cooldown = unit.unit_type.attack_cooldown_secs();
                        unit.attack_flash    = 1.0;

                        let dmg  = unit.unit_type.attack_damage();
                        let from = unit.world_pos();
                        let to_pos = enemies.iter()
                            .find(|e| e.id == eid)
                            .map(|e| e.world_pos())
                            .unwrap_or(from);

                        if unit.unit_type.attack_range_tiles() >= 4.0 {
                            // Ranged – fire projectile
                            let pid = self.next_pid();
                            projectiles.push(Projectile::new(
                                pid, ProjOwner::Player, unit.id, eid,
                                from, to_pos, 320.0, dmg, 4.0,
                            ));
                        } else {
                            // Melee – instant damage
                            if let Some(e) = enemies.iter_mut().find(|e| e.id == eid) {
                                e.take_damage(dmg);
                                effects.spawn_hit(to_pos, true);
                            }
                        }
                    }
                } else {
                    // Out of range – stop attacking
                    if unit.attack_target == Some(eid) {
                        unit.attacking     = false;
                        unit.attack_target = None;
                    }
                }
            } else {
                unit.attacking     = false;
                unit.attack_target = None;
            }
        }

        // 2. Enemy AI
        // Snapshot unit positions (avoids double-borrow)
        let unit_pos_snap: Vec<(usize, f32, f32)> = units.iter()
            .filter(|u| !u.dead)
            .map(|u| (u.id, u.pixel_x, u.pixel_y))
            .collect();

        // Collect ranged attack orders separately to avoid borrow clash
        let mut ranged_orders: Vec<(usize, usize, f32, Pos2, Pos2)> = vec![];
        // (enemy_id, unit_id, dmg, from, to)

        for enemy in enemies.iter_mut() {
            enemy.tick(dt);
            if enemy.dead { continue; }

            // Find nearest live player unit
            let nearest = unit_pos_snap.iter()
                .map(|&(uid, ux, uy)| {
                    let dx = ux - enemy.pixel_x;
                    let dy = uy - enemy.pixel_y;
                    (uid, (dx * dx + dy * dy).sqrt() / TS)
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            match nearest {
                Some((uid, dist_t)) if dist_t <= enemy.enemy_type.detect_range_tiles() => {
                    let range = enemy.enemy_type.attack_range_tiles();
                    if dist_t <= range {
                        enemy.ai_state = AiState::Attack { target_id: uid };
                        enemy.path.clear();

                        if enemy.attack_cooldown <= 0.0 {
                            enemy.attack_cooldown = enemy.enemy_type.attack_cooldown_secs();
                            enemy.attack_flash    = 1.0;

                            let dmg  = enemy.enemy_type.attack_damage();
                            let from = enemy.world_pos();

                            if enemy.enemy_type.is_ranged() {
                                if let Some(&(_, ux, uy)) = unit_pos_snap.iter().find(|&&(id, _, _)| id == uid) {
                                    let to = Pos2::new(ux, uy);
                                    ranged_orders.push((enemy.id, uid, dmg, from, to));
                                }
                            }
                            // Melee damage is applied below in a separate pass to avoid borrow clash
                        }
                    } else {
                        // Chase
                        enemy.ai_state = AiState::Chase { target_id: uid };
                        if enemy.path.is_empty() {
                            if let Some(&(_, ux, uy)) = unit_pos_snap.iter().find(|&&(id, _, _)| id == uid) {
                                let gtx = (ux / TS) as i32;
                                let gty = (uy / TS) as i32;
                                let path = find_path(map, enemy.tile_pos(), (gtx, gty));
                                if !path.is_empty() { enemy.path = path; }
                            }
                        }
                        enemy.step(dt);
                    }
                }
                _ => {
                    // Patrol around spawn
                    enemy.ai_state    = AiState::Patrol;
                    enemy.patrol_timer -= dt;
                    if enemy.patrol_timer <= 0.0 {
                        enemy.patrol_timer = 2.0 + (enemy.id as f32 * 0.41).sin().abs() * 2.5;
                        let ox  = (enemy.id as i32 * 3 + 2) % 5 - 2;
                        let oy  = (enemy.id as i32 * 7 + 1) % 5 - 2;
                        let gx  = (enemy.spawn_x + ox).clamp(1, crate::engine::map::MAP_W - 2);
                        let gy  = (enemy.spawn_y + oy).clamp(1, crate::engine::map::MAP_H - 2);
                        if map.walkable(gx, gy) {
                            let path = find_path(map, enemy.tile_pos(), (gx, gy));
                            if !path.is_empty() { enemy.path = path; }
                        }
                    }
                    enemy.step(dt);
                }
            }
        }

        // 3. Ranged enemy → fire projectiles
        for (eid, uid, dmg, from, to) in ranged_orders {
            let pid = self.next_pid();
            projectiles.push(Projectile::new(
                pid, ProjOwner::Enemy, eid, uid,
                from, to, 250.0, dmg, 4.0,
            ));
        }

        // 4. Melee enemy → direct unit damage
        let mut melee_hits: Vec<(usize, f32, Pos2)> = vec![];
        for enemy in enemies.iter() {
            if enemy.dead || enemy.attack_cooldown > 0.01 { continue; }
            if enemy.enemy_type.is_ranged() { continue; }
            if let AiState::Attack { target_id } = enemy.ai_state {
                let dmg  = enemy.enemy_type.attack_damage();
                if let Some(u) = units.iter().find(|u| u.id == target_id && !u.dead) {
                    melee_hits.push((target_id, dmg, u.world_pos()));
                }
            }
        }
        for (uid, dmg, pos) in melee_hits {
            if let Some(u) = units.iter_mut().find(|u| u.id == uid) {
                u.take_damage(dmg);
                effects.spawn_hit(pos, false);
            }
        }

        // 5. Projectile flight and hit resolution
        let mut proj_hits: Vec<(ProjOwner, usize, f32, Pos2)> = vec![];
        projectiles.retain_mut(|p| {
            let live_pos = match p.owner {
                ProjOwner::Player => enemies.iter().find(|e| e.id == p.dst_id && !e.dead).map(|e| e.world_pos()),
                ProjOwner::Enemy  => units.iter().find(|u| u.id == p.dst_id && !u.dead).map(|u| u.world_pos()),
            };
            let done = p.update(dt, live_pos);
            if done {
                proj_hits.push((p.owner.clone(), p.dst_id, p.damage, p.pos));
                false
            } else {
                true
            }
        });

        for (owner, dst_id, dmg, pos) in proj_hits {
            match owner {
                ProjOwner::Player => {
                    if let Some(e) = enemies.iter_mut().find(|e| e.id == dst_id && !e.dead) {
                        e.take_damage(dmg);
                        effects.spawn_hit(pos, true);
                    }
                }
                ProjOwner::Enemy => {
                    if let Some(u) = units.iter_mut().find(|u| u.id == dst_id && !u.dead) {
                        u.take_damage(dmg);
                        effects.spawn_hit(pos, false);
                    }
                }
            }
        }

        // 6. Death processing
        for u in units.iter_mut() {
            if !u.dead && u.health <= 0.0 {
                u.dead     = true;
                u.health   = 0.0;
                self.lost_units += 1;
                effects.spawn_death(u.world_pos(), false);
            }
        }
        for e in enemies.iter_mut() {
            if !e.dead && e.health <= 0.0 {
                e.dead     = true;
                e.health   = 0.0;
                self.score         += e.enemy_type.xp_value();
                self.killed_enemies += 1;
                effects.spawn_death(e.world_pos(), true);
            }
        }

        // Remove dead player units immediately; fade dead enemies then remove
        units.retain(|u| !u.dead);
        enemies.retain(|e| !(e.dead && e.death_timer <= 0.0));
    }
}