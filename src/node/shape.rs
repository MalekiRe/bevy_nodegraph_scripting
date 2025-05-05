use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::color::Color;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{
    ChildOf, Children, Component, EntityWorldMut, Luminance, Name, Plugin, Resource, Transform,
    Vec2, Vec3, World,
};
use bevy_prototype_lyon::prelude::{ShapeBuilder, ShapeBuilderBase, ShapePath};
use std::f32::consts::PI;

pub struct NodeShapePlugin;
impl Plugin for NodeShapePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeSettings>();
    }
}

#[derive(Resource, Clone)]
pub struct ShapeSettings {
    pub port_radius: f32,
    pub port_padding: f32,
    pub width: f32,
    pub min_height: f32,
    pub notch_width: f32,
    pub notch_height: f32,
    pub notch_x: f32,
}

impl Default for ShapeSettings {
    fn default() -> Self {
        Self {
            port_radius: 10.0,
            port_padding: 20.0,
            width: 250.0,
            min_height: 40.0,
            notch_width: 40.0,
            notch_height: 20.0,
            notch_x: 40.0,
        }
    }
}

#[derive(Clone)]
pub struct ShapeInfo {
    pub top_ports: Vec<PortDrawType>,
    pub bottom_ports: Vec<PortDrawType>,
    pub notch_type: NotchType,
    pub color: Color,
}

impl ShapeSettings {
    pub fn shape_data(&self, shape_info: ShapeInfo) -> ShapeData {
        let mut height = self.min_height;

        for _ in 0..(shape_info.top_ports.len() + shape_info.bottom_ports.len()) {
            height += self.port_radius * 2.0 + self.port_padding;
        }

        ShapeData {
            width: self.width,
            height,
            notch: Notch {
                notch_width: self.notch_width,
                notch_height: self.notch_height,
                notch_x: self.notch_x,
                notch_type: shape_info.notch_type,
            },
            ports: Ports {
                top_ports: shape_info.top_ports,
                bottom_ports: shape_info.bottom_ports,
                port_radius: self.port_radius,
                port_padding: self.port_padding,
            },
            color: shape_info.color,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum NotchType {
    TopAndBottom,
    Top,
    Bottom,
}

#[derive(Clone, PartialEq)]
pub enum ShapeType {
    Normal,
    BottomRightShadow,
    TopLeftOutline,
}

#[derive(Component, Clone, PartialEq)]
#[component(immutable)]
#[component(on_insert = on_insert_shape_data)]
pub struct ShapeData {
    pub width: f32,
    pub height: f32,
    pub notch: Notch,
    pub ports: Ports,
    pub color: Color,
}

#[derive(Component)]
struct Shadow;

fn on_insert_shape_data(mut world: DeferredWorld, hook_context: HookContext) {
    world
        .commands()
        .entity(hook_context.entity)
        .queue(|mut entity_world_mut: EntityWorldMut| {
            entity_world_mut.remove::<bevy::render::primitives::Aabb>();
            entity_world_mut.remove::<bevy::prelude::Mesh2d>();
            let shape_data = entity_world_mut.get::<ShapeData>().unwrap().clone();
            entity_world_mut.insert(
                ShapeBuilder::with(&shape_data.path(ShapeType::Normal))
                    .fill(shape_data.color)
                    .stroke((Color::srgba(0.0, 0.0, 0.0, 0.0), 0.0))
                    .build(),
            );
            entity_world_mut.insert(Name::new("OWO MY THING"));
        });
    world.commands().queue(move |world: &mut World| {
        let shape_data = world
            .entity(hook_context.entity)
            .get::<ShapeData>()
            .unwrap()
            .clone();
        if let Some(children) = world.entity(hook_context.entity).get::<Children>() {
            let children = children.iter().cloned().collect::<Vec<_>>();
            for child in children {
                if world.entity(child).contains::<Shadow>() {
                    world.commands().entity(child).despawn();
                }
            }
        }
        let y_adjust = (2.4 / shape_data.height) * 1.45;
        world.spawn((
            ShapeBuilder::with(&shape_data.path(ShapeType::BottomRightShadow))
                .fill(shape_data.color.darker(0.2))
                .stroke((Color::BLACK, 1.0))
                .build(),
            Transform::from_xyz(-0.5, -2.4, -0.02).with_scale(Vec3::new(
                1.015,
                1.0 + (y_adjust),
                0.05,
            )),
            Shadow,
            ChildOf(hook_context.entity),
        ));
    });
}

impl ShapeData {
    pub fn port_position(&self, port_position: PortPosition) -> Vec2 {
        let (height_offset, width_offset, port_index) = match port_position {
            PortPosition::BottomLeft(port_index) => (
                self.height / 2.0 + self.ports.port_radius + self.ports.port_padding,
                0.0,
                port_index,
            ),
            PortPosition::BottomRight(port_index) => (
                self.height / 2.0 + self.ports.port_radius + self.ports.port_padding,
                self.width,
                port_index,
            ),
            PortPosition::TopLeft(port_index) => (self.height, 0.0, port_index),
            PortPosition::TopRight(port_index) => (self.height, self.width, port_index),
        };
        let port = port_index + 1;
        Vec2::new(
            width_offset,
            height_offset - port as f32 * (self.ports.port_radius + self.ports.port_padding),
        )
    }

    pub fn path(&self, shape_type: ShapeType) -> ShapePath {
        let mut path_builder = ShapePath::new()
            // Start at top-left
            .move_to(Vec2::new(0., 0.));

        path_builder = self.notch.bottom_notch(path_builder);

        // Continue along top edge
        path_builder = path_builder.line_to(Vec2::new(self.width, 0.));

        'draw_ports: {
            match shape_type {
                ShapeType::BottomRightShadow | ShapeType::TopLeftOutline => break 'draw_ports,
                ShapeType::Normal => {}
            }
            for (position, draw_type) in self
                .ports
                .top_ports
                .iter()
                .enumerate()
                .map(|(index, draw_type)| (PortPosition::TopRight(index), draw_type))
                .chain(
                    self.ports
                        .bottom_ports
                        .iter()
                        .enumerate()
                        .map(|(index, draw_type)| (PortPosition::BottomRight(index), draw_type)),
                )
                .map(|(port_position, draw_type)| (self.port_position(port_position), draw_type))
            {
                match draw_type {
                    PortDrawType::Flow => {
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x, position.y - self.ports.port_radius));
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x + self.ports.port_radius, position.y));
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x, position.y + self.ports.port_radius));
                    }
                    PortDrawType::Data => {
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x, position.y - self.ports.port_radius));
                        path_builder = path_builder.arc(
                            position,
                            Vec2::splat(self.ports.port_radius),
                            PI,
                            0.0,
                        );
                    }
                }
            }
        }

        // Right edge
        path_builder = path_builder.line_to(Vec2::new(self.width, self.height));

        path_builder = self.notch.top_notch(self, path_builder);

        // Continue along bottom edge
        path_builder = path_builder.line_to(Vec2::new(0., self.height));

        'draw_ports: {
            match shape_type {
                ShapeType::BottomRightShadow | ShapeType::TopLeftOutline => break 'draw_ports,
                ShapeType::Normal => {}
            }
            for (position, draw_type) in self
                .ports
                .top_ports
                .iter()
                .enumerate()
                .map(|(index, draw_type)| (PortPosition::TopLeft(index), draw_type))
                .chain(
                    self.ports
                        .bottom_ports
                        .iter()
                        .enumerate()
                        .map(|(index, draw_type)| (PortPosition::BottomLeft(index), draw_type)),
                )
                .map(|(port_position, draw_type)| (self.port_position(port_position), draw_type))
                .rev()
            {
                match draw_type {
                    PortDrawType::Flow => {
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x, position.y - self.ports.port_radius));
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x - self.ports.port_radius, position.y));
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x, position.y + self.ports.port_radius));
                    }
                    PortDrawType::Data => {
                        path_builder = path_builder
                            .line_to(Vec2::new(position.x, position.y - self.ports.port_radius));
                        path_builder = path_builder.arc(
                            position,
                            Vec2::splat(self.ports.port_radius),
                            -PI,
                            0.0,
                        );
                    }
                }
            }
        }

        // Left edge back up
        path_builder = path_builder.line_to(Vec2::new(0., 0.));

        path_builder.close()
    }
}

#[derive(Component, Copy, Clone)]
#[require(Transform)]
pub enum PortPosition {
    TopLeft(usize),
    TopRight(usize),
    BottomLeft(usize),
    BottomRight(usize),
}

#[derive(Clone, PartialEq)]
pub enum PortDrawType {
    Flow,
    Data,
}

#[derive(Clone, PartialEq)]
pub struct Notch {
    pub notch_width: f32,
    pub notch_height: f32,
    pub notch_x: f32,
    pub notch_type: NotchType,
}

impl Notch {
    pub fn bottom_notch(&self, path_builder: ShapePath) -> ShapePath {
        match self.notch_type {
            NotchType::Top => return path_builder,
            NotchType::Bottom | NotchType::TopAndBottom => (),
        }
        // Move to start of top notch
        path_builder
            .line_to(Vec2::new(self.notch_x, 0.))
            // Carve top notch inward (hexagonal notch)
            .line_to(Vec2::new(
                self.notch_x + self.notch_width * 0.25,
                -self.notch_height,
            ))
            .line_to(Vec2::new(
                self.notch_x + self.notch_width * 0.75,
                -self.notch_height,
            ))
            .line_to(Vec2::new(self.notch_x + self.notch_width, 0.))
    }
    pub fn top_notch(&self, shape_data: &ShapeData, mut path_builder: ShapePath) -> ShapePath {
        match self.notch_type {
            NotchType::Bottom => return path_builder,
            NotchType::Top | NotchType::TopAndBottom => (),
        }
        // Move to start of bottom bump
        path_builder = path_builder.line_to(Vec2::new(
            self.notch_x + self.notch_width,
            shape_data.height,
        ));

        // Add bump outward (hexagonal bump)
        path_builder = path_builder.line_to(Vec2::new(
            self.notch_x + self.notch_width * 0.75,
            shape_data.height - self.notch_height,
        ));
        path_builder = path_builder.line_to(Vec2::new(
            self.notch_x + self.notch_width * 0.25,
            shape_data.height - self.notch_height,
        ));
        path_builder = path_builder.line_to(Vec2::new(self.notch_x, shape_data.height));
        path_builder
    }
}

#[derive(Clone, PartialEq)]
pub struct Ports {
    pub top_ports: Vec<PortDrawType>,
    pub bottom_ports: Vec<PortDrawType>,
    pub port_radius: f32,
    pub port_padding: f32,
}
