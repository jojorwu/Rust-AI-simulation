use bevy::prelude::*;
use log::info;
use sysinfo::System;
use std::time::Duration;

#[derive(Resource, Default)]
pub struct MemoryLimitReached(pub bool);

#[derive(Resource)]
pub struct MonitoringState {
    pub sys: System,
}

impl Default for MonitoringState {
    fn default() -> Self {
        MonitoringState {
            sys: System::new_all(),
        }
    }
}

#[derive(Resource)]
struct MonitoringTimer(Timer);

pub struct MonitoringPlugin;


impl Plugin for MonitoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MonitoringState>()
            .init_resource::<MemoryLimitReached>()
            .insert_resource(MonitoringTimer(Timer::new(
                Duration::from_secs(5),
                TimerMode::Repeating,
            )))
            .add_systems(FixedUpdate, memory_monitoring_system);
            // .add_systems(
            //     FixedUpdate,
            //     memory_limiting_system
            //         .run_if(|config: Res<Config>| config.performance.enable_ram_limit),
            // );
    }
}

fn memory_monitoring_system(
    time: Res<Time>,
    mut timer: ResMut<MonitoringTimer>,
    mut monitoring_state: ResMut<MonitoringState>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        monitoring_state.sys.refresh_memory();
        let total_memory = monitoring_state.sys.total_memory();
        let used_memory = monitoring_state.sys.used_memory();
        info!(
            "Memory usage: {} / {} MB",
            used_memory / (1024 * 1024),
            total_memory / (1024 * 1024)
        );
    }
}

// fn memory_limiting_system(
//     mut memory_limit_reached: ResMut<MemoryLimitReached>,
//     monitoring_state: Res<MonitoringState>,
//     config: Res<Config>,
// ) {
//     if memory_limit_reached.0 {
//         return;
//     }
//     let used_memory_gb = monitoring_state.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
//     if used_memory_gb > config.performance.ram_limit_gb as f64 {
//         memory_limit_reached.0 = true;
//         info!("RAM limit reached! No new agents will be spawned.");
//     }
// }
