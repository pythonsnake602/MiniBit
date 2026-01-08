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
use minibit_lib::duels::{CombatState, DefaultDuelsConfig, DuelsPlugin, EndGameEvent, PlayerGameState};
use valence::entity::{EntityId, EntityStatuses};
use valence::prelude::*;

pub fn main(path: PathBuf) {
    App::new()
        .add_plugins(DuelsPlugin::<DefaultDuelsConfig> { path, default_gamemode: GameMode::Adventure, copy_map: false, phantom: PhantomData })
        .add_plugins(DefaultPlugins)
        .add_systems(EventLoopUpdate, handle_combat_events)
        .add_systems(Update, handle_oob_clients)
        .run();
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    client: &'static mut Client,
    id: &'static EntityId,
    pos: &'static Position,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    gamestate: &'static PlayerGameState,
}

impl combat::HasCombatState for CombatQueryItem<'_> {
    fn get_combat_state(&mut self) -> &mut CombatState {
        &mut *self.state
    }
}

fn handle_combat_events(
    server: Res<Server>,
    mut clients: Query<CombatQuery>,
    mut sprinting: EventReader<SprintEvent>,
    mut interact_entity: EventReader<InteractEntityEvent>,
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
            &mut *attacker.client,
            attacker.id,
            attacker.pos,
            attacker.state.has_bonus_knockback,
            &mut *victim.client,
            victim.id,
            victim.pos,
        );

        victim.client.set_velocity(velocity);

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
