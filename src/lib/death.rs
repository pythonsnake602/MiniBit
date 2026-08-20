use crate::duels::CombatState;
use chunkedge::prelude::*;
use chunkedge::protocol::Sound;
use chunkedge::protocol::sound::SoundCategory;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeathSet;

#[derive(EntityEvent)]
pub struct DeathEvent {
    pub entity: Entity,
    pub show: bool,
}

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(play_death_sound);
    }
}

pub fn play_death_sound(
    event: On<DeathEvent>,
    mut clients: Query<(&mut Client, &Position)>,
    states: Query<&CombatState>,
) {
    let entity = event.entity;
    let show = event.show;

    let Ok(state) = states.get(entity) else {
        return;
    };
    let Some(attacker) = state.last_attacker else {
        return;
    };
    if let Ok((mut client, pos)) = clients.get_mut(attacker) && show
    {
        client.play_sound(
            Sound::EntityArrowHitPlayer,
            SoundCategory::Player,
            pos.0,
            1.0,
            1.0,
        );
    }
}
