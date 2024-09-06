use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineMeshExt, OutlineMode, OutlineVolume};
use bevy_mod_raycast::prelude::*;

use crate::core::system_sets::SpawningSet;

#[derive(Reflect)]
struct InteractionRaycastSet;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_interaction.after(SpawningSet));
        app.add_systems(Update, check_looked_at);
        app.add_plugins(DeferredRaycastingPlugin::<InteractionRaycastSet>::default());
        app.insert_resource(RaycastPluginState::<InteractionRaycastSet>::default());
    }
}

#[derive(Component)]
pub struct Interactor;

#[derive(Component)]
pub struct Interactable;

fn setup_interaction(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    interactors: Query<Entity, With<Interactor>>,
    interactables: Query<(Entity, &Handle<Mesh>), With<Interactable>>,
) {
    info!(
        "Setting up interactions. Interactors: {}, Interactables: {}.",
        interactors.iter().len(),
        interactables.iter().len(),
    );

    for interactor in interactors.iter() {
        commands
            .entity(interactor)
            .insert(RaycastSource::<InteractionRaycastSet>::new_transform_empty());
    }

    for (interactable, mesh_handle) in interactables.iter() {
        // Allow the entity to accept raycasts
        commands.entity(interactable).insert(RaycastMesh::<InteractionRaycastSet>::default());

        // Set up the mesh for outline generation
        let mesh = match meshes.get_mut(mesh_handle.id()) {
            Some(mesh) => mesh,
            None => {
                // Something has gone weird if we get here.
                // I don't know Bevy well enough to know if there's a situation that could
                //  get us here, so I'm just going to emit a warning and skip this
                //  interactable.
                // (If an interactable object doesn't have a mesh, it doesn't really make sense
                //  for it to be interactable, because the player won't be able to see it)
                warn!("Couldn't get mesh while setting up an Interactable. This Interactable will not have an outline.");
                continue;
            },
        };
        match mesh.generate_outline_normals() {
            Ok(_) => {
                commands.entity(interactable).insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: false,
                        colour: Color::srgb(0.0, 1.0, 0.0),
                        width: 10.0,
                    },
                    mode: OutlineMode::RealVertex,
                    ..default()
                });
            },
            Err(why) => {
                warn!("Problem while generating normals for the mesh of an Interactable. This Interactable will not have an outline.");
                warn!("Normal generation error: {}", why);
            }
        }
    }
}

fn check_looked_at(
    mut query: Query<(&mut OutlineVolume, &RaycastMesh<InteractionRaycastSet>), With<Interactable>>,
) {
    for (mut volume, mesh) in query.iter_mut() {
        volume.visible = mesh.intersections().len() > 0;
    }
}
