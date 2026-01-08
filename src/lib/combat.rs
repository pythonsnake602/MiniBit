/*
    MiniBit - A Minecraft minigame server network written in Rust.
    Copyright (C) 2024  Cheezer1656 (https://github.com/Cheezer1656/)

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use valence::prelude::*;
use valence::protocol::packets::play::DamageTiltS2c;
use valence::protocol::sound::SoundCategory;
use valence::protocol::{Sound, VarInt, WritePacket};
use valence::math::Vec3Swizzles;
use valence::entity::EntityId;
use crate::duels::CombatState;

/// Handles sprint events to set bonus knockback state
pub fn handle_sprint_events<T>(
    sprinting: &mut EventReader<SprintEvent>,
    clients: &mut Query<T>,
) where
    T: bevy_ecs::query::QueryData,
    for<'a> T::Item<'a>: HasCombatState,
{
    for &SprintEvent { client, state } in sprinting.read() {
        if let Ok(mut client_query) = clients.get_mut(client) {
            client_query.get_combat_state().has_bonus_knockback = state == SprintState::Start;
        }
    }
}

/// Trait for types that have a mutable combat state
pub trait HasCombatState {
    fn get_combat_state(&mut self) -> &mut CombatState;
}

/// Apply knockback and sound effects for a combat interaction
/// Returns the calculated velocity vector
pub fn apply_combat_effects(
    attacker_client: &mut Client,
    _attacker_id: &EntityId,
    attacker_pos: &Position,
    attacker_has_bonus_knockback: bool,
    victim_client: &mut Client,
    victim_id: &EntityId,
    victim_pos: &Position,
) -> Vec3 {
    let victim_pos_xz = victim_pos.0.xz();
    let attacker_pos_xz = attacker_pos.0.xz();

    let dir = (victim_pos_xz - attacker_pos_xz).normalize().as_vec2();

    let knockback_xz = if attacker_has_bonus_knockback {
        18.0
    } else {
        8.0
    };
    let knockback_y = if attacker_has_bonus_knockback {
        8.432
    } else {
        6.432
    };

    let velocity = Vec3::new(dir.x * knockback_xz, knockback_y, dir.y * knockback_xz);

    victim_client.play_sound(
        Sound::EntityPlayerHurt,
        SoundCategory::Player,
        victim_pos.0,
        1.0,
        1.0,
    );
    victim_client.write_packet(&DamageTiltS2c {
        entity_id: VarInt(0),
        yaw: 0.0,
    });
    attacker_client.play_sound(
        Sound::EntityPlayerHurt,
        SoundCategory::Player,
        victim_pos.0,
        1.0,
        1.0,
    );
    attacker_client.write_packet(&DamageTiltS2c {
        entity_id: VarInt(victim_id.get()),
        yaw: 0.0,
    });

    velocity
}

/// Check if a combat interaction should be processed
pub fn should_process_combat(
    interaction: EntityInteraction,
    current_tick: i64,
    victim_last_attacked_tick: i64,
    attacker_game_id: Option<Entity>,
    victim_game_id: Option<Entity>,
) -> bool {
    interaction == EntityInteraction::Attack
        && current_tick - victim_last_attacked_tick >= 10
        && attacker_game_id == victim_game_id
}

/// Check if a combat interaction should be processed (with team check)
pub fn should_process_combat_with_teams(
    interaction: EntityInteraction,
    current_tick: i64,
    victim_last_attacked_tick: i64,
    attacker_team: u8,
    victim_team: u8,
    attacker_game_id: Option<Entity>,
    victim_game_id: Option<Entity>,
) -> bool {
    interaction == EntityInteraction::Attack
        && current_tick - victim_last_attacked_tick >= 10
        && attacker_team != victim_team
        && attacker_game_id == victim_game_id
}
