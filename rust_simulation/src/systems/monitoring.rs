use bevy::prelude::*;
use log::info;
use sysinfo::System;
use std::time::Duration;

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
            .insert_resource(MonitoringTimer(Timer::new(
                Duration::from_secs(5),
                TimerMode::Repeating,
            )))
            .add_systems(FixedUpdate, memory_monitoring_system);
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
            used_memory / 1024 / 1024,
            total_memory / 1024 / 1024
        );
    }
}
