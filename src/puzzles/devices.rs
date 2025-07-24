use std::cmp::max;
use bevy::prelude::*;
use crate::puzzles::simulation::{InputPort, OutputPort, Port};

#[derive(Component)]
pub enum Device {
    Empty,
}

pub struct DeviceEntities {
    pub base: Entity,
    pub input_ports: Vec<Entity>,
    pub output_ports: Vec<Entity>,
}

pub trait SpawnDeviceCommandExt {
    fn spawn_device(
        &mut self,
        device: Device,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> DeviceEntities;

    fn spawn_generic_device(
        &mut self,
        n_inputs: usize,
        n_outputs: usize,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> DeviceEntities;
}

impl<'w, 's> SpawnDeviceCommandExt for Commands<'w, 's> {
    fn spawn_device(
        &mut self,
        device: Device,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>
    ) -> DeviceEntities {
        match device {
            Device::Empty => self.spawn_generic_device(2, 2, meshes, materials),
        }
    }

    fn spawn_generic_device(
        &mut self,
        n_inputs: usize,
        n_outputs: usize,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> DeviceEntities {
        // Size calculations
        // TODO: I don't like that this command takes the mesh and material assets -
        //  I'd be keen to split the code between logic and visuals
        let port_width = 15.0;
        let port_height = 30.0;
        let gap_between_ports = 15.0;

        let base_width = 80.0;
        let base_height = (max(n_inputs, n_outputs) as f32)
            * (port_height + gap_between_ports)
            + gap_between_ports;

        let base_mesh = meshes.add(Rectangle::new(base_width, base_height));
        let port_mesh = meshes.add(Rectangle::new(port_width, port_height));

        let base_material = materials.add(Color::hsl(160.0, 0.95, 0.7));
        let input_port_material = materials.add(Color::hsl(340.0, 0.95, 0.7));
        let output_port_material = materials.add(Color::hsl(20.0, 0.95, 0.7));

        let base = self.spawn((
            Device::Empty,
            Mesh2d(base_mesh),
            MeshMaterial2d(base_material),
        )).id();

        // This might need some explanation: `input_port_span` is the distance between the top
        // and bottom input ports, measured from the _centre_ of those ports.
        // It is this way because it makes the maths easier later, trust me.
        let input_port_span = (n_inputs as f32 - 1.0) * (port_height + gap_between_ports);
        let input_port_start_y = input_port_span / 2.0;
        let mut input_ports = Vec::new();
        for i in 0..n_inputs {
            let input_port = self.spawn((
                Port(None),
                Mesh2d(port_mesh.clone()),
                MeshMaterial2d(input_port_material.clone()),
                Transform::from_xyz(
                    -base_width / 2.0,
                    input_port_start_y - i as f32 * (port_height + gap_between_ports),
                    1.0
                ),
            )).id();
            self.entity(base)
                .add_one_related::<InputPort>(input_port)
                .add_child(input_port);
            input_ports.push(input_port);
        }

        let output_port_span = (n_outputs as f32 - 1.0) * (port_height + gap_between_ports);
        let output_port_start_y = output_port_span / 2.0;
        let mut output_ports = Vec::new();
        for i in 0..n_outputs {
            let output_port = self.spawn((
                Port(None),
                Mesh2d(port_mesh.clone()),
                MeshMaterial2d(output_port_material.clone()),
                Transform::from_xyz(
                    base_width / 2.0,
                    output_port_start_y - i as f32 * (port_height + gap_between_ports),
                    1.0
                ),
            )).id();
            self.entity(base)
                .add_one_related::<OutputPort>(output_port)
                .add_child(output_port);
            output_ports.push(output_port);
        }

        DeviceEntities { base, input_ports, output_ports }
    }
}
