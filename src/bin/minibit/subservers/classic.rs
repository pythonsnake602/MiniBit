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

#![allow(clippy::type_complexity)]

use std::marker::PhantomData;
use std::path::PathBuf;
use bevy_ecs::query::QueryData;
use minibit_lib::combat;
use minibit_lib::duels::{CombatState, DefaultDuelsConfig, DuelsPlugin, EndGameEvent, Entities, PlayerGameState, StartGameEvent};
use valence::entity::living::Health;
use valence::entity::Velocity;
use valence::entity::{EntityId, EntityStatuses};
use valence::prelude::*;

pub fn main(path: PathBuf) {
    App::new()
        .add_plugins(DuelsPlugin::<DefaultDuelsConfig> { path, default_gamemode: GameMode::Adventure, copy_map: false, phantom: PhantomData })
        .add_plugins(DefaultPlugins)
        .add_systems(EventLoopUpdate, handle_combat_events)
        .add_systems(Update, (start_game, end_game, handle_oob_clients))
        .run();
}

fn start_game(
    mut clients: Query<&mut Inventory>,
    games: Query<&Entities>,
    mut start_game: EventReader<StartGameEvent>,
) {
    for event in start_game.read() {
        if let Ok(entities) = games.get(event.0) {
            for entity in entities.0.iter() {
                if let Ok(mut inv) = clients.get_mut(*entity) {
                    inv.set_slot(36, ItemStack::new(ItemKind::IronSword, 1, None));
                }
            }
        }
    }
}

fn end_game(
    mut clients: Query<&mut Inventory>,
    games: Query<&Entities>,
    mut start_game: EventReader<StartGameEvent>,
) {
    for event in start_game.read() {
        if let Ok(entities) = games.get(event.0) {
            for entity in entities.0.iter() {
                if let Ok(mut inv) = clients.get_mut(*entity) {
                    for slot in 0..inv.slot_count() {
                        inv.set_slot(slot, ItemStack::EMPTY);
                    }
                }
            }
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    client: &'static mut Client,
    id: &'static EntityId,
    pos: &'static Position,
    vel: &'static mut Velocity,
    health: &'static mut Health,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    gamestate: &'static PlayerGameState,
}

impl combat::HasCombatState for CombatQueryItem<'_> {
    fn get_combat_state(&mut self) -> &mut CombatState {
        self.state
    }
}

fn handle_combat_events(
    server: Res<Server>,
    mut clients: Query<CombatQuery>,
    mut sprinting: EventReader<SprintEvent>,
    mut interact_entity: EventReader<InteractEntityEvent>,
    mut end_game: EventWriter<EndGameEvent>,
) {
    combat::handle_sprint_events(&mut sprinting, &mut clients);

    for &InteractEntityEvent {
        client: attacker_client,
        entity: victim_client,
        interact: interaction,
        ..
    } in interact_entity.read()
    {
        let Ok([mut attacker, mut victim]) = clients.get_many_mut([attacker_client, victim_client])
        else {
            continue;
        };

        if !combat::should_process_combat(
            interaction,
            server.current_tick(),
            victim.state.last_attacked_tick,
            attacker.gamestate.game_id,
            victim.gamestate.game_id,
        ) {
            continue;
        }

        victim.state.last_attacked_tick = server.current_tick();

        let velocity = combat::apply_combat_effects(
            attacker.client,
            attacker.id,
            attacker.pos,
            attacker.state.has_bonus_knockback,
            victim.client,
            victim.id,
            victim.pos,
        );

        victim.client.set_velocity(velocity);

        let damage = 5.83;
        if victim.health.0 > damage {
            victim.health.0 -= damage;
        } else {
            end_game.send(EndGameEvent {
                game_id: victim.gamestate.game_id.unwrap(),
                loser: victim.gamestate.team,
            });
        }

        attacker.state.has_bonus_knockback = false;
    }
}

fn handle_oob_clients(
    positions: Query<(&Position, &PlayerGameState), With<Client>>,
    mut end_game: EventWriter<EndGameEvent>,
) {
    for (pos, gamestate) in positions.iter() {
        if pos.0.y < 0.0 && let Some(game_id) = gamestate.game_id {
            end_game.send(EndGameEvent {
                game_id,
                loser: gamestate.team,
            });
        }
    }
}
