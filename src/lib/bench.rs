use std::time::Instant;
use valence::prelude::*;

#[derive(Resource)]
struct TickStart(Instant);

pub struct BenchPlugin;

impl Plugin for BenchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, record_tick_start_time)
            .add_systems(Last, print_tick_time);
    }
}

fn record_tick_start_time(mut commands: Commands) {
    commands.insert_resource(TickStart(Instant::now()));
}

fn print_tick_time(server: Res<Server>, time: Res<TickStart>, clients: Query<(), With<Client>>) {
    let tick = server.current_tick();
    if tick % (i64::from(server.tick_rate().get()) / 2) == 0 {
        let client_count = clients.iter().len();

        let millis = time.0.elapsed().as_secs_f32() * 1000.0;
        println!("Tick={tick}, MSPT={millis:.04}ms, Clients={client_count}");
    }
}
