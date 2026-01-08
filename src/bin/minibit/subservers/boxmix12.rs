/*
    MiniBit - A Minecraft minigame server network written in Rust.
    Copyright (C) 2026  Cheezer1656 (https://github.com/Cheezer1656/)

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
use minibit_lib::duels::{DefaultDuelsConfig, DuelsPlugin, Entities, PlayerGameState, StartGameEvent};
use valence::interact_block::InteractBlockEvent;
use valence::prelude::*;

#[derive(Component, Default)]
struct BoxMixState {
    coins: u32,
    boxes_opened: u32,
}

#[derive(Resource)]
struct BoxLocations {
    boxes: Vec<BlockPos>,
    shop_locations: Vec<BlockPos>,
}

impl Default for BoxLocations {
    fn default() -> Self {
        Self {
            boxes: vec![
                BlockPos::new(5, 10, 5),
                BlockPos::new(-5, 10, 5),
                BlockPos::new(5, 10, -5),
                BlockPos::new(-5, 10, -5),
            ],
            shop_locations: vec![
                BlockPos::new(0, 10, 0),
            ],
        }
    }
}

pub fn main(path: PathBuf) {
    App::new()
        .add_plugins(DuelsPlugin::<DefaultDuelsConfig> { 
            path, 
            default_gamemode: GameMode::Adventure, 
            copy_map: false, 
            phantom: PhantomData 
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(BoxLocations::default())
        .add_systems(
            Update,
            (
                init_clients.after(minibit_lib::duels::map::init_clients::<DefaultDuelsConfig>),
                start_game,
                handle_block_interactions,
                update_scoreboard,
            ),
        )
        .run();
}

fn init_clients(clients: Query<Entity, Added<Client>>, mut commands: Commands) {
    for client in clients.iter() {
        commands.entity(client).insert(BoxMixState::default());
    }
}

fn start_game(
    mut clients: Query<(&mut Inventory, &mut BoxMixState), With<Client>>,
    games: Query<&Entities>,
    mut start_game: EventReader<StartGameEvent>,
) {
    for event in start_game.read() {
        if let Ok(entities) = games.get(event.0) {
            for entity in entities.0.iter() {
                if let Ok((mut inventory, mut state)) = clients.get_mut(*entity) {
                    // Reset state
                    state.coins = 0;
                    state.boxes_opened = 0;
                    
                    // Clear inventory
                    for slot in 0..inventory.slot_count() {
                        inventory.set_slot(slot, ItemStack::EMPTY);
                    }
                    
                    // Give starting items
                    inventory.set_slot(36, ItemStack::new(ItemKind::WoodenSword, 1, None));
                }
            }
        }
    }
}

fn handle_block_interactions(
    mut clients: Query<(&mut Client, &mut Inventory, &mut BoxMixState, &PlayerGameState)>,
    mut events: EventReader<InteractBlockEvent>,
    box_locations: Res<BoxLocations>,
) {
    for event in events.read() {
        if let Ok((mut client, mut inventory, mut state, gamestate)) = clients.get_mut(event.client) {
            // Only handle interactions if player is in a game
            if gamestate.game_id.is_none() {
                continue;
            }

            let block_pos = event.position;

            // Check if player clicked a box
            if box_locations.boxes.contains(&block_pos) {
                state.boxes_opened += 1;
                state.coins += 10;
                
                client.send_chat_message(format!(
                    "§a§lBox opened! +10 coins (Total: {} coins)",
                    state.coins
                ));
                
                // Give random resource
                let resources = [
                    (ItemKind::IronIngot, 1),
                    (ItemKind::GoldIngot, 1),
                    (ItemKind::Diamond, 1),
                    (ItemKind::Emerald, 1),
                ];
                
                let resource_idx = (state.boxes_opened as usize) % resources.len();
                let (item, count) = resources[resource_idx];
                
                // Find empty slot or slot with same item
                for slot in 36..45 {
                    let current = inventory.slot(slot);
                    if current.item == ItemKind::Air {
                        inventory.set_slot(slot, ItemStack::new(item, count, None));
                        break;
                    } else if current.item == item && current.count < 64 {
                        let new_count = current.count + count;
                        inventory.set_slot(slot, ItemStack::new(item, new_count, None));
                        break;
                    }
                }
            }
            
            // Check if player clicked shop
            else if box_locations.shop_locations.contains(&block_pos) {
                if state.coins >= 50 {
                    state.coins -= 50;
                    client.send_chat_message(format!(
                        "§6§lPurchased Iron Sword! (-50 coins, Remaining: {} coins)",
                        state.coins
                    ));
                    
                    inventory.set_slot(36, ItemStack::new(ItemKind::IronSword, 1, None));
                } else {
                    client.send_chat_message(format!(
                        "§c§lNot enough coins! Need 50, you have {}",
                        state.coins
                    ));
                }
            }
        }
    }
}

fn update_scoreboard(
    _clients: Query<(&Client, &BoxMixState, &PlayerGameState)>,
) {
    // Could be used for future UI updates if needed
}
