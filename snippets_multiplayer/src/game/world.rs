use super::common::{Rect2F, Rect2X, Vector2F};
use rand::seq::IndexedRandom;

#[derive(Debug)]
pub struct World {
    new_entity_id: EntityId,
    entities: Vec<Entity>,
    tile_size: f32,
}

#[derive(Debug, PartialEq)]
pub enum EntityState {
    Idle,
    Moving {
        from_position: Vector2F,
        destination: Vector2F,
    },
}

#[derive(Debug)]
pub struct NpcController {
    spawnpoint: Vector2F,
    roaming_range: Option<f32>,
    change_destination_counter: u32,
}

#[derive(Debug)]
pub struct PlayerController {
}

#[derive(Debug)]
pub enum EntityController {
    Npc(NpcController),
    Player(PlayerController),
}

pub type EntityId = u32;

#[derive(Debug)]
pub struct EntityStats {
    movement_speed: f32,
}

#[derive(Debug)]
pub struct Entity {
    id: u32,
    pub name: String,
    pub position: Vector2F,
    state: EntityState,
    stats: EntityStats,
    controller: EntityController,
}

const PLAYER_MOVEMENT_SPEED: f32 = 0.5;
const NPC_MOVEMENT_SPEED: f32 = 0.35;
const NPC_DIRECTION_SELECTION_TICKS: u32 = 3;

impl World {
    pub fn new() -> Self {
        const TILE_SIZE_SIDE: f32 = 5.0;
        println!("World created");
        Self {
            new_entity_id: 0,
            entities: vec![],
            tile_size: TILE_SIZE_SIDE,
        }
    }

    // TODO snap to grid
    pub fn create_entity_npc<S: AsRef<str>>(&mut self, name: S, intial_position: Vector2F) -> EntityId {
        self.create_entity(
            name, 
            intial_position, 
            EntityStats {
                movement_speed: NPC_MOVEMENT_SPEED
            }, 
            EntityController::Npc(NpcController {
                spawnpoint: intial_position,
                roaming_range: Some(3.0),
                change_destination_counter: NPC_DIRECTION_SELECTION_TICKS
            })
        )
    }

    // TODO snap to grid
    pub fn create_entity<S: AsRef<str>>(&mut self, name: S, intial_position: Vector2F, stats: EntityStats, controller: EntityController) -> EntityId {
        let new_id = self.new_entity_id;
        self.new_entity_id += 1;

        let entity = Entity { 
            id: new_id, 
            name: name.as_ref().to_string(),
            position: intial_position,
            state: EntityState::Idle,
            stats,
            controller
        };

        self.entities.push(entity);
        new_id
    }

    pub fn get_entity_by_id(&self, entity_id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == entity_id)
    }

    pub fn get_entity_by_id_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == entity_id)
    }

    pub fn is_tile_occupied(&self, tile_position: &Vector2F) -> bool {
        let checked_tile = Rect2F::new(tile_position.x, tile_position.y, self.tile_size, self.tile_size);
        for entity in self.entities.iter() {
            let is_colliding = match entity.state {
                EntityState::Idle => checked_tile.contains(&entity.position),
                EntityState::Moving { from_position, destination } => {
                    checked_tile.contains(&from_position) || checked_tile.contains(&destination)
                },
            };

            if is_colliding {
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self) {
        println!("World tick");

        // TODO Do it better
        let occupied_positions: Vec<_> = self
        .entities
        .iter()
        .map(|e| match &e.state {
            EntityState::Moving { destination, .. } => *destination,
            EntityState::Idle => e.position,
        })
        .collect();

        self.entities.iter_mut().for_each(|e| {
            println!(" - {e:?}");

            if let EntityState::Moving { from_position, destination } = e.state {
                // Interpolate movement
                // Destination was checked when entity was idle -> no need to check
                let direction = (destination - from_position).normal();
                let previous_location_to_destination = destination - e.position;
                e.position += direction * e.stats.movement_speed;
                let new_location_to_destination = destination - e.position;                

                // Check if destination was reached, by checking change of dot product
                let destination_was_reached = previous_location_to_destination.dot(new_location_to_destination) < 0.0;

                // Align to destination, Change state to Idle and reset counter
                if destination_was_reached {
                    println!("   {} reached destination {:?}", e.name, destination);
                    e.state = EntityState::Idle;
                    if let EntityController::Npc(npc_controller) = &mut e.controller {
                        npc_controller.change_destination_counter = NPC_DIRECTION_SELECTION_TICKS;
                    }
                    e.position = destination;
                }
            }

            match &mut e.controller {
                EntityController::Npc(npc_controller) => {
                    // Check if is capable of roaming
                    if let Some(_range) = npc_controller.roaming_range {

                        // Every `xx` try selecting new destination
                        // If destination is not valid (out of range, occupied or reserved)
                        // then try next time.
                        if e.state == EntityState::Idle {
                            // Count down, at counting exhaustion try selecting new destination
                            if npc_controller.change_destination_counter > 0 {
                                npc_controller.change_destination_counter -= 1;
                            } else {
                                // Triggered -> try selecting new destination
                                let directions = [
                                    Vector2F::new(1.0, 0.0),
                                    Vector2F::new(-1.0, 0.0),
                                    Vector2F::new(0.0, 1.0),
                                    Vector2F::new(0.0, -1.0),
                                ];

                                let random_direction = directions.choose(&mut rand::rng()).unwrap();

                                let destination_position = e.position + (*random_direction * self.tile_size);

                                if !occupied_positions.contains(&destination_position) {
                                    println!("   Setting new destination {destination_position:?}!");
                                    e.state = EntityState::Moving {
                                        from_position: e.position,
                                        destination: destination_position
                                    };
                                } else {
                                    println!("   Tile {destination_position:?} already occupied!");
                                }
                            }
                        }

                    }
                },
                EntityController::Player(_player_controller) => {
                    unimplemented!("Player controller usage in tick missing")
                },
            }
        });
    }
}

#[test]
fn test_world_creation() {
    let world = World::new();
    assert_eq!(world.new_entity_id, 0);
}

#[test]
fn test_world_entity_creation_should_increase_entities_count() {
    let mut world = World::new();
    assert_eq!(world.new_entity_id, 0);

    let new_entity_id = world.create_entity_npc("Bob", Vector2F::new(1.0, 2.0));
    assert_eq!(new_entity_id, 0);

    assert_eq!(world.new_entity_id, 1);
}

#[test]
fn test_world_entity_access() {
    let entity_name = "Bob";
    let entity_position = Vector2F::new(1.0, 2.0);

    let mut world = World::new();
    let new_entity_id = world.create_entity_npc(entity_name, entity_position);

    let entity = world.get_entity_by_id(new_entity_id).unwrap();
    assert_eq!(entity.name, entity_name);
    assert_eq!(entity.position, entity_position);
    assert_eq!(entity.state, EntityState::Idle);
}

#[test]
fn test_world_entity_translate() {
    let entity_initial_position = Vector2F::new(1.0, 2.0);
    let translation = Vector2F::new(100.0, 500.0);

    let mut world = World::new();
    let new_entity_id = world.create_entity_npc("Bob", entity_initial_position);

    let entity = world.get_entity_by_id_mut(new_entity_id).unwrap();
    entity.position += translation;
    assert_eq!(entity.position, entity_initial_position + translation);
}